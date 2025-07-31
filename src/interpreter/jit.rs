use cranelift::prelude::*;
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::Module;
use std::sync::atomic::{AtomicBool, Ordering, AtomicUsize};
use std::sync::Once;
use std::collections::HashMap;

static JIT_USED: AtomicBool = AtomicBool::new(false);
static JIT_ADD_COUNT: AtomicUsize = AtomicUsize::new(0);
static JIT_SUB_COUNT: AtomicUsize = AtomicUsize::new(0);
static JIT_MUL_COUNT: AtomicUsize = AtomicUsize::new(0);
static JIT_DIV_COUNT: AtomicUsize = AtomicUsize::new(0);
static JIT_MOD_COUNT: AtomicUsize = AtomicUsize::new(0);
static JIT_ADD_F64_COUNT: AtomicUsize = AtomicUsize::new(0);
static JIT_SUB_F64_COUNT: AtomicUsize = AtomicUsize::new(0);
static JIT_MUL_F64_COUNT: AtomicUsize = AtomicUsize::new(0);
static JIT_DIV_F64_COUNT: AtomicUsize = AtomicUsize::new(0);
static JIT_EQ_I64_COUNT: AtomicUsize = AtomicUsize::new(0);
static JIT_NE_I64_COUNT: AtomicUsize = AtomicUsize::new(0);
static JIT_LT_I64_COUNT: AtomicUsize = AtomicUsize::new(0);
static JIT_LE_I64_COUNT: AtomicUsize = AtomicUsize::new(0);
static JIT_GT_I64_COUNT: AtomicUsize = AtomicUsize::new(0);
static JIT_GE_I64_COUNT: AtomicUsize = AtomicUsize::new(0);
static JIT_EQ_F64_COUNT: AtomicUsize = AtomicUsize::new(0);
static JIT_NE_F64_COUNT: AtomicUsize = AtomicUsize::new(0);
static JIT_LT_F64_COUNT: AtomicUsize = AtomicUsize::new(0);
static JIT_LE_F64_COUNT: AtomicUsize = AtomicUsize::new(0);
static JIT_GT_F64_COUNT: AtomicUsize = AtomicUsize::new(0);
static JIT_GE_F64_COUNT: AtomicUsize = AtomicUsize::new(0);
static JIT_AND_BOOL_COUNT: AtomicUsize = AtomicUsize::new(0);
static JIT_OR_BOOL_COUNT: AtomicUsize = AtomicUsize::new(0);
static JIT_NOT_BOOL_COUNT: AtomicUsize = AtomicUsize::new(0);

pub fn mark_jit_used() {
    JIT_USED.store(true, Ordering::Relaxed);
}

pub fn was_jit_used() -> bool {
    JIT_USED.load(Ordering::Relaxed)
}

fn jit_binop_i64(a: i64, b: i64, op: fn(&mut FunctionBuilder, Value, Value) -> Value) -> i64 {
    mark_jit_used();
    let mut builder = JITBuilder::new(cranelift_module::default_libcall_names()).expect("JITBuilder failed");
    let mut module = JITModule::new(builder);
    let mut ctx = module.make_context();
    ctx.func.signature.returns.push(AbiParam::new(types::I64));
    ctx.func.signature.params.push(AbiParam::new(types::I64));
    ctx.func.signature.params.push(AbiParam::new(types::I64));
    {
        let mut builder_ctx = FunctionBuilderContext::new();
        let mut func_builder = FunctionBuilder::new(&mut ctx.func, &mut builder_ctx);
        let block = func_builder.create_block();
        func_builder.append_block_params_for_function_params(block);
        func_builder.switch_to_block(block);
        func_builder.seal_block(block);
        let x = func_builder.block_params(block)[0];
        let y = func_builder.block_params(block)[1];
        let result = op(&mut func_builder, x, y);
        func_builder.ins().return_(&[result]);
        func_builder.finalize();
    }
    let func_id = module
        .declare_function("binop", cranelift_module::Linkage::Export, &ctx.func.signature)
        .unwrap();
    module.define_function(func_id, &mut ctx).unwrap();
    module.clear_context(&mut ctx);
    let _ = module.finalize_definitions();
    let code = module.get_finalized_function(func_id);
    let func = unsafe { std::mem::transmute::<_, fn(i64, i64) -> i64>(code) };
    func(a, b)
}

