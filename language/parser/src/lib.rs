#![feature(try_trait_v2)]

extern crate core;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::runtime::Handle;
use syntax::async_util::NameResolver;
use syntax::syntax::Syntax;
use syntax::types::Types;
use crate::parser::top_parser::parse_top;
use crate::parser::util::ParserUtils;
use crate::tokens::tokenizer::Tokenizer;
use crate::tokens::tokens::TokenTypes;

pub mod parser;
pub mod tokens;

pub async fn parse(syntax: Arc<Mutex<Syntax>>, handle: Handle, name: String, file: String) {
    let mut tokenizer = Tokenizer::new(file.as_bytes());
    let mut tokens = Vec::new();
    loop {
        tokens.push(tokenizer.next());
        if tokens.last().unwrap().token_type == TokenTypes::EOF {
            break
        }
    }
    let mut parser_utils = ParserUtils {
        buffer: file.as_bytes(),
        tokens,
        syntax,
        file: name,
        imports: ImportNameResolver::new(),
        handle,
    };
    parse_top(&mut parser_utils);
}

#[derive(Clone)]
pub struct ImportNameResolver {
    pub imports: HashMap<String, String>,
    pub generics: HashMap<String, Types>,
    pub parent: Option<String>,
    pub last_id: u32
}

impl ImportNameResolver {
    pub fn new() -> Self {
        return Self {
            imports: HashMap::new(),
            generics: HashMap::new(),
            parent: None,
            last_id: 0
        }
    }
}

impl NameResolver for ImportNameResolver {
    fn resolve<'a>(&'a self, name: &'a String) -> &'a String {
        return self.imports.get(name).unwrap_or(name);
    }

    fn generic(&self, name: &String) -> Option<Types> {
        return self.generics.get(name).map(|types| types.clone());
    }
}