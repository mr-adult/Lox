use std::array::from_fn;

use crate::{tokenizer::{Tokenizer, LoxToken}, object::{Function, UpValue}};

pub (crate) fn compile(source: &str) -> Option<Function> {
    let token_stream = Tokenizer::new(source);
    let mut compiler = Compiler::new(FunctionType::Script, None);

    let mut errs = Vec::new();
    for token_result in token_stream {
        match token_result {
            Err(err) => errs.push(err),
            Ok(token) => {
                todo!();
            }
        }
    }

    todo!();
}

struct Compiler<'c> {
    enclosing: Option<&'c Self>,
    function: Function,
    f_type: FunctionType,

    // locals: [Local; u8::MAX as usize],
    local_len: usize,
    // upvalues: [UpValue; u8::MAX as usize],
    scope_depth: usize,
}

impl<'c> Compiler<'c> {
    fn new(f_type: FunctionType, enclosing: Option<&'c Self>) -> Self {
        Self {
            enclosing: enclosing, 
            function: Function::new(), 
            f_type, 
            // locals: (), 
            local_len: 0, 
            // upvalues: (), 
            scope_depth: 0 
        }
    }
}

struct Local {
    name: LoxToken,
    depth: usize,
    is_captured: bool,
}

enum FunctionType {
    Function,
    Initializer,
    Method,
    Script,
}