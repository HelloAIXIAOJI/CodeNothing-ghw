// 模式匹配解释器
use crate::ast::{Pattern, MatchArm, Expression, Statement};
use crate::interpreter::{Interpreter, Value, ExecutionResult};
use crate::interpreter::pattern_jit::{should_use_pattern_jit, jit_match_pattern};
use crate::interpreter::expression_evaluator::ExpressionEvaluator;
use crate::interpreter::statement_executor::StatementExecutor;
use std::collections::HashMap;

/// 模式匹配结果
#[derive(Debug, Clone)]
pub struct MatchResult {
    pub matched: bool,
    pub bindings: HashMap<String, Value>,
}

impl MatchResult {
    pub fn new_matched(bindings: HashMap<String, Value>) -> Self {
        MatchResult {
            matched: true,
            bindings,
        }
    }
    
    pub fn new_unmatched() -> Self {
        MatchResult {
            matched: false,
            bindings: HashMap::new(),
        }
    }
}

pub trait PatternMatcher {
    fn execute_match_statement(&mut self, match_expr: Expression, arms: Vec<MatchArm>) -> ExecutionResult;
    fn evaluate_match_expression(&mut self, match_expr: Expression, arms: Vec<MatchArm>) -> Value;
    fn match_pattern(&mut self, pattern: &Pattern, value: &Value) -> MatchResult;
    fn evaluate_guard(&mut self, guard: &Expression, bindings: &HashMap<String, Value>) -> bool;
    fn execute_match_arm_body(&mut self, body: &[Statement], bindings: &HashMap<String, Value>) -> ExecutionResult;
    fn match_pattern_interpreted(&mut self, pattern: &Pattern, value: &Value) -> MatchResult;
    fn match_pattern_with_bindings(&mut self, pattern: &Pattern, value: &Value) -> MatchResult;
}

impl<'a> PatternMatcher for Interpreter<'a> {
    /// 执行match语句
    fn execute_match_statement(&mut self, match_expr: Expression, arms: Vec<MatchArm>) -> ExecutionResult {
        // 计算匹配表达式的值
        let match_value = self.evaluate_expression(&match_expr);
        
        // 遍历所有匹配分支
        for arm in &arms {
            // 尝试匹配模式
            let match_result = self.match_pattern(&arm.pattern, &match_value);
            
            if match_result.matched {
                // 检查守卫条件（如果有）
                if let Some(ref guard) = arm.guard {
                    if !self.evaluate_guard(guard, &match_result.bindings) {
                        continue; // 守卫条件不满足，继续下一个分支
                    }
                }
                
                // 执行匹配分支的代码
                return self.execute_match_arm_body(&arm.body, &match_result.bindings);
            }
        }
        
        // 没有匹配的分支，这是一个运行时错误
        panic!("match表达式没有匹配的分支");
    }
    
    /// 计算match表达式的值
    fn evaluate_match_expression(&mut self, match_expr: Expression, arms: Vec<MatchArm>) -> Value {
        // 计算匹配表达式的值
        let match_value = self.evaluate_expression(&match_expr);
        
        // 遍历所有匹配分支
        for arm in &arms {
            // 尝试匹配模式
            let match_result = self.match_pattern(&arm.pattern, &match_value);
            
            if match_result.matched {
                // 检查守卫条件（如果有）
                if let Some(ref guard) = arm.guard {
                    if !self.evaluate_guard(guard, &match_result.bindings) {
                        continue; // 守卫条件不满足，继续下一个分支
                    }
                }
                
                // 保存当前环境
                let saved_env = self.local_env.clone();
                
                // 应用模式绑定
                for (name, value) in match_result.bindings {
                    self.local_env.insert(name, value);
                }
                
                // 执行匹配分支的代码，获取最后一个表达式的值
                let mut result_value = Value::None;
                for stmt in &arm.body {
                    match self.execute_statement_direct(stmt.clone()) {
                        ExecutionResult::None => {},
                        ExecutionResult::Return(value) => {
                            result_value = value;
                            break;
                        },
                        _ => break,
                    }
                }
                
                // 恢复环境
                self.local_env = saved_env;
                
                return result_value;
            }
        }
        
        // 没有匹配的分支，这是一个运行时错误
        panic!("match表达式没有匹配的分支");
    }
    
    /// 匹配模式（支持JIT优化）
    fn match_pattern(&mut self, pattern: &Pattern, value: &Value) -> MatchResult {
        use std::sync::Mutex;
        use std::collections::HashMap;

        // 使用lazy_static或者简单的全局计数器
        static PATTERN_USAGE_COUNTER: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

        let usage_count = PATTERN_USAGE_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        // 如果使用次数足够多，尝试JIT编译
        if should_use_pattern_jit(pattern, usage_count) {
            match jit_match_pattern(pattern, value, self) {
                Ok(matched) => {
                    if matched {
                        // JIT匹配成功，但我们仍需要处理变量绑定
                        return self.match_pattern_with_bindings(pattern, value);
                    } else {
                        return MatchResult::new_unmatched();
                    }
                },
                Err(_) => {
                    // JIT失败，回退到解释执行
                }
            }
        }

        // 解释执行模式匹配
        self.match_pattern_interpreted(pattern, value)
    }

    /// 解释执行模式匹配
    fn match_pattern_interpreted(&mut self, pattern: &Pattern, value: &Value) -> MatchResult {
        match pattern {
            // 字面量模式
            Pattern::IntLiteral(expected) => {
                if let Value::Int(actual) = value {
                    if *expected == *actual {
                        MatchResult::new_matched(HashMap::new())
                    } else {
                        MatchResult::new_unmatched()
                    }
                } else {
                    MatchResult::new_unmatched()
                }
            },
            
            Pattern::FloatLiteral(expected) => {
                if let Value::Float(actual) = value {
                    if (*expected - *actual).abs() < f64::EPSILON {
                        MatchResult::new_matched(HashMap::new())
                    } else {
                        MatchResult::new_unmatched()
                    }
                } else {
                    MatchResult::new_unmatched()
                }
            },
            
            Pattern::BoolLiteral(expected) => {
                if let Value::Bool(actual) = value {
                    if *expected == *actual {
                        MatchResult::new_matched(HashMap::new())
                    } else {
                        MatchResult::new_unmatched()
                    }
                } else {
                    MatchResult::new_unmatched()
                }
            },
            
            Pattern::StringLiteral(expected) => {
                if let Value::String(actual) = value {
                    if expected == actual {
                        MatchResult::new_matched(HashMap::new())
                    } else {
                        MatchResult::new_unmatched()
                    }
                } else {
                    MatchResult::new_unmatched()
                }
            },
            
            // 变量模式 - 总是匹配并绑定值
            Pattern::Variable(name) => {
                let mut bindings = HashMap::new();
                bindings.insert(name.clone(), value.clone());
                MatchResult::new_matched(bindings)
            },
            
            // 通配符模式 - 总是匹配但不绑定
            Pattern::Wildcard => {
                MatchResult::new_matched(HashMap::new())
            },
            
            // 元组模式
            Pattern::Tuple(patterns) => {
                if let Value::Array(values) = value {
                    if patterns.len() == values.len() {
                        let mut all_bindings = HashMap::new();
                        
                        for (pattern, value) in patterns.iter().zip(values.iter()) {
                            let result = self.match_pattern(pattern, value);
                            if !result.matched {
                                return MatchResult::new_unmatched();
                            }
                            all_bindings.extend(result.bindings);
                        }
                        
                        MatchResult::new_matched(all_bindings)
                    } else {
                        MatchResult::new_unmatched()
                    }
                } else {
                    MatchResult::new_unmatched()
                }
            },
            
            // 数组模式
            Pattern::Array(patterns) => {
                if let Value::Array(values) = value {
                    if patterns.len() == values.len() {
                        let mut all_bindings = HashMap::new();
                        
                        for (pattern, value) in patterns.iter().zip(values.iter()) {
                            let result = self.match_pattern(pattern, value);
                            if !result.matched {
                                return MatchResult::new_unmatched();
                            }
                            all_bindings.extend(result.bindings);
                        }
                        
                        MatchResult::new_matched(all_bindings)
                    } else {
                        MatchResult::new_unmatched()
                    }
                } else {
                    MatchResult::new_unmatched()
                }
            },
            
            // 或模式 - 任意一个子模式匹配即可
            Pattern::Or(patterns) => {
                for pattern in patterns {
                    let result = self.match_pattern(pattern, value);
                    if result.matched {
                        return result;
                    }
                }
                MatchResult::new_unmatched()
            },
            
            // 其他模式暂时不实现
            _ => {
                println!("警告: 模式类型 {:?} 尚未实现", pattern);
                MatchResult::new_unmatched()
            }
        }
    }
    
    /// 计算守卫条件
    fn evaluate_guard(&mut self, guard: &Expression, bindings: &HashMap<String, Value>) -> bool {
        // 保存当前环境
        let saved_env = self.local_env.clone();
        
        // 应用模式绑定
        for (name, value) in bindings {
            self.local_env.insert(name.clone(), value.clone());
        }
        
        // 计算守卫表达式
        let guard_value = self.evaluate_expression(guard);
        
        // 恢复环境
        self.local_env = saved_env;
        
        // 返回布尔结果
        match guard_value {
            Value::Bool(b) => b,
            _ => false, // 非布尔值视为false
        }
    }
    
    /// 执行匹配分支的代码
    fn execute_match_arm_body(&mut self, body: &[Statement], bindings: &HashMap<String, Value>) -> ExecutionResult {
        // 保存当前环境
        let saved_env = self.local_env.clone();
        
        // 应用模式绑定
        for (name, value) in bindings {
            self.local_env.insert(name.clone(), value.clone());
        }
        
        // 执行语句块
        let mut result = ExecutionResult::None;
        for stmt in body {
            result = self.execute_statement_direct(stmt.clone());
            match result {
                ExecutionResult::None => {},
                _ => break, // 遇到return、break、continue等，立即返回
            }
        }
        
        // 恢复环境
        self.local_env = saved_env;
        
        result
    }

    /// 带变量绑定的模式匹配（用于JIT后的绑定处理）
    fn match_pattern_with_bindings(&mut self, pattern: &Pattern, value: &Value) -> MatchResult {
        match pattern {
            Pattern::Variable(name) => {
                let mut bindings = HashMap::new();
                bindings.insert(name.clone(), value.clone());
                MatchResult::new_matched(bindings)
            },
            Pattern::Wildcard => {
                MatchResult::new_matched(HashMap::new())
            },
            Pattern::Tuple(patterns) => {
                if let Value::Array(values) = value {
                    if patterns.len() == values.len() {
                        let mut all_bindings = HashMap::new();

                        for (pattern, value) in patterns.iter().zip(values.iter()) {
                            let result = self.match_pattern_with_bindings(pattern, value);
                            if !result.matched {
                                return MatchResult::new_unmatched();
                            }
                            all_bindings.extend(result.bindings);
                        }

                        MatchResult::new_matched(all_bindings)
                    } else {
                        MatchResult::new_unmatched()
                    }
                } else {
                    MatchResult::new_unmatched()
                }
            },
            Pattern::Or(patterns) => {
                for pattern in patterns {
                    let result = self.match_pattern_with_bindings(pattern, value);
                    if result.matched {
                        return result;
                    }
                }
                MatchResult::new_unmatched()
            },
            _ => {
                // 对于字面量模式，不需要绑定
                MatchResult::new_matched(HashMap::new())
            }
        }
    }
}
