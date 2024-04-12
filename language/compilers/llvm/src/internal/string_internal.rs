use crate::compiler::CompilerImpl;
use crate::internal::instructions::malloc_type;
use crate::internal::intrinsics::compile_llvm_intrinsics;
use crate::type_getter::CompilerTypeGetter;
use inkwell::values::{BasicMetadataValueEnum, BasicValue, FunctionValue};
use inkwell::AddressSpace;

/// Compiles internal string methods
pub fn string_internal<'ctx>(
    type_getter: &CompilerTypeGetter<'ctx>,
    compiler: &CompilerImpl<'ctx>,
    name: &String,
    value: &FunctionValue<'ctx>,
) -> bool {
    let params = value.get_params();
    if name.starts_with("string::Cast") {
        type_getter.compiler.builder.build_return(Some(value.get_params().first().unwrap())).unwrap();
    } else if name.starts_with("string::Add<char + u64>_char::add") {
        let pointer_type = params.first().unwrap().into_pointer_value();
        let malloc = malloc_type(type_getter, pointer_type.get_type().const_zero(), &mut 0);
        let pointer_type = compiler
            .builder
            .build_bitcast(pointer_type, compiler.context.i64_type().ptr_type(AddressSpace::default()), "1")
            .unwrap()
            .into_pointer_value();
        let returning = compiler
            .builder
            .build_int_add(
                compiler.builder.build_load(pointer_type, "2").unwrap().into_int_value(),
                compiler.builder.build_load(params.get(1).unwrap().into_pointer_value(), "3").unwrap().into_int_value(),
                "1",
            )
            .unwrap();
        compiler.builder.build_store(malloc, returning).unwrap();
        compiler.builder.build_return(Some(&malloc)).unwrap();
    } else if name.starts_with("string::Add<str + str>_str::add") {
        let length = type_getter
            .compiler
            .builder
            .build_call(
                type_getter
                    .compiler
                    .module
                    .get_function("strlen")
                    .unwrap_or_else(|| compile_llvm_intrinsics("strlen", type_getter)),
                &[BasicMetadataValueEnum::PointerValue(value.get_params().first().unwrap().into_pointer_value())],
                "0",
            )
            .unwrap()
            .try_as_basic_value()
            .unwrap_left()
            .into_int_value();
        let second_length = type_getter
            .compiler
            .builder
            .build_call(
                type_getter
                    .compiler
                    .module
                    .get_function("strlen")
                    .unwrap_or_else(|| compile_llvm_intrinsics("strlen", type_getter)),
                &[BasicMetadataValueEnum::PointerValue(value.get_params().get(1).unwrap().into_pointer_value())],
                "2",
            )
            .unwrap()
            .try_as_basic_value()
            .unwrap_left()
            .into_int_value();
        let total = type_getter.compiler.builder.build_int_add(length, second_length, "4").unwrap();
        let malloc = type_getter
            .compiler
            .builder
            .build_call(
                type_getter
                    .compiler
                    .module
                    .get_function("malloc")
                    .unwrap_or_else(|| compile_llvm_intrinsics("malloc", type_getter)),
                &[BasicMetadataValueEnum::IntValue(total)],
                "5",
            )
            .unwrap()
            .try_as_basic_value()
            .unwrap_left()
            .into_pointer_value();
        type_getter
            .compiler
            .builder
            .build_call(
                type_getter
                    .compiler
                    .module
                    .get_function("strcpy")
                    .unwrap_or_else(|| compile_llvm_intrinsics("strcpy", type_getter)),
                &[
                    BasicMetadataValueEnum::PointerValue(malloc),
                    BasicMetadataValueEnum::PointerValue(value.get_params().first().unwrap().into_pointer_value()),
                ],
                "6",
            )
            .unwrap()
            .try_as_basic_value()
            .unwrap_left()
            .into_pointer_value();
        type_getter
            .compiler
            .builder
            .build_call(
                type_getter
                    .compiler
                    .module
                    .get_function("strcat")
                    .unwrap_or_else(|| compile_llvm_intrinsics("strcat", type_getter)),
                &[
                    BasicMetadataValueEnum::PointerValue(malloc),
                    BasicMetadataValueEnum::PointerValue(value.get_params().get(1).unwrap().into_pointer_value()),
                ],
                "7",
            )
            .unwrap()
            .try_as_basic_value()
            .unwrap_left()
            .into_pointer_value();
        type_getter.compiler.builder.build_return(Some(&malloc.as_basic_value_enum())).unwrap();
    } else if name.starts_with("string::Add<str + char>_str::add") {
        let length = type_getter
            .compiler
            .builder
            .build_call(
                type_getter
                    .compiler
                    .module
                    .get_function("strlen")
                    .unwrap_or_else(|| compile_llvm_intrinsics("strlen", type_getter)),
                &[BasicMetadataValueEnum::PointerValue(value.get_params().first().unwrap().into_pointer_value())],
                "0",
            )
            .unwrap()
            .try_as_basic_value()
            .unwrap_left()
            .into_int_value();
        let total = type_getter
            .compiler
            .builder
            .build_int_add(length, type_getter.compiler.context.i64_type().const_int(1, false), "4")
            .unwrap();
        let malloc = type_getter
            .compiler
            .builder
            .build_call(
                type_getter
                    .compiler
                    .module
                    .get_function("malloc")
                    .unwrap_or_else(|| compile_llvm_intrinsics("malloc", type_getter)),
                &[BasicMetadataValueEnum::IntValue(total)],
                "5",
            )
            .unwrap()
            .try_as_basic_value()
            .unwrap_left()
            .into_pointer_value();
        type_getter
            .compiler
            .builder
            .build_call(
                type_getter
                    .compiler
                    .module
                    .get_function("strcpy")
                    .unwrap_or_else(|| compile_llvm_intrinsics("strcpy", type_getter)),
                &[
                    BasicMetadataValueEnum::PointerValue(malloc),
                    BasicMetadataValueEnum::PointerValue(value.get_params().first().unwrap().into_pointer_value()),
                ],
                "6",
            )
            .unwrap()
            .try_as_basic_value()
            .unwrap_left()
            .into_pointer_value();
        type_getter
            .compiler
            .builder
            .build_store(
                unsafe { type_getter.compiler.builder.build_in_bounds_gep(malloc, &[length], "7").unwrap() },
                type_getter
                    .compiler
                    .builder
                    .build_load(value.get_params().get(1).unwrap().into_pointer_value(), "8")
                    .unwrap(),
            )
            .unwrap();
        let plus_one = type_getter
            .compiler
            .builder
            .build_int_add(length, type_getter.compiler.context.i64_type().const_int(1, false), "9")
            .unwrap();
        type_getter
            .compiler
            .builder
            .build_store(
                unsafe { type_getter.compiler.builder.build_in_bounds_gep(malloc, &[plus_one], "10").unwrap() },
                type_getter.compiler.context.i8_type().const_zero(),
            )
            .unwrap();
        type_getter.compiler.builder.build_return(Some(&malloc.as_basic_value_enum())).unwrap();
    } else if name.starts_with("string::Add<char + str>_char::add") {
        // get length of str
        let length = type_getter
            .compiler
            .builder
            .build_call(
                type_getter
                    .compiler
                    .module
                    .get_function("strlen")
                    .unwrap_or_else(|| compile_llvm_intrinsics("strlen", type_getter)),
                &[BasicMetadataValueEnum::PointerValue(value.get_params().get(1).unwrap().into_pointer_value())],
                "0",
            )
            .unwrap()
            .try_as_basic_value()
            .unwrap_left()
            .into_int_value();
        // add one to str length to account for char
        let total = type_getter
            .compiler
            .builder
            .build_int_add(length, type_getter.compiler.context.i64_type().const_int(1, false), "4")
            .unwrap();
        // allocate memory for new string
        let malloc = type_getter
            .compiler
            .builder
            .build_call(
                type_getter
                    .compiler
                    .module
                    .get_function("malloc")
                    .unwrap_or_else(|| compile_llvm_intrinsics("malloc", type_getter)),
                &[BasicMetadataValueEnum::IntValue(total)],
                "5",
            )
            .unwrap()
            .try_as_basic_value()
            .unwrap_left()
            .into_pointer_value();
        // add one to malloc, to move str into malloc[1], not malloc[0]
        
        let plus_one;
        unsafe {
            plus_one = type_getter
                .compiler
                .builder
                // need to cast malloc to int before adding
                .build_in_bounds_gep(malloc, &[type_getter.compiler.context.i64_type().const_int(1, false)], "9")
                .unwrap();
        }
        // copy str into new memory
        type_getter
            .compiler
            .builder
            .build_call(
                type_getter
                    .compiler
                    .module
                    .get_function("strcpy")
                    .unwrap_or_else(|| compile_llvm_intrinsics("strcpy", type_getter)),
                &[
                    // change this to malloc + 1? this puts string at 0, we need at 1
                    BasicMetadataValueEnum::PointerValue(plus_one),
                    BasicMetadataValueEnum::PointerValue(value.get_params().get(1).unwrap().into_pointer_value()),
                ],
                "6",
            )
            .unwrap()
            .try_as_basic_value()
            .unwrap_left()
            .into_pointer_value();
        // add the char to beginning of new memory
        type_getter
            .compiler
            .builder
            .build_store(
                // change this? not &[length], but &[0]?
                unsafe { type_getter.compiler.builder.build_in_bounds_gep(malloc, &[type_getter.compiler.context.i64_type().const_int(0, false)], "7").unwrap() },
                type_getter
                    .compiler
                    .builder
                    .build_load(value.get_params().first().unwrap().into_pointer_value(), "8")
                    .unwrap(),
            )
            .unwrap();
        // no need to add escape character, as strcpy should have already moved it
        // finish instruction
        type_getter.compiler.builder.build_return(Some(&malloc.as_basic_value_enum())).unwrap();
        return true;
    } else {
        return false;
    }
    return true;
}
