#[derive(Debug, Clone)]
pub enum Type {
    Int,
    Float,
    Bool,
    String,
    Long,
    Array(Box<Type>),
    Map(Box<Type>, Box<Type>),
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
    FunctionCall(String, Vec<Expression>),
    NamespacedFunctionCall(Vec<String>, Vec<Expression>), // 命名空间函数调用
    GlobalFunctionCall(String, Vec<Expression>), // 全局函数明确调用 (::func)
    Variable(String),
    BinaryOp(Box<Expression>, BinaryOperator, Box<Expression>),
    // 未来可以扩展更多表达式类型
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
pub enum Statement {
    Return(Expression),
    VariableDeclaration(String, Type, Expression),
    VariableAssignment(String, Expression),
    Increment(String), // 自增语句 (var++)
    Decrement(String), // 自减语句 (var--)
    CompoundAssignment(String, BinaryOperator, Expression), // 复合赋值 (+=, -=, *=, /=, %=)
    UsingNamespace(Vec<String>), // 导入命名空间 (using ns xxx;)
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
    pub functions: Vec<Function>,
    pub namespaces: Vec<Namespace>, // 嵌套命名空间
}

#[derive(Debug, Clone)]
pub struct Program {
    pub functions: Vec<Function>,
    pub namespaces: Vec<Namespace>, // 顶层命名空间
} 