#[derive(Debug, Clone)]
pub enum Type {
    Int,
    Float,
    Bool,
    String,
    Long,
    Void,    // 添加void类型
    Array(Box<Type>),
    Map(Box<Type>, Box<Type>),
    Exception, // 新增：异常类型
    // 未来可以扩展更多类型
}

#[derive(Debug, Clone)]
pub enum Expression {
    IntLiteral(i32),
    FloatLiteral(f64),
    BoolLiteral(bool),
    StringLiteral(String),
    LongLiteral(i64),
    ArrayLiteral(Vec<Expression>),
    MapLiteral(Vec<(Expression, Expression)>),
    IndexAccess(Box<Expression>, Box<Expression>),
    MemberAccess(Box<Expression>, String), // object.member
    FunctionCall(String, Vec<Expression>),
    NamespacedFunctionCall(Vec<String>, Vec<Expression>), // 命名空间函数调用
    GlobalFunctionCall(String, Vec<Expression>), // 全局函数明确调用 (::func)
    LibraryFunctionCall(String, String, Vec<Expression>), // 库函数调用 (lib::func)
    Variable(String),
    BinaryOp(Box<Expression>, BinaryOperator, Box<Expression>),
    CompareOp(Box<Expression>, CompareOperator, Box<Expression>), // 比较操作
    LogicalOp(Box<Expression>, LogicalOperator, Box<Expression>), // 逻辑操作
    PreIncrement(String),  // 前置自增 (++var)
    PreDecrement(String),  // 前置自减 (--var)
    PostIncrement(String), // 后置自增 (var++)
    PostDecrement(String), // 后置自减 (var--)
    TernaryOp(Box<Expression>, Box<Expression>, Box<Expression>), // 三元条件运算符 (cond ? expr1 : expr2)
    Throw(Box<Expression>), // 新增：抛出异常
    // 链式调用相关
    MethodCall(Box<Expression>, String, Vec<Expression>), // 方法调用 (obj.method(args))
    ChainCall(Box<Expression>, Vec<(String, Vec<Expression>)>), // 链式调用 (obj.method1().method2())
    // 未来可以扩展更多表达式类型
}

// 命名空间类型
#[derive(Debug, Clone, PartialEq)]
pub enum NamespaceType {
    Code,    // 代码命名空间 (ns xxx)
    Library, // 库命名空间 (lib xxx)
}

#[derive(Debug, Clone)]
pub enum BinaryOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
}

#[derive(Debug, Clone)]
pub enum CompareOperator {
    Equal,        // ==
    NotEqual,     // !=
    Greater,      // >
    Less,         // <
    GreaterEqual, // >=
    LessEqual,    // <=
}

#[derive(Debug, Clone)]
pub enum LogicalOperator {
    And,  // &&
    Or,   // ||
    Not,  // !
}

#[derive(Debug, Clone)]
pub enum Statement {
    Return(Expression),
    VariableDeclaration(String, Type, Expression),
    ConstantDeclaration(String, Type, Expression), // 新增：常量声明
    VariableAssignment(String, Expression),
    Increment(String), // 后置自增语句 (var++)
    Decrement(String), // 后置自减语句 (var--)
    PreIncrement(String), // 前置自增语句 (++var)
    PreDecrement(String), // 前置自减语句 (--var)
    CompoundAssignment(String, BinaryOperator, Expression), // 复合赋值 (+=, -=, *=, /=, %=)
    ImportNamespace(NamespaceType, Vec<String>), // 统一的命名空间导入，第一个参数表示类型，第二个参数是路径
    FileImport(String),    // 导入文件 (using file "xxx.cn";)
    FunctionCallStatement(Expression), // 函数调用语句
    NamespacedFunctionCallStatement(Vec<String>, Vec<Expression>), // 命名空间函数调用语句 (ns::func())
    LibraryFunctionCallStatement(String, String, Vec<Expression>), // 库函数调用语句 (lib::func())
    IfElse(Expression, Vec<Statement>, Vec<(Option<Expression>, Vec<Statement>)>), // if-else 语句，包含条件、if块和多个else-if/else块
    ForLoop(String, Expression, Expression, Vec<Statement>), // for循环，包含变量名、范围起始值、范围结束值和循环体
    WhileLoop(Expression, Vec<Statement>), // while循环，包含条件和循环体
    Break, // 跳出当前循环
    Continue, // 跳过当前迭代，继续下一次迭代
    ForEachLoop(String, Expression, Vec<Statement>), // foreach循环，包含变量名、集合表达式和循环体
    TryCatch(Vec<Statement>, Vec<(String, Type, Vec<Statement>)>, Option<Vec<Statement>>), // 新增：try-catch-finally 语句
    Throw(Expression), // 新增：抛出异常语句
    // 未来可以扩展更多语句类型
}

#[derive(Debug, Clone)]
pub struct Parameter {
    pub name: String,
    pub param_type: Type,
}

#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub parameters: Vec<Parameter>,
    pub return_type: Type,
    pub body: Vec<Statement>,
}

#[derive(Debug, Clone)]
pub struct Namespace {
    pub name: String,
    pub ns_type: NamespaceType, // 添加命名空间类型字段
    pub functions: Vec<Function>,
    pub namespaces: Vec<Namespace>, // 嵌套命名空间
}

#[derive(Debug, Clone)]
pub struct Program {
    pub functions: Vec<Function>,
    pub namespaces: Vec<Namespace>, // 顶层命名空间
    pub imported_namespaces: Vec<(NamespaceType, Vec<String>)>, // 统一的导入记录
    pub file_imports: Vec<String>,   // 顶层文件导入
    pub constants: Vec<(String, Type, Expression)>, // 新增：顶层常量定义
} 