use inkwell::builder::Builder;
use inkwell::{AddressSpace, IntPredicate};
use inkwell::types::BasicTypeEnum;
use inkwell::values::{BasicValueEnum, FunctionValue};
use crate::compiler::CompilerImpl;

pub fn compile_internal<'ctx>(compiler: &CompilerImpl<'ctx>, name: &String, value: FunctionValue<'ctx>) {
    let block = compiler.context.append_basic_block(value, "0");
    compiler.builder.position_at_end(block);
    let params = value.get_params();
    //Trunc to go u64 -> u8
    if name.starts_with("numbers::cast_") {
        build_cast(value.get_params().get(0).unwrap(), value.get_type().get_return_type().unwrap(), compiler);
        return;
    } else if name.starts_with("math::add_") {
        let returning = compiler.builder.build_int_add(compiler.builder.build_load(params.get(0).unwrap().into_pointer_value(), "2").into_int_value(),
                                                       compiler.builder.build_load(params.get(1).unwrap().into_pointer_value(), "3").into_int_value(), "1");
        compiler.builder.build_return(Some(&returning));
    } else if name.starts_with("math::equal_") {
        let returning = compiler.builder
            .build_int_compare(IntPredicate::EQ, compiler.builder.build_load(params.get(0).unwrap().into_pointer_value(), "2").into_int_value(),
                               compiler.builder.build_load(params.get(1).unwrap().into_pointer_value(), "3").into_int_value(), "1");
        compiler.builder.build_return(Some(&returning));
    } else if name.starts_with("array::index_") {
        let gep;
        unsafe {
            gep = compiler.builder
                .build_in_bounds_gep(params.get(0).unwrap().into_pointer_value(),
                                     &[get_loaded(&compiler.builder, params.get(1).unwrap()).into_int_value()], "1");
        }
        compiler.builder.build_return(Some(&compiler.builder.build_bitcast(gep, compiler.context.i64_type().ptr_type(AddressSpace::default()), "2")));
    } else {
        panic!("Unknown internal operation: {}", name)
    }
}

fn get_loaded<'ctx>(compiler: &Builder<'ctx>, value: &BasicValueEnum<'ctx>) -> BasicValueEnum<'ctx> {
    if value.is_pointer_value() {
        return compiler.build_load(value.into_pointer_value(), "0");
    }
    return value.clone();
}

fn build_cast(first: &BasicValueEnum, _second: BasicTypeEnum, compiler: &CompilerImpl) {
    //TODO float casting
    compiler.builder.build_return(Some(&compiler.builder.build_load(first.into_pointer_value(), "1")));
}