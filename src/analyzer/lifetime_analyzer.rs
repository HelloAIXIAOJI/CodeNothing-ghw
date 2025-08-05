// CodeNothing v0.7.4 变量生命周期分析器
// 实现编译时生命周期分析，优化运行时性能

use std::collections::{HashMap, HashSet};
use crate::ast::{Statement, Expression, Function, Program, Type};

/// 变量作用域信息
#[derive(Debug, Clone)]
pub struct VariableScope {
    pub scope_id: usize,
    pub parent_scope: Option<usize>,
    pub variables: HashMap<String, VariableInfo>,
    pub start_line: usize,
    pub end_line: usize,
}

/// 变量信息
#[derive(Debug, Clone)]
pub struct VariableInfo {
    pub name: String,
    pub var_type: Option<Type>,
    pub declared_line: usize,
    pub last_used_line: usize,
    pub is_safe: bool,           // 编译时确定是否安全
    pub usage_pattern: UsagePattern,
}

/// 变量使用模式
#[derive(Debug, Clone, PartialEq)]
pub enum UsagePattern {
    SingleAssignment,    // 单次赋值，只读
    LocalOnly,          // 仅在局部作用域使用
    CrossScope,         // 跨作用域使用
    LoopVariable,       // 循环变量
    FunctionParameter,  // 函数参数
}

/// 生命周期分析结果
#[derive(Debug, Clone)]
pub struct LifetimeAnalysisResult {
    pub safe_variables: HashSet<String>,
    pub optimization_opportunities: Vec<OptimizationOpportunity>,
    pub estimated_performance_gain: f32,
}

/// 优化机会
#[derive(Debug, Clone)]
pub struct OptimizationOpportunity {
    pub variable_name: String,
    pub optimization_type: OptimizationType,
    pub estimated_savings: f32, // 预估节省的时间（毫秒）
}

/// 优化类型
#[derive(Debug, Clone)]
pub enum OptimizationType {
    SkipBoundsCheck,     // 跳过边界检查
    SkipNullCheck,       // 跳过空指针检查
    SkipTypeCheck,       // 跳过类型检查
    InlineAccess,        // 内联访问
}

/// 变量生命周期分析器
pub struct VariableLifetimeAnalyzer {
    pub scopes: Vec<VariableScope>,
    pub safe_variables: HashSet<String>,
    pub current_scope_id: usize,
    pub current_line: usize,
    pub analysis_result: Option<LifetimeAnalysisResult>,
}

impl VariableLifetimeAnalyzer {
    /// 创建新的生命周期分析器
    pub fn new() -> Self {
        Self {
            scopes: Vec::new(),
            safe_variables: HashSet::new(),
            current_scope_id: 0,
            current_line: 0,
            analysis_result: None,
        }
    }

    /// 分析整个程序的变量生命周期
    pub fn analyze_program(&mut self, program: &Program) -> LifetimeAnalysisResult {
        crate::lifetime_debug_println!("开始变量生命周期分析...");

        // 创建全局作用域
        self.create_scope(None, 0, usize::MAX);

        // 分析所有函数
        for function in &program.functions {
            self.analyze_function(function);
        }

        // 分析主程序常量
        for (const_name, const_type, _const_expr) in &program.constants {
            self.declare_variable(const_name, Some(const_type.clone()), UsagePattern::SingleAssignment);
        }

        // 生成分析结果
        let result = self.generate_analysis_result();
        self.analysis_result = Some(result.clone());

        crate::lifetime_debug_println!("生命周期分析完成，发现 {} 个安全变量", self.safe_variables.len());
        result
    }

    /// 创建新的作用域
    fn create_scope(&mut self, parent: Option<usize>, start_line: usize, end_line: usize) -> usize {
        let scope_id = self.scopes.len();
        let scope = VariableScope {
            scope_id,
            parent_scope: parent,
            variables: HashMap::new(),
            start_line,
            end_line,
        };
        self.scopes.push(scope);
        scope_id
    }

    /// 分析函数
    fn analyze_function(&mut self, function: &Function) {
        let function_scope = self.create_scope(Some(self.current_scope_id), 0, usize::MAX);
        let old_scope = self.current_scope_id;
        self.current_scope_id = function_scope;

        // 分析函数参数
        for param in &function.parameters {
            self.declare_variable(&param.name, Some(param.param_type.clone()), UsagePattern::FunctionParameter);
        }

        // 分析函数体
        for statement in &function.body {
            self.analyze_statement(statement);
        }

        self.current_scope_id = old_scope;
    }