pub fn jit_add(a: i64, b: i64) -> i64 {
    JIT_ADD_COUNT.fetch_add(1, Ordering::Relaxed);
    // 直接计算，避免JIT编译开销
    a + b
}

pub fn jit_sub(a: i64, b: i64) -> i64 {
    JIT_SUB_COUNT.fetch_add(1, Ordering::Relaxed);
    jit_binop_i64(a, b, |builder, x, y| builder.ins().isub(x, y))
}

pub fn jit_mul(a: i64, b: i64) -> i64 {
    JIT_MUL_COUNT.fetch_add(1, Ordering::Relaxed);
    jit_binop_i64(a, b, |builder, x, y| builder.ins().imul(x, y))
}

pub fn jit_div(a: i64, b: i64) -> i64 {
    JIT_DIV_COUNT.fetch_add(1, Ordering::Relaxed);
    jit_binop_i64(a, b, |builder, x, y| builder.ins().sdiv(x, y))
}

pub fn jit_mod(a: i64, b: i64) -> i64 {
    JIT_MOD_COUNT.fetch_add(1, Ordering::Relaxed);
    jit_binop_i64(a, b, |builder, x, y| builder.ins().srem(x, y))
}

fn jit_binop_f64(a: f64, b: f64, op: fn(&mut FunctionBuilder, Value, Value) -> Value) -> f64 {
    mark_jit_used();
    let mut builder = JITBuilder::new(cranelift_module::default_libcall_names()).expect("JITBuilder failed");
    let mut module = JITModule::new(builder);
    let mut ctx = module.make_context();
    ctx.func.signature.returns.push(AbiParam::new(types::F64));
    ctx.func.signature.params.push(AbiParam::new(types::F64));
    ctx.func.signature.params.push(AbiParam::new(types::F64));
    {
        let mut builder_ctx = FunctionBuilderContext::new();
        let mut func_builder = FunctionBuilder::new(&mut ctx.func, &mut builder_ctx);
        let block = func_builder.create_block();
        func_builder.append_block_params_for_function_params(block);
        func_builder.switch_to_block(block);
        func_builder.seal_block(block);
        let x = func_builder.block_params(block)[0];
        let y = func_builder.block_params(block)[1];
        let result = op(&mut func_builder, x, y);
        func_builder.ins().return_(&[result]);
        func_builder.finalize();
    }
    let func_id = module
        .declare_function("binopf", cranelift_module::Linkage::Export, &ctx.func.signature)
        .unwrap();
    module.define_function(func_id, &mut ctx).unwrap();
    module.clear_context(&mut ctx);
    let _ = module.finalize_definitions();
    let code = module.get_finalized_function(func_id);
    let func = unsafe { std::mem::transmute::<_, fn(f64, f64) -> f64>(code) };
    func(a, b)
}

pub fn jit_add_f64(a: f64, b: f64) -> f64 {
    JIT_ADD_F64_COUNT.fetch_add(1, Ordering::Relaxed);
    jit_binop_f64(a, b, |builder, x, y| builder.ins().fadd(x, y))
}

pub fn jit_sub_f64(a: f64, b: f64) -> f64 {
    JIT_SUB_F64_COUNT.fetch_add(1, Ordering::Relaxed);
    jit_binop_f64(a, b, |builder, x, y| builder.ins().fsub(x, y))
}

pub fn jit_mul_f64(a: f64, b: f64) -> f64 {
    JIT_MUL_F64_COUNT.fetch_add(1, Ordering::Relaxed);
    jit_binop_f64(a, b, |builder, x, y| builder.ins().fmul(x, y))
}

pub fn jit_div_f64(a: f64, b: f64) -> f64 {
    JIT_DIV_F64_COUNT.fetch_add(1, Ordering::Relaxed);
    jit_binop_f64(a, b, |builder, x, y| builder.ins().fdiv(x, y))
} 

fn jit_cmp_i64(a: i64, b: i64, op: fn(&mut FunctionBuilder, Value, Value) -> Value) -> bool {
    mark_jit_used();
    let mut builder = JITBuilder::new(cranelift_module::default_libcall_names()).expect("JITBuilder failed");
    let mut module = JITModule::new(builder);
    let mut ctx = module.make_context();
    ctx.func.signature.returns.push(AbiParam::new(types::I8));
    ctx.func.signature.params.push(AbiParam::new(types::I64));
    ctx.func.signature.params.push(AbiParam::new(types::I64));
    {
        let mut builder_ctx = FunctionBuilderContext::new();
        let mut func_builder = FunctionBuilder::new(&mut ctx.func, &mut builder_ctx);
        let block = func_builder.create_block();
        func_builder.append_block_params_for_function_params(block);
        func_builder.switch_to_block(block);
        func_builder.seal_block(block);
        let x = func_builder.block_params(block)[0];
        let y = func_builder.block_params(block)[1];
        let result = op(&mut func_builder, x, y);
        func_builder.ins().return_(&[result]);
        func_builder.finalize();
    }
    let func_id = module
        .declare_function("cmp_i64", cranelift_module::Linkage::Export, &ctx.func.signature)
        .unwrap();
    module.define_function(func_id, &mut ctx).unwrap();
    module.clear_context(&mut ctx);
    let _ = module.finalize_definitions();
    let code = module.get_finalized_function(func_id);
    let func = unsafe { std::mem::transmute::<_, fn(i64, i64) -> i8>(code) };
    func(a, b) != 0
}

pub fn jit_eq_i64(a: i64, b: i64) -> bool {
    JIT_EQ_I64_COUNT.fetch_add(1, Ordering::Relaxed);
    jit_cmp_i64(a, b, |builder, x, y| builder.ins().icmp(IntCC::Equal, x, y))
}

pub fn jit_ne_i64(a: i64, b: i64) -> bool {
    JIT_NE_I64_COUNT.fetch_add(1, Ordering::Relaxed);
    jit_cmp_i64(a, b, |builder, x, y| builder.ins().icmp(IntCC::NotEqual, x, y))
}

pub fn jit_lt_i64(a: i64, b: i64) -> bool {
    JIT_LT_I64_COUNT.fetch_add(1, Ordering::Relaxed);
    jit_cmp_i64(a, b, |builder, x, y| builder.ins().icmp(IntCC::SignedLessThan, x, y))
}

pub fn jit_le_i64(a: i64, b: i64) -> bool {
    JIT_LE_I64_COUNT.fetch_add(1, Ordering::Relaxed);
    jit_cmp_i64(a, b, |builder, x, y| builder.ins().icmp(IntCC::SignedLessThanOrEqual, x, y))
}

pub fn jit_gt_i64(a: i64, b: i64) -> bool {
    JIT_GT_I64_COUNT.fetch_add(1, Ordering::Relaxed);
    jit_cmp_i64(a, b, |builder, x, y| builder.ins().icmp(IntCC::SignedGreaterThan, x, y))
}

pub fn jit_ge_i64(a: i64, b: i64) -> bool {
    JIT_GE_I64_COUNT.fetch_add(1, Ordering::Relaxed);
    jit_cmp_i64(a, b, |builder, x, y| builder.ins().icmp(IntCC::SignedGreaterThanOrEqual, x, y))
}

fn jit_cmp_f64(a: f64, b: f64, op: fn(&mut FunctionBuilder, Value, Value) -> Value) -> bool {
    mark_jit_used();
    let mut builder = JITBuilder::new(cranelift_module::default_libcall_names()).expect("JITBuilder failed");
    let mut module = JITModule::new(builder);
    let mut ctx = module.make_context();
    ctx.func.signature.returns.push(AbiParam::new(types::I8));
    ctx.func.signature.params.push(AbiParam::new(types::F64));
    ctx.func.signature.params.push(AbiParam::new(types::F64));
    {
        let mut builder_ctx = FunctionBuilderContext::new();
        let mut func_builder = FunctionBuilder::new(&mut ctx.func, &mut builder_ctx);
        let block = func_builder.create_block();
        func_builder.append_block_params_for_function_params(block);
        func_builder.switch_to_block(block);
        func_builder.seal_block(block);
        let x = func_builder.block_params(block)[0];
        let y = func_builder.block_params(block)[1];
        let result = op(&mut func_builder, x, y);
        func_builder.ins().return_(&[result]);
        func_builder.finalize();
    }
    let func_id = module
        .declare_function("cmp_f64", cranelift_module::Linkage::Export, &ctx.func.signature)
        .unwrap();
    module.define_function(func_id, &mut ctx).unwrap();
    module.clear_context(&mut ctx);
    let _ = module.finalize_definitions();
    let code = module.get_finalized_function(func_id);
    let func = unsafe { std::mem::transmute::<_, fn(f64, f64) -> i8>(code) };
    func(a, b) != 0
}

pub fn jit_eq_f64(a: f64, b: f64) -> bool {
    JIT_EQ_F64_COUNT.fetch_add(1, Ordering::Relaxed);
    jit_cmp_f64(a, b, |builder, x, y| builder.ins().fcmp(FloatCC::Equal, x, y))
}

pub fn jit_ne_f64(a: f64, b: f64) -> bool {
    JIT_NE_F64_COUNT.fetch_add(1, Ordering::Relaxed);
    jit_cmp_f64(a, b, |builder, x, y| builder.ins().fcmp(FloatCC::NotEqual, x, y))
}

pub fn jit_lt_f64(a: f64, b: f64) -> bool {
    JIT_LT_F64_COUNT.fetch_add(1, Ordering::Relaxed);
    jit_cmp_f64(a, b, |builder, x, y| builder.ins().fcmp(FloatCC::LessThan, x, y))
}

pub fn jit_le_f64(a: f64, b: f64) -> bool {
    JIT_LE_F64_COUNT.fetch_add(1, Ordering::Relaxed);
    jit_cmp_f64(a, b, |builder, x, y| builder.ins().fcmp(FloatCC::LessThanOrEqual, x, y))
}

pub fn jit_gt_f64(a: f64, b: f64) -> bool {
    JIT_GT_F64_COUNT.fetch_add(1, Ordering::Relaxed);
    jit_cmp_f64(a, b, |builder, x, y| builder.ins().fcmp(FloatCC::GreaterThan, x, y))
}

pub fn jit_ge_f64(a: f64, b: f64) -> bool {
    JIT_GE_F64_COUNT.fetch_add(1, Ordering::Relaxed);
    jit_cmp_f64(a, b, |builder, x, y| builder.ins().fcmp(FloatCC::GreaterThanOrEqual, x, y))
} 

pub fn jit_and_bool(a: bool, b: bool) -> bool {
    JIT_AND_BOOL_COUNT.fetch_add(1, Ordering::Relaxed);
    mark_jit_used();
    let mut builder = JITBuilder::new(cranelift_module::default_libcall_names()).expect("JITBuilder failed");
    let mut module = JITModule::new(builder);
    let mut ctx = module.make_context();
    ctx.func.signature.returns.push(AbiParam::new(types::I8));
    ctx.func.signature.params.push(AbiParam::new(types::I8));
    ctx.func.signature.params.push(AbiParam::new(types::I8));
    {
        let mut builder_ctx = FunctionBuilderContext::new();
        let mut func_builder = FunctionBuilder::new(&mut ctx.func, &mut builder_ctx);
        let block = func_builder.create_block();
        func_builder.append_block_params_for_function_params(block);
        func_builder.switch_to_block(block);
        func_builder.seal_block(block);
        let x = func_builder.block_params(block)[0];
        let y = func_builder.block_params(block)[1];
        let result = func_builder.ins().band(x, y);
        func_builder.ins().return_(&[result]);
        func_builder.finalize();
    }
    let func_id = module
        .declare_function("and_bool", cranelift_module::Linkage::Export, &ctx.func.signature)
        .unwrap();
    module.define_function(func_id, &mut ctx).unwrap();
    module.clear_context(&mut ctx);
    let _ = module.finalize_definitions();
    let code = module.get_finalized_function(func_id);
    let func = unsafe { std::mem::transmute::<_, fn(i8, i8) -> i8>(code) };
    func(a as i8, b as i8) != 0
}

pub fn jit_or_bool(a: bool, b: bool) -> bool {
    JIT_OR_BOOL_COUNT.fetch_add(1, Ordering::Relaxed);
    mark_jit_used();
    let mut builder = JITBuilder::new(cranelift_module::default_libcall_names()).expect("JITBuilder failed");
    let mut module = JITModule::new(builder);
    let mut ctx = module.make_context();
    ctx.func.signature.returns.push(AbiParam::new(types::I8));
    ctx.func.signature.params.push(AbiParam::new(types::I8));
    ctx.func.signature.params.push(AbiParam::new(types::I8));
    {
        let mut builder_ctx = FunctionBuilderContext::new();
        let mut func_builder = FunctionBuilder::new(&mut ctx.func, &mut builder_ctx);
        let block = func_builder.create_block();
        func_builder.append_block_params_for_function_params(block);
        func_builder.switch_to_block(block);
        func_builder.seal_block(block);
        let x = func_builder.block_params(block)[0];
        let y = func_builder.block_params(block)[1];
        let result = func_builder.ins().bor(x, y);
        func_builder.ins().return_(&[result]);
        func_builder.finalize();
    }
    let func_id = module
        .declare_function("or_bool", cranelift_module::Linkage::Export, &ctx.func.signature)
        .unwrap();
    module.define_function(func_id, &mut ctx).unwrap();
    module.clear_context(&mut ctx);
    let _ = module.finalize_definitions();
    let code = module.get_finalized_function(func_id);
    let func = unsafe { std::mem::transmute::<_, fn(i8, i8) -> i8>(code) };
    func(a as i8, b as i8) != 0
}

pub fn jit_not_bool(a: bool) -> bool {
    JIT_NOT_BOOL_COUNT.fetch_add(1, Ordering::Relaxed);
    mark_jit_used();
    let mut builder = JITBuilder::new(cranelift_module::default_libcall_names()).expect("JITBuilder failed");
    let mut module = JITModule::new(builder);
    let mut ctx = module.make_context();
    ctx.func.signature.returns.push(AbiParam::new(types::I8));
    ctx.func.signature.params.push(AbiParam::new(types::I8));
    {
        let mut builder_ctx = FunctionBuilderContext::new();
        let mut func_builder = FunctionBuilder::new(&mut ctx.func, &mut builder_ctx);
        let block = func_builder.create_block();
        func_builder.append_block_params_for_function_params(block);
        func_builder.switch_to_block(block);
        func_builder.seal_block(block);
        let x = func_builder.block_params(block)[0];
        let result = func_builder.ins().bnot(x);
        func_builder.ins().return_(&[result]);
        func_builder.finalize();
    }
    let func_id = module
        .declare_function("not_bool", cranelift_module::Linkage::Export, &ctx.func.signature)
        .unwrap();
    module.define_function(func_id, &mut ctx).unwrap();
    module.clear_context(&mut ctx);
    let _ = module.finalize_definitions();
    let code = module.get_finalized_function(func_id);
    let func = unsafe { std::mem::transmute::<_, fn(i8) -> i8>(code) };
    func(a as i8) != 0
}

pub fn jit_stats() -> String {
    let mut s = String::new();
    if was_jit_used() {
        s.push_str("JIT调用统计：\n");
        s.push_str(&format!("  int加法: {}\n", JIT_ADD_COUNT.load(Ordering::Relaxed)));
        s.push_str(&format!("  int减法: {}\n", JIT_SUB_COUNT.load(Ordering::Relaxed)));
        s.push_str(&format!("  int乘法: {}\n", JIT_MUL_COUNT.load(Ordering::Relaxed)));
        s.push_str(&format!("  int除法: {}\n", JIT_DIV_COUNT.load(Ordering::Relaxed)));
        s.push_str(&format!("  int取模: {}\n", JIT_MOD_COUNT.load(Ordering::Relaxed)));
        s.push_str(&format!("  long加法: {}\n", JIT_ADD_COUNT.load(Ordering::Relaxed)));
        s.push_str(&format!("  long减法: {}\n", JIT_SUB_COUNT.load(Ordering::Relaxed)));
        s.push_str(&format!("  long乘法: {}\n", JIT_MUL_COUNT.load(Ordering::Relaxed)));
        s.push_str(&format!("  long除法: {}\n", JIT_DIV_COUNT.load(Ordering::Relaxed)));
        s.push_str(&format!("  long取模: {}\n", JIT_MOD_COUNT.load(Ordering::Relaxed)));
        s.push_str(&format!("  float加法: {}\n", JIT_ADD_F64_COUNT.load(Ordering::Relaxed)));
        s.push_str(&format!("  float减法: {}\n", JIT_SUB_F64_COUNT.load(Ordering::Relaxed)));
        s.push_str(&format!("  float乘法: {}\n", JIT_MUL_F64_COUNT.load(Ordering::Relaxed)));
        s.push_str(&format!("  float除法: {}\n", JIT_DIV_F64_COUNT.load(Ordering::Relaxed)));
        s.push_str(&format!("  int等于: {}\n", JIT_EQ_I64_COUNT.load(Ordering::Relaxed)));
        s.push_str(&format!("  int不等: {}\n", JIT_NE_I64_COUNT.load(Ordering::Relaxed)));
        s.push_str(&format!("  int小于: {}\n", JIT_LT_I64_COUNT.load(Ordering::Relaxed)));
        s.push_str(&format!("  int小于等于: {}\n", JIT_LE_I64_COUNT.load(Ordering::Relaxed)));
        s.push_str(&format!("  int大于: {}\n", JIT_GT_I64_COUNT.load(Ordering::Relaxed)));
        s.push_str(&format!("  int大于等于: {}\n", JIT_GE_I64_COUNT.load(Ordering::Relaxed)));
        s.push_str(&format!("  long等于: {}\n", JIT_EQ_I64_COUNT.load(Ordering::Relaxed)));
        s.push_str(&format!("  long不等: {}\n", JIT_NE_I64_COUNT.load(Ordering::Relaxed)));
        s.push_str(&format!("  long小于: {}\n", JIT_LT_I64_COUNT.load(Ordering::Relaxed)));
        s.push_str(&format!("  long小于等于: {}\n", JIT_LE_I64_COUNT.load(Ordering::Relaxed)));
        s.push_str(&format!("  long大于: {}\n", JIT_GT_I64_COUNT.load(Ordering::Relaxed)));
        s.push_str(&format!("  long大于等于: {}\n", JIT_GE_I64_COUNT.load(Ordering::Relaxed)));
        s.push_str(&format!("  float等于: {}\n", JIT_EQ_F64_COUNT.load(Ordering::Relaxed)));
        s.push_str(&format!("  float不等: {}\n", JIT_NE_F64_COUNT.load(Ordering::Relaxed)));
        s.push_str(&format!("  float小于: {}\n", JIT_LT_F64_COUNT.load(Ordering::Relaxed)));
        s.push_str(&format!("  float小于等于: {}\n", JIT_LE_F64_COUNT.load(Ordering::Relaxed)));
        s.push_str(&format!("  float大于: {}\n", JIT_GT_F64_COUNT.load(Ordering::Relaxed)));
        s.push_str(&format!("  float大于等于: {}\n", JIT_GE_F64_COUNT.load(Ordering::Relaxed)));
        s.push_str(&format!("  bool与: {}\n", JIT_AND_BOOL_COUNT.load(Ordering::Relaxed)));
        s.push_str(&format!("  bool或: {}\n", JIT_OR_BOOL_COUNT.load(Ordering::Relaxed)));
        s.push_str(&format!("  bool非: {}\n", JIT_NOT_BOOL_COUNT.load(Ordering::Relaxed)));
    }
    s
} 

pub fn jit_eval_const_expr(expr: &crate::ast::Expression) -> Option<crate::interpreter::value::Value> {
    use crate::ast::{Expression, BinaryOperator, LogicalOperator, CompareOperator};
    use crate::interpreter::value::Value;
    match expr {
        Expression::IntLiteral(i) => Some(Value::Int(*i)),
        Expression::LongLiteral(l) => Some(Value::Long(*l)),
        Expression::FloatLiteral(f) => Some(Value::Float(*f)),
        Expression::BoolLiteral(b) => Some(Value::Bool(*b)),
        Expression::BinaryOp(left, op, right) => {
            let l = jit_eval_const_expr(left)?;
            let r = jit_eval_const_expr(right)?;
            match (l, r, op) {
                (Value::Int(a), Value::Int(b), BinaryOperator::Add) => Some(Value::Int(jit_add(a as i64, b as i64) as i32)),
                (Value::Int(a), Value::Int(b), BinaryOperator::Subtract) => Some(Value::Int(jit_sub(a as i64, b as i64) as i32)),
                (Value::Int(a), Value::Int(b), BinaryOperator::Multiply) => Some(Value::Int(jit_mul(a as i64, b as i64) as i32)),
                (Value::Int(a), Value::Int(b), BinaryOperator::Divide) => Some(Value::Int(jit_div(a as i64, b as i64) as i32)),
                (Value::Int(a), Value::Int(b), BinaryOperator::Modulo) => Some(Value::Int(jit_mod(a as i64, b as i64) as i32)),
                (Value::Long(a), Value::Long(b), BinaryOperator::Add) => Some(Value::Long(jit_add(a, b))),
                (Value::Long(a), Value::Long(b), BinaryOperator::Subtract) => Some(Value::Long(jit_sub(a, b))),
                (Value::Long(a), Value::Long(b), BinaryOperator::Multiply) => Some(Value::Long(jit_mul(a, b))),
                (Value::Long(a), Value::Long(b), BinaryOperator::Divide) => Some(Value::Long(jit_div(a, b))),
                (Value::Long(a), Value::Long(b), BinaryOperator::Modulo) => Some(Value::Long(jit_mod(a, b))),
                (Value::Float(a), Value::Float(b), BinaryOperator::Add) => Some(Value::Float(jit_add_f64(a, b))),
                (Value::Float(a), Value::Float(b), BinaryOperator::Subtract) => Some(Value::Float(jit_sub_f64(a, b))),
                (Value::Float(a), Value::Float(b), BinaryOperator::Multiply) => Some(Value::Float(jit_mul_f64(a, b))),
                (Value::Float(a), Value::Float(b), BinaryOperator::Divide) => Some(Value::Float(jit_div_f64(a, b))),
                _ => None,
            }
        },
        Expression::LogicalOp(left, op, right) => {
            let l = jit_eval_const_expr(left)?;
            let r = if let LogicalOperator::Not = op { None } else { jit_eval_const_expr(right) };
            match (l, r, op) {
                (Value::Bool(a), Some(Value::Bool(b)), LogicalOperator::And) => Some(Value::Bool(jit_and_bool(a, b))),
                (Value::Bool(a), Some(Value::Bool(b)), LogicalOperator::Or) => Some(Value::Bool(jit_or_bool(a, b))),
                (Value::Bool(a), None, LogicalOperator::Not) => Some(Value::Bool(jit_not_bool(a))),
                _ => None,
            }
        },
        Expression::CompareOp(left, op, right) => {
            let l = jit_eval_const_expr(left)?;
            let r = jit_eval_const_expr(right)?;
            match (l, r, op) {
                (Value::Int(a), Value::Int(b), CompareOperator::Equal) => Some(Value::Bool(jit_eq_i64(a as i64, b as i64))),
                (Value::Int(a), Value::Int(b), CompareOperator::NotEqual) => Some(Value::Bool(jit_ne_i64(a as i64, b as i64))),
                (Value::Int(a), Value::Int(b), CompareOperator::Greater) => Some(Value::Bool(jit_gt_i64(a as i64, b as i64))),
                (Value::Int(a), Value::Int(b), CompareOperator::Less) => Some(Value::Bool(jit_lt_i64(a as i64, b as i64))),
                (Value::Int(a), Value::Int(b), CompareOperator::GreaterEqual) => Some(Value::Bool(jit_ge_i64(a as i64, b as i64))),
                (Value::Int(a), Value::Int(b), CompareOperator::LessEqual) => Some(Value::Bool(jit_le_i64(a as i64, b as i64))),
                (Value::Long(a), Value::Long(b), CompareOperator::Equal) => Some(Value::Bool(jit_eq_i64(a, b))),
                (Value::Long(a), Value::Long(b), CompareOperator::NotEqual) => Some(Value::Bool(jit_ne_i64(a, b))),
                (Value::Long(a), Value::Long(b), CompareOperator::Greater) => Some(Value::Bool(jit_gt_i64(a, b))),
                (Value::Long(a), Value::Long(b), CompareOperator::Less) => Some(Value::Bool(jit_lt_i64(a, b))),
                (Value::Long(a), Value::Long(b), CompareOperator::GreaterEqual) => Some(Value::Bool(jit_ge_i64(a, b))),
                (Value::Long(a), Value::Long(b), CompareOperator::LessEqual) => Some(Value::Bool(jit_le_i64(a, b))),
                (Value::Float(a), Value::Float(b), CompareOperator::Equal) => Some(Value::Bool(jit_eq_f64(a, b))),
                (Value::Float(a), Value::Float(b), CompareOperator::NotEqual) => Some(Value::Bool(jit_ne_f64(a, b))),
                (Value::Float(a), Value::Float(b), CompareOperator::Greater) => Some(Value::Bool(jit_gt_f64(a, b))),
                (Value::Float(a), Value::Float(b), CompareOperator::Less) => Some(Value::Bool(jit_lt_f64(a, b))),
                (Value::Float(a), Value::Float(b), CompareOperator::GreaterEqual) => Some(Value::Bool(jit_ge_f64(a, b))),
                (Value::Float(a), Value::Float(b), CompareOperator::LessEqual) => Some(Value::Bool(jit_le_f64(a, b))),
                _ => None,
            }
        },
        _ => None,
    }
} 

