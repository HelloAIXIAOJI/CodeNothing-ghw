pub mod value;
pub mod evaluator;
pub mod executor;
pub mod library_loader;
pub mod interpreter_core;
pub mod function_calls;
pub mod expression_evaluator;
pub mod statement_executor;
pub mod jit;
pub mod handlers;
pub mod memory_manager;
pub mod pattern_matcher;

// Re-export main types and functions
pub use interpreter_core::{interpret, Interpreter, debug_println};
pub use function_calls::FunctionCallHandler;
pub use expression_evaluator::ExpressionEvaluator;
pub use statement_executor::StatementExecutor;
pub use value::Value;
pub use evaluator::{Evaluator, perform_binary_operation, evaluate_compare_operation};
pub use executor::{Executor, ExecutionResult, update_variable_value, handle_increment, handle_decrement, execute_if_else};
pub use library_loader::{load_library, call_library_function, convert_values_to_string_args, convert_value_to_string_arg}; 
pub use jit::{jit_eval_const_expr, should_compile_array_operation, compile_array_operation};
pub use pattern_matcher::PatternMatcher;