    /// 分析语句
    fn analyze_statement(&mut self, statement: &Statement) {
        self.current_line += 1;

        match statement {
            Statement::VariableDeclaration(name, var_type, init_expr) => {
                self.declare_variable(name, Some(var_type.clone()), UsagePattern::LocalOnly);
                self.analyze_expression(init_expr);
            },
            Statement::VariableAssignment(name, expr) => {
                self.use_variable(name);
                self.analyze_expression(expr);
            },
            Statement::IfElse(condition, then_block, else_blocks) => {
                self.analyze_expression(condition);

                let if_scope = self.create_scope(Some(self.current_scope_id), self.current_line, self.current_line + 100);
                let old_scope = self.current_scope_id;
                self.current_scope_id = if_scope;

                for stmt in then_block {
                    self.analyze_statement(stmt);
                }

                for (condition_opt, else_stmts) in else_blocks {
                    if let Some(cond) = condition_opt {
                        self.analyze_expression(cond);
                    }
                    for stmt in else_stmts {
                        self.analyze_statement(stmt);
                    }
                }

                self.current_scope_id = old_scope;
            },
            Statement::WhileLoop(condition, body) => {
                self.analyze_expression(condition);

                let loop_scope = self.create_scope(Some(self.current_scope_id), self.current_line, self.current_line + 100);
                let old_scope = self.current_scope_id;
                self.current_scope_id = loop_scope;

                for stmt in body {
                    self.analyze_statement(stmt);
                }

                self.current_scope_id = old_scope;
            },
            Statement::ForLoop(var_name, start_expr, end_expr, body) => {
                self.analyze_expression(start_expr);
                self.analyze_expression(end_expr);

                let loop_scope = self.create_scope(Some(self.current_scope_id), self.current_line, self.current_line + 100);
                let old_scope = self.current_scope_id;
                self.current_scope_id = loop_scope;

                // 循环变量
                self.declare_variable(var_name, Some(Type::Int), UsagePattern::LoopVariable);

                for stmt in body {
                    self.analyze_statement(stmt);
                }

                self.current_scope_id = old_scope;
            },
            Statement::FunctionCallStatement(expr) => {
                self.analyze_expression(expr);
            },
            Statement::Return(expr) => {
                if let Some(e) = expr {
                    self.analyze_expression(e);
                }
            },
            _ => {
                // 其他语句类型的处理
            }
        }
    }

    /// 分析表达式
    fn analyze_expression(&mut self, expression: &Expression) {
        match expression {
            Expression::Variable(name) => {
                self.use_variable(name);
            },
            Expression::BinaryOp(left, _op, right) => {
                self.analyze_expression(left);
                self.analyze_expression(right);
            },
            Expression::FunctionCall(name, args) => {
                for arg in args {
                    self.analyze_expression(arg);
                }
            },
            Expression::ArrayAccess(array_expr, index_expr) => {
                self.analyze_expression(array_expr);
                self.analyze_expression(index_expr);
            },
            Expression::FieldAccess(obj_expr, _field) => {
                self.analyze_expression(obj_expr);
            },
            _ => {
                // 其他表达式类型
            }
        }
    }

    /// 声明变量
    fn declare_variable(&mut self, name: &str, var_type: Option<Type>, usage_pattern: UsagePattern) {
        let is_safe = self.is_variable_safe(&usage_pattern);

        let var_info = VariableInfo {
            name: name.to_string(),
            var_type,
            declared_line: self.current_line,
            last_used_line: self.current_line,
            is_safe,
            usage_pattern,
        };

        if let Some(scope) = self.scopes.get_mut(self.current_scope_id) {
            scope.variables.insert(name.to_string(), var_info);
        }

        // 如果变量是安全的，添加到安全变量集合
        if is_safe {
            self.safe_variables.insert(name.to_string());
        }
    }

    /// 使用变量
    fn use_variable(&mut self, name: &str) {
        // 在当前作用域及父作用域中查找变量
        let mut scope_id = Some(self.current_scope_id);
        
        while let Some(sid) = scope_id {
            if let Some(scope) = self.scopes.get_mut(sid) {
                if let Some(var_info) = scope.variables.get_mut(name) {
                    var_info.last_used_line = self.current_line;
                    return;
                }
                scope_id = scope.parent_scope;
            } else {
                break;
            }
        }
    }

    /// 判断变量是否安全（可以跳过运行时检查）
    fn is_variable_safe(&self, usage_pattern: &UsagePattern) -> bool {
        match usage_pattern {
            UsagePattern::SingleAssignment => true,
            UsagePattern::LocalOnly => true,
            UsagePattern::FunctionParameter => true,
            UsagePattern::LoopVariable => false, // 循环变量可能有边界问题
            UsagePattern::CrossScope => false,   // 跨作用域使用需要检查
        }
    }

    /// 生成分析结果
    fn generate_analysis_result(&self) -> LifetimeAnalysisResult {
        let mut optimization_opportunities = Vec::new();
        let mut total_estimated_gain = 0.0;

        for var_name in &self.safe_variables {
            // 为每个安全变量生成优化机会
            optimization_opportunities.push(OptimizationOpportunity {
                variable_name: var_name.clone(),
                optimization_type: OptimizationType::SkipBoundsCheck,
                estimated_savings: 0.1, // 每次访问节省0.1ms
            });

            optimization_opportunities.push(OptimizationOpportunity {
                variable_name: var_name.clone(),
                optimization_type: OptimizationType::SkipNullCheck,
                estimated_savings: 0.05, // 每次访问节省0.05ms
            });

            total_estimated_gain += 0.15;
        }

        LifetimeAnalysisResult {
            safe_variables: self.safe_variables.clone(),
            optimization_opportunities,
            estimated_performance_gain: total_estimated_gain,
        }
    }

    /// 获取分析结果
    pub fn get_analysis_result(&self) -> Option<&LifetimeAnalysisResult> {
        self.analysis_result.as_ref()
    }

    /// 检查变量是否安全
    pub fn is_variable_runtime_safe(&self, var_name: &str) -> bool {
        self.safe_variables.contains(var_name)
    }
}
