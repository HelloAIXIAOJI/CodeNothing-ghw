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
pub struct Program {
    pub functions: Vec<Function>,
} 