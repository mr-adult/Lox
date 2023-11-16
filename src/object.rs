use std::{fmt::Debug, rc::Rc};
use super::chunk::Chunk;

#[derive(Debug, Eq, Hash)]
pub (crate) enum Object {
    String(Rc<str>),
}

impl Object {
    pub (crate) fn to_string(&self) -> String {
        match self {
            Object::String(string) => string.to_string(),
        }
    }
}

impl Clone for Object {
    fn clone(&self) -> Self {
        match self {
            Object::String(string) => Object::String(string.clone()),
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