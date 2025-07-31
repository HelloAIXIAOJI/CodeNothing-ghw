// CodeNothing 编译时类型检查器
// 在代码执行前进行静态类型分析和验证

use crate::ast::{Statement, Expression, Type, Function, Parameter, Program, Class, Enum};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct TypeCheckError {
    pub message: String,
    pub line: Option<usize>,
    pub column: Option<usize>,
}

impl TypeCheckError {
    pub fn new(message: String) -> Self {
        Self {
            message,
            line: None,
            column: None,
        }
    }
    
    pub fn with_location(message: String, line: usize, column: usize) -> Self {
        Self {
            message,
            line: Some(line),
            column: Some(column),
        }
    }
}

pub struct TypeChecker {
    // 变量类型表
    variable_types: HashMap<String, Type>,
    // 函数签名表
    function_signatures: HashMap<String, (Vec<Type>, Type)>, // (参数类型, 返回类型)
    // 类定义表
    class_definitions: HashMap<String, HashMap<String, Type>>, // 类名 -> 字段名 -> 字段类型
    // 枚举定义表
    enum_definitions: HashMap<String, Vec<String>>, // 枚举名 -> 变体列表
    // 错误收集
    errors: Vec<TypeCheckError>,
    // 当前函数的返回类型
    current_function_return_type: Option<Type>,
}

impl TypeChecker {
    pub fn new() -> Self {
        Self {
            variable_types: HashMap::new(),
            function_signatures: HashMap::new(),
            class_definitions: HashMap::new(),
            enum_definitions: HashMap::new(),
            errors: Vec::new(),
            current_function_return_type: None,
        }
    }
    
