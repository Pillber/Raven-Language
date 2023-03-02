use std::fs;
use std::path::PathBuf;
use ast::program::Program;

pub mod code;
pub mod literal;
pub mod parser;
pub mod top_elements;
pub mod types;
pub mod util;

pub fn parse(input: Box<dyn FileStructure>) -> Program {
    let mut output = Program::new();
    let root_offset = input.get_root().to_str().unwrap().len() + 1;
    let mut failed = Vec::new();

    for file in input.get_files() {
        let name = file.to_str().unwrap()[root_offset..file.to_str().unwrap().len() - 3].to_string();
        match parser::parse(&mut output, &name, fs::read_to_string(&file).unwrap(), true) {
            Ok(_) => {},
            Err(mut errors) => failed.append(&mut errors)
        };
    }

    for file in input.get_files() {
        let name = file.to_str().unwrap()[root_offset..file.to_str().unwrap().len() - 3].to_string();
        match parser::parse(&mut output, &name, fs::read_to_string(&file).unwrap(), false) {
            Ok(_) => {},
            Err(mut errors) => failed.append(&mut errors)
        };

        for (_name, structure) in &output.elem_types {
            println!("{}", structure);
        }

        for (_name, function) in &output.static_functions {
            println!("{}", function);
        }
    }

    if !failed.is_empty() {
        let mut errors = "Parsing Errors:\n".to_string();
        for error in failed {
            errors += format!("\n{}\n", error).as_str();
        }
        panic!("{}", errors);
    }
    return output;
}

pub trait FileStructure {
    fn get_files(&self) -> Vec<PathBuf>;

    fn get_root(&self) -> PathBuf;
}