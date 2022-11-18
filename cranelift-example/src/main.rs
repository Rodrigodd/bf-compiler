use cranelift::{
    codegen::{
        ir::{types::I64, AbiParam, Function, InstBuilder, Signature, UserFuncName},
        isa::{self, CallConv},
        settings, verify_function, Context,
    },
    frontend::{FunctionBuilder, FunctionBuilderContext},
};
use target_lexicon::Triple;

fn main() {
    let builder = settings::builder();
    let flags = settings::Flags::new(builder);

    let isa = match isa::lookup(Triple::host()) {
        Err(err) => panic!("Error looking up target: {}", err),
        Ok(isa_builder) => isa_builder.finish(flags).unwrap(),
    };

    let mut sig = Signature::new(CallConv::SystemV);
    sig.params.push(AbiParam::new(I64));
    sig.returns.push(AbiParam::new(I64));

    let mut func = Function::with_name_signature(UserFuncName::default(), sig);

    let mut func_ctx = FunctionBuilderContext::new();
    let mut builder = FunctionBuilder::new(&mut func, &mut func_ctx);

    let block = builder.create_block();
    builder.seal_block(block);

    builder.append_block_params_for_function_params(block);
    builder.switch_to_block(block);

    let arg = builder.block_params(block)[0];
    let plus_one = builder.ins().iadd_imm(arg, 1);
    builder.ins().return_(&[plus_one]);

    builder.finalize();

    verify_function(&func, &*isa).unwrap();

    println!("{}", func.display());

    let mut ctx = Context::for_function(func);
    ctx.set_disasm(true);
    let code = ctx.compile(&*isa).unwrap();

    println!("{}", code.disasm.as_ref().unwrap());
    std::fs::write("dump.bin", code.code_buffer()).unwrap();

    // let mut buffer = memmap2::MmapOptions::new()
    //     .len(code.code_buffer().len())
    //     .map_anon()
    //     .unwrap();

    // buffer.copy_from_slice(code.code_buffer());

    // let buffer = buffer.make_exec().unwrap();

    // let x = unsafe {
    //     let code_fn: unsafe extern "sysv64" fn(usize) -> usize =
    //         std::mem::transmute(buffer.as_ptr());

    //     code_fn(1)
    // };

    // println!("out: {}", x);
}
