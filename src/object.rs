use std::fmt::Debug;
use super::chunk::Chunk;

#[derive(Debug)]
pub (crate) enum Object {
    String(*const String),
}

impl Object {
    pub (crate) fn to_string(&self) -> String {
        match self {
            Object::String(string) => unsafe { &**string }.to_string(),
        }
    }
}

impl Clone for Object {
    fn clone(&self) -> Self {
        match self {
            Object::String(string) => Object::String(unsafe { &**string } as *const String),
        }
    }
}

impl PartialEq for Object {
    fn eq(&self, other: &Self) -> bool {
        match self {
            Object::String(a) => {
                match other {
                    Object::String(b) => *a == *b,
                }
            },
        }
    }

    fn ne(&self, other: &Self) -> bool {
        !self.eq(other)
    }
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