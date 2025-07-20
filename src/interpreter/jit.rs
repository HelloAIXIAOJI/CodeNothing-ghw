use cranelift::prelude::*;
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::Module;
use std::sync::atomic::{AtomicBool, Ordering, AtomicUsize};
use std::sync::Once;

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
    jit_binop_i64(a, b, |builder, x, y| builder.ins().iadd(x, y))
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
    }
    s
} 
