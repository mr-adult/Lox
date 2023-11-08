use crate::object::{Object, Function};

#[derive(Default)]
pub (crate) enum Value {
    #[default]
    Nil,
    Boolean(bool),
    Number(f64),
    Object(Box<Object>),
}

impl Value {
    pub (crate) fn to_string(&self) -> String {
        match self {
            Value::Nil => "nil".to_string(),
            Value::Boolean(bool) => bool.to_string(),
            Value::Number(num) => num.to_string(),
            Value::Object(_) => todo!(),
        }
    }
}

impl Clone for Value {
    fn clone(&self) -> Self {
        match self {
            Self::Nil => Self::Nil,
            Self::Boolean(arg0) => Self::Boolean(arg0.clone()),
            Self::Number(arg0) => Self::Number(arg0.clone()),
            Self::Object(arg0) => todo!(),
        }
    }
}

impl From<Function> for Value {
    fn from(func: Function) -> Self {
        todo!();
        // Value::Object(
        //     Box::new(
        //         Object::Closure(
        //             Box::new(
        //                 Closure {
        //                     function: func,
        //                     upvalues: Vec::with_capacity(func.upvalue_count as usize),
        //                 }
        //             )
        //         )
        //     )
        // )
    }
}