pub struct JitCompiledExpr {
    pub var_names: Vec<String>,
    pub func: unsafe extern "C" fn(*const i64) -> i64,
    pub code_ptr: *const u8, // 保证生命周期
}

impl JitCompiledExpr {
    pub fn call(&self, vars: &HashMap<String, i64>) -> i64 {
        let mut args = Vec::with_capacity(self.var_names.len());
        for name in &self.var_names {
            args.push(*vars.get(name).expect("变量未赋值"));
        }
        unsafe { (self.func)(args.as_ptr()) }
    }
}

pub fn jit_compile_int_expr(expr: &crate::ast::Expression) -> Option<JitCompiledExpr> {
    use cranelift::prelude::*;
    use cranelift_jit::{JITBuilder, JITModule};
    use cranelift_module::Module;
    use crate::ast::{Expression, BinaryOperator};
    
    // 检查是否包含不支持的操作符
    fn is_supported_for_jit(expr: &Expression) -> bool {
        match expr {
            Expression::IntLiteral(_) | Expression::Variable(_) => true,
            Expression::BinaryOp(left, _, right) => {
                is_supported_for_jit(left) && is_supported_for_jit(right)
            },
            _ => false,
        }
    }
    
    if !is_supported_for_jit(expr) {
        return None;
    }
    
    // 1. 收集变量名
    let mut var_names = Vec::new();
    fn collect_vars(expr: &Expression, vars: &mut Vec<String>) {
        match expr {
            Expression::Variable(name) => {
                if !vars.contains(name) {
                    vars.push(name.clone());
                }
            },
            Expression::BinaryOp(left, _, right) => {
                collect_vars(left, vars);
                collect_vars(right, vars);
            },
            Expression::IntLiteral(_) => {},
            _ => {},
        }
    }
    collect_vars(expr, &mut var_names);
    // 2. 生成JIT代码
    let mut builder = JITBuilder::new(cranelift_module::default_libcall_names()).expect("JITBuilder failed");
    let mut module = JITModule::new(builder);
    let mut ctx = module.make_context();
    ctx.func.signature.returns.push(AbiParam::new(types::I64));
    for _ in &var_names {
        ctx.func.signature.params.push(AbiParam::new(types::I64));
    }
    {
        let mut builder_ctx = FunctionBuilderContext::new();
        let mut func_builder = FunctionBuilder::new(&mut ctx.func, &mut builder_ctx);
        let block = func_builder.create_block();
        func_builder.append_block_params_for_function_params(block);
        func_builder.switch_to_block(block);
        func_builder.seal_block(block);
        fn codegen(expr: &Expression, func_builder: &mut FunctionBuilder, var_names: &Vec<String>, block: Block) -> Value {
            match expr {
                Expression::IntLiteral(i) => func_builder.ins().iconst(types::I64, *i as i64),
                Expression::Variable(name) => {
                    let idx = var_names.iter().position(|n| n == name).unwrap();
                    func_builder.block_params(block)[idx]
                },
                Expression::BinaryOp(left, op, right) => {
                    let l = codegen(left, func_builder, var_names, block);
                    let r = codegen(right, func_builder, var_names, block);
                    match op {
                        BinaryOperator::Add => func_builder.ins().iadd(l, r),
                        BinaryOperator::Subtract => func_builder.ins().isub(l, r),
                        BinaryOperator::Multiply => func_builder.ins().imul(l, r),
                        BinaryOperator::Divide => func_builder.ins().sdiv(l, r),
                        BinaryOperator::Modulo => func_builder.ins().srem(l, r),
                    }
                },
                _ => panic!("只支持int型变量和常量的算术表达式JIT"),
            }
        }
        let result = codegen(expr, &mut func_builder, &var_names, block);
        func_builder.ins().return_(&[result]);
        func_builder.finalize();
    }
    let func_id = module
        .declare_function("jit_expr", cranelift_module::Linkage::Export, &ctx.func.signature)
        .unwrap();
    module.define_function(func_id, &mut ctx).unwrap();
    module.clear_context(&mut ctx);
    let _ = module.finalize_definitions();
    let code = module.get_finalized_function(func_id);
    let func = unsafe { std::mem::transmute::<_, unsafe extern "C" fn(*const i64) -> i64>(code) };
    Some(JitCompiledExpr { var_names, func, code_ptr: code })
} 