    // 主要的类型检查入口
    pub fn check_program(&mut self, program: &Program) -> Result<(), Vec<TypeCheckError>> {
        // 第一遍：收集所有函数、类、枚举的定义
        self.collect_program_definitions(program);

        // 第二遍：检查所有函数的类型
        for function in &program.functions {
            self.check_function_declaration(function);
        }

        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(self.errors.clone())
        }
    }
    
    // 收集程序定义阶段
    fn collect_program_definitions(&mut self, program: &Program) {
        // 收集函数定义
        for function in &program.functions {
            let param_types: Vec<Type> = function.parameters.iter()
                .map(|p| p.param_type.clone())
                .collect();
            self.function_signatures.insert(
                function.name.clone(),
                (param_types, function.return_type.clone())
            );
        }

        // 收集类定义
        for class in &program.classes {
            let mut fields = HashMap::new();
            for field in &class.fields {
                fields.insert(field.name.clone(), field.field_type.clone());
            }
            self.class_definitions.insert(class.name.clone(), fields);
        }

        // 收集枚举定义
        for enum_decl in &program.enums {
            let variants: Vec<String> = enum_decl.variants.iter()
                .map(|v| v.name.clone())
                .collect();
            self.enum_definitions.insert(enum_decl.name.clone(), variants);
        }
    }
    
    // 检查语句类型
    fn check_statement(&mut self, statement: &Statement) {
        match statement {
            Statement::VariableDeclaration(name, declared_type, init_expr) => {
                self.check_variable_declaration(name, declared_type, &Some(init_expr.clone()));
            },
            Statement::ConstantDeclaration(name, declared_type, init_expr) => {
                self.check_variable_declaration(name, declared_type, &Some(init_expr.clone()));
            },
            Statement::VariableAssignment(name, expr) => {
                self.check_assignment(name, expr);
            },
            Statement::Return(expr) => {
                self.check_return_statement(&Some(expr.clone()));
            },
            Statement::IfElse(condition, then_block, else_blocks) => {
                self.check_if_else_statement(condition, then_block, else_blocks);
            },
            Statement::WhileLoop(condition, body) => {
                self.check_while_statement(condition, body);
            },
            Statement::ForLoop(var_name, start, end, body) => {
                self.check_for_loop_statement(var_name, start, end, body);
            },
            Statement::FunctionCallStatement(expr) => {
                self.check_expression(expr);
            },
            _ => {
                // 其他语句类型的检查
            }
        }
    }
    
    // 检查变量声明
    fn check_variable_declaration(&mut self, name: &str, declared_type: &Type, init_expr: &Option<Expression>) {
        if let Some(expr) = init_expr {
            let expr_type = self.infer_expression_type(expr);
            
            // 如果声明类型不是Auto，检查类型匹配
            if !matches!(declared_type, Type::Auto) {
                if !self.types_compatible(declared_type, &expr_type) {
                    self.errors.push(TypeCheckError::new(
                        format!("类型不匹配: 变量 '{}' 声明为 {:?}，但初始化表达式类型为 {:?}",
                                name, declared_type, expr_type)
                    ));
                }
            }
            
            // 记录变量类型
            let final_type = if matches!(declared_type, Type::Auto) {
                expr_type
            } else {
                declared_type.clone()
            };
            
            self.variable_types.insert(name.to_string(), final_type);
        } else {
            // 没有初始化表达式，直接记录声明类型
            self.variable_types.insert(name.to_string(), declared_type.clone());
        }
    }
    
    // 检查赋值语句
    fn check_assignment(&mut self, name: &str, expr: &Expression) {
        let expr_type = self.infer_expression_type(expr);
        
        if let Some(var_type) = self.variable_types.get(name) {
            if !self.types_compatible(var_type, &expr_type) {
                self.errors.push(TypeCheckError::new(
                    format!("类型不匹配: 变量 '{}' 类型为 {:?}，但赋值表达式类型为 {:?}",
                            name, var_type, expr_type)
                ));
            }
        } else {
            self.errors.push(TypeCheckError::new(
                format!("未声明的变量: '{}'", name)
            ));
        }
    }
    
    // 检查函数声明
    fn check_function_declaration(&mut self, func: &Function) {
        // 保存当前函数返回类型
        let prev_return_type = self.current_function_return_type.clone();
        self.current_function_return_type = Some(func.return_type.clone());
        
        // 创建新的作用域
        let prev_variables = self.variable_types.clone();
        
        // 添加参数到变量表
        for param in &func.parameters {
            self.variable_types.insert(param.name.clone(), param.param_type.clone());
        }
        
        // 检查函数体
        for statement in &func.body {
            self.check_statement(statement);
        }
        
        // 恢复作用域
        self.variable_types = prev_variables;
        self.current_function_return_type = prev_return_type;
    }
    
    // 检查返回语句
    fn check_return_statement(&mut self, expr: &Option<Expression>) {
        // 先克隆期望的返回类型以避免借用冲突
        if let Some(expected_return_type) = self.current_function_return_type.clone() {
            if let Some(return_expr) = expr {
                let return_type = self.infer_expression_type(return_expr);
                if !self.types_compatible(&expected_return_type, &return_type) {
                    self.errors.push(TypeCheckError::new(
                        format!("返回类型不匹配: 期望 {:?}，但返回 {:?}",
                                expected_return_type, return_type)
                    ));
                }
            } else {
                // 没有返回表达式，检查是否应该返回void
                if !matches!(expected_return_type, Type::Void) {
                    self.errors.push(TypeCheckError::new(
                        format!("缺少返回值: 函数应该返回 {:?}", expected_return_type)
                    ));
                }
            }
        }
    }
    
    // 检查if语句
    fn check_if_statement(&mut self, condition: &Expression, then_block: &[Statement], else_block: &Option<Vec<Statement>>) {
        let condition_type = self.infer_expression_type(condition);
        if !matches!(condition_type, Type::Bool) {
            self.errors.push(TypeCheckError::new(
                format!("if条件必须是bool类型，但得到 {:?}", condition_type)
            ));
        }
        
        // 检查then块
        for statement in then_block {
            self.check_statement(statement);
        }
        
        // 检查else块
        if let Some(else_statements) = else_block {
            for statement in else_statements {
                self.check_statement(statement);
            }
        }
    }
    
    // 检查while语句
    fn check_while_statement(&mut self, condition: &Expression, body: &[Statement]) {
        let condition_type = self.infer_expression_type(condition);
        if !matches!(condition_type, Type::Bool) {
            self.errors.push(TypeCheckError::new(
                format!("while条件必须是bool类型，但得到 {:?}", condition_type)
            ));
        }
        
        for statement in body {
            self.check_statement(statement);
        }
    }
    
    // 检查if-else语句
    fn check_if_else_statement(&mut self, condition: &Expression, then_block: &[Statement],
                              else_blocks: &[(Option<Expression>, Vec<Statement>)]) {
        let condition_type = self.infer_expression_type(condition);
        if !matches!(condition_type, Type::Bool) {
            self.errors.push(TypeCheckError::new(
                format!("if条件必须是bool类型，但得到 {:?}", condition_type)
            ));
        }

        // 检查then块
        for statement in then_block {
            self.check_statement(statement);
        }

        // 检查else-if和else块
        for (else_condition, else_statements) in else_blocks {
            if let Some(else_cond) = else_condition {
                let else_condition_type = self.infer_expression_type(else_cond);
                if !matches!(else_condition_type, Type::Bool) {
                    self.errors.push(TypeCheckError::new(
                        format!("else-if条件必须是bool类型，但得到 {:?}", else_condition_type)
                    ));
                }
            }

            for statement in else_statements {
                self.check_statement(statement);
            }
        }
    }

    // 检查for循环语句
    fn check_for_loop_statement(&mut self, var_name: &str, start: &Expression, end: &Expression, body: &[Statement]) {
        let start_type = self.infer_expression_type(start);
        let end_type = self.infer_expression_type(end);

        // 检查范围类型
        if !matches!(start_type, Type::Int | Type::Long) {
            self.errors.push(TypeCheckError::new(
                format!("for循环起始值必须是整数类型，但得到 {:?}", start_type)
            ));
        }

        if !matches!(end_type, Type::Int | Type::Long) {
            self.errors.push(TypeCheckError::new(
                format!("for循环结束值必须是整数类型，但得到 {:?}", end_type)
            ));
        }

        // 添加循环变量到作用域
        let prev_var_type = self.variable_types.get(var_name).cloned();
        self.variable_types.insert(var_name.to_string(), start_type);

        // 检查循环体
        for statement in body {
            self.check_statement(statement);
        }

        // 恢复变量作用域
        if let Some(prev_type) = prev_var_type {
            self.variable_types.insert(var_name.to_string(), prev_type);
        } else {
            self.variable_types.remove(var_name);
        }
    }
    
    // 检查表达式（不返回类型）
    fn check_expression(&mut self, expr: &Expression) {
        self.infer_expression_type(expr);
    }

    // 推断表达式类型
    fn infer_expression_type(&mut self, expr: &Expression) -> Type {
        match expr {
            Expression::IntLiteral(_) => Type::Int,
            Expression::FloatLiteral(_) => Type::Float,
            Expression::BoolLiteral(_) => Type::Bool,
            Expression::StringLiteral(_) => Type::String,
            Expression::LongLiteral(_) => Type::Long,

            Expression::Variable(name) => {
                if let Some(var_type) = self.variable_types.get(name) {
                    var_type.clone()
                } else {
                    self.errors.push(TypeCheckError::new(
                        format!("未声明的变量: '{}'", name)
                    ));
                    Type::Auto // 错误恢复
                }
            },

            Expression::BinaryOp(left, op, right) => {
                let left_type = self.infer_expression_type(left);
                let right_type = self.infer_expression_type(right);
                self.infer_binary_op_type(&left_type, op, &right_type)
            },

            Expression::CompareOp(left, _op, right) => {
                let left_type = self.infer_expression_type(left);
                let right_type = self.infer_expression_type(right);

                // 比较操作的两边应该是兼容类型
                if !self.types_compatible(&left_type, &right_type) {
                    self.errors.push(TypeCheckError::new(
                        format!("比较操作的类型不兼容: {:?} 和 {:?}", left_type, right_type)
                    ));
                }

                Type::Bool
            },

            Expression::LogicalOp(left, _op, right) => {
                let left_type = self.infer_expression_type(left);
                let right_type = self.infer_expression_type(right);

                // 逻辑操作的两边应该是bool类型
                if !matches!(left_type, Type::Bool) {
                    self.errors.push(TypeCheckError::new(
                        format!("逻辑操作的左操作数必须是bool类型，但得到 {:?}", left_type)
                    ));
                }
                if !matches!(right_type, Type::Bool) {
                    self.errors.push(TypeCheckError::new(
                        format!("逻辑操作的右操作数必须是bool类型，但得到 {:?}", right_type)
                    ));
                }

                Type::Bool
            },

            Expression::FunctionCall(name, args) => {
                self.check_function_call(name, args)
            },

            Expression::MethodCall(obj_expr, method_name, args) => {
                let obj_type = self.infer_expression_type(obj_expr);
                self.check_method_call(&obj_type, method_name, args)
            },

            Expression::FieldAccess(obj_expr, field_name) => {
                let obj_type = self.infer_expression_type(obj_expr);
                self.check_field_access(&obj_type, field_name)
            },

            Expression::ArrayAccess(array_expr, index_expr) => {
                let array_type = self.infer_expression_type(array_expr);
                let index_type = self.infer_expression_type(index_expr);

                // 索引必须是整数类型
                if !matches!(index_type, Type::Int | Type::Long) {
                    self.errors.push(TypeCheckError::new(
                        format!("数组索引必须是整数类型，但得到 {:?}", index_type)
                    ));
                }

                // 返回数组元素类型
                match array_type {
                    Type::Array(element_type) => *element_type,
                    _ => {
                        self.errors.push(TypeCheckError::new(
                            format!("尝试对非数组类型进行索引访问: {:?}", array_type)
                        ));
                        Type::Auto // 错误恢复
                    }
                }
            },

            Expression::AddressOf(expr) => {
                let target_type = self.infer_expression_type(expr);
                Type::Pointer(Box::new(target_type))
            },

            Expression::Dereference(expr) => {
                let ptr_type = self.infer_expression_type(expr);
                match ptr_type {
                    Type::Pointer(target_type) => *target_type,
                    Type::OptionalPointer(target_type) => *target_type,
                    _ => {
                        self.errors.push(TypeCheckError::new(
                            format!("尝试解引用非指针类型: {:?}", ptr_type)
                        ));
                        Type::Auto // 错误恢复
                    }
                }
            },

            Expression::PointerMemberAccess(ptr_expr, member_name) => {
                let ptr_type = self.infer_expression_type(ptr_expr);
                match ptr_type {
                    Type::Pointer(target_type) | Type::OptionalPointer(target_type) => {
                        self.check_field_access(&target_type, member_name)
                    },
                    _ => {
                        self.errors.push(TypeCheckError::new(
                            format!("尝试对非指针类型进行成员访问: {:?}", ptr_type)
                        ));
                        Type::Auto // 错误恢复
                    }
                }
            },

            Expression::ArrayLiteral(elements) => {
                if elements.is_empty() {
                    Type::Array(Box::new(Type::Auto))
                } else {
                    let first_type = self.infer_expression_type(&elements[0]);

                    // 检查所有元素类型是否一致
                    for (i, element) in elements.iter().enumerate().skip(1) {
                        let element_type = self.infer_expression_type(element);
                        if !self.types_compatible(&first_type, &element_type) {
                            self.errors.push(TypeCheckError::new(
                                format!("数组元素类型不一致: 第0个元素是 {:?}，第{}个元素是 {:?}",
                                        first_type, i, element_type)
                            ));
                        }
                    }

                    Type::Array(Box::new(first_type))
                }
            },

            Expression::TernaryOp(condition, true_expr, false_expr) => {
                let condition_type = self.infer_expression_type(condition);
                let true_type = self.infer_expression_type(true_expr);
                let false_type = self.infer_expression_type(false_expr);

                // 条件必须是bool类型
                if !matches!(condition_type, Type::Bool) {
                    self.errors.push(TypeCheckError::new(
                        format!("三元操作符的条件必须是bool类型，但得到 {:?}", condition_type)
                    ));
                }

                // 两个分支的类型应该兼容
                if self.types_compatible(&true_type, &false_type) {
                    true_type
                } else {
                    self.errors.push(TypeCheckError::new(
                        format!("三元操作符的两个分支类型不兼容: {:?} 和 {:?}", true_type, false_type)
                    ));
                    Type::Auto // 错误恢复
                }
            },

            _ => {
                // 其他表达式类型的处理
                Type::Auto
            }
        }
    }

    // 推断二元操作的结果类型
    fn infer_binary_op_type(&mut self, left_type: &Type, op: &crate::ast::BinaryOperator, right_type: &Type) -> Type {
        use crate::ast::BinaryOperator;

        match op {
            BinaryOperator::Add | BinaryOperator::Subtract | BinaryOperator::Multiply | BinaryOperator::Divide => {
                // 算术操作
                if self.types_compatible(left_type, right_type) {
                    match (left_type, right_type) {
                        (Type::Int, Type::Int) => Type::Int,
                        (Type::Float, _) | (_, Type::Float) => Type::Float,
                        (Type::Long, _) | (_, Type::Long) => Type::Long,
                        (Type::String, Type::String) if matches!(op, BinaryOperator::Add) => Type::String,
                        _ => {
                            self.errors.push(TypeCheckError::new(
                                format!("不支持的算术操作: {:?} {:?} {:?}", left_type, op, right_type)
                            ));
                            Type::Auto
                        }
                    }
                } else {
                    self.errors.push(TypeCheckError::new(
                        format!("算术操作的类型不兼容: {:?} 和 {:?}", left_type, right_type)
                    ));
                    Type::Auto
                }
            },
            BinaryOperator::Modulo => {
                // 模运算只支持整数类型
                if matches!(left_type, Type::Int | Type::Long) && matches!(right_type, Type::Int | Type::Long) {
                    if matches!(left_type, Type::Long) || matches!(right_type, Type::Long) {
                        Type::Long
                    } else {
                        Type::Int
                    }
                } else {
                    self.errors.push(TypeCheckError::new(
                        format!("模运算只支持整数类型，但得到 {:?} 和 {:?}", left_type, right_type)
                    ));
                    Type::Auto
                }
            }
        }
    }

    // 检查函数调用
    fn check_function_call(&mut self, name: &str, args: &[Expression]) -> Type {
        // 先克隆函数签名以避免借用冲突
        if let Some((param_types, return_type)) = self.function_signatures.get(name).cloned() {
            // 检查参数数量
            if args.len() != param_types.len() {
                self.errors.push(TypeCheckError::new(
                    format!("函数 '{}' 期望 {} 个参数，但提供了 {} 个",
                            name, param_types.len(), args.len())
                ));
                return return_type;
            }

            // 检查参数类型
            for (i, (arg_expr, expected_type)) in args.iter().zip(param_types.iter()).enumerate() {
                let arg_type = self.infer_expression_type(arg_expr);
                if !self.types_compatible(&expected_type, &arg_type) {
                    self.errors.push(TypeCheckError::new(
                        format!("函数 '{}' 的第 {} 个参数类型不匹配: 期望 {:?}，但得到 {:?}",
                                name, i + 1, expected_type, arg_type)
                    ));
                }
            }

            return_type
        } else {
            self.errors.push(TypeCheckError::new(
                format!("未声明的函数: '{}'", name)
            ));
            Type::Auto // 错误恢复
        }
    }

    // 检查方法调用
    fn check_method_call(&mut self, obj_type: &Type, method_name: &str, args: &[Expression]) -> Type {
        // 这里可以根据对象类型检查内置方法
        match obj_type {
            Type::String => {
                match method_name {
                    "length" => {
                        if !args.is_empty() {
                            self.errors.push(TypeCheckError::new(
                                format!("字符串的 length() 方法不接受参数")
                            ));
                        }
                        Type::Int
                    },
                    _ => {
                        self.errors.push(TypeCheckError::new(
                            format!("字符串类型没有方法 '{}'", method_name)
                        ));
                        Type::Auto
                    }
                }
            },
            Type::Array(_) => {
                match method_name {
                    "length" => {
                        if !args.is_empty() {
                            self.errors.push(TypeCheckError::new(
                                format!("数组的 length() 方法不接受参数")
                            ));
                        }
                        Type::Int
                    },
                    _ => {
                        self.errors.push(TypeCheckError::new(
                            format!("数组类型没有方法 '{}'", method_name)
                        ));
                        Type::Auto
                    }
                }
            },
            _ => {
                self.errors.push(TypeCheckError::new(
                    format!("类型 {:?} 没有方法 '{}'", obj_type, method_name)
                ));
                Type::Auto
            }
        }
    }

    // 检查字段访问
    fn check_field_access(&mut self, obj_type: &Type, field_name: &str) -> Type {
        match obj_type {
            Type::Class(class_name) => {
                // 先克隆类定义以避免借用冲突
                if let Some(class_fields) = self.class_definitions.get(class_name).cloned() {
                    if let Some(field_type) = class_fields.get(field_name) {
                        field_type.clone()
                    } else {
                        self.errors.push(TypeCheckError::new(
                            format!("类 '{}' 没有字段 '{}'", class_name, field_name)
                        ));
                        Type::Auto
                    }
                } else {
                    self.errors.push(TypeCheckError::new(
                        format!("未定义的类: '{}'", class_name)
                    ));
                    Type::Auto
                }
            },
            _ => {
                self.errors.push(TypeCheckError::new(
                    format!("类型 {:?} 不支持字段访问", obj_type)
                ));
                Type::Auto
            }
        }
    }

    // 检查类型兼容性
    fn types_compatible(&self, expected: &Type, actual: &Type) -> bool {
        match (expected, actual) {
            // 完全相同的类型
            (a, b) if a == b => true,

            // Auto类型与任何类型兼容
            (Type::Auto, _) | (_, Type::Auto) => true,

            // 数值类型的隐式转换
            (Type::Float, Type::Int) => true,
            (Type::Long, Type::Int) => true,

            // 指针类型兼容性
            (Type::Pointer(expected_target), Type::Pointer(actual_target)) => {
                self.types_compatible(expected_target, actual_target)
            },
            (Type::OptionalPointer(expected_target), Type::Pointer(actual_target)) => {
                self.types_compatible(expected_target, actual_target)
            },
            (Type::OptionalPointer(_), Type::Void) => true, // 可选指针可以为null

            // 数组类型兼容性
            (Type::Array(expected_element), Type::Array(actual_element)) => {
                self.types_compatible(expected_element, actual_element)
            },

            _ => false
        }
    }
}
