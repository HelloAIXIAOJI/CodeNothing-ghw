// CodeNothing 静态分析模块

pub mod type_checker;
pub mod lifetime_analyzer;

pub use type_checker::{TypeChecker, TypeCheckError};
pub use lifetime_analyzer::{VariableLifetimeAnalyzer, LifetimeAnalysisResult, VariableScope, VariableInfo, OptimizationOpportunity};
