use std::fmt::Debug;
use crate::value::Value;
use super::chunk::Chunk;

#[derive(Debug)]
pub (crate) enum Object {
    Closure(Box<Closure>),
    Function(Box<Function>),
    String(Box<String>),
    Native(Box<Native>),
    Upvalue(Box<UpValue>),
}

#[derive(Debug)]
pub (crate) struct Function {
    pub (crate) arity: u8,
    pub (crate) upvalue_count: u8,
    pub (crate) chunks: Vec<Chunk>,
    pub (crate) name: String,
}

impl Function {
    pub (crate) fn new() -> Function {
        Function { 
            arity: 0, 
            upvalue_count: 0, 
            chunks: Vec::new(), 
            name: "<no name>".to_string(),
        }
    }
}

pub (crate) struct Native {
    arity: u8,
    args: Vec<Value>,
    implementation: Box<dyn Fn(Vec<Value>) -> Value>
}

impl Debug for Native {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Native")
            .field("arity", &self.arity)
            .field("args", &self.args)
            .finish()
    }
}

#[derive(Debug)]
pub (crate) struct UpValue {
    location: *mut Value,
    closed: Option<Value>,
}

#[derive(Debug)]
pub (crate) struct Closure {
    pub (crate) function: Function,
    pub (crate) upvalues: Vec<UpValue>
}

impl Closure {
    pub (crate) fn arity(&self) -> u8 {
        self.function.arity
    }
}