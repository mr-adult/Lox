use std::usize;

use crate::{fixed_vec::FixedVec, value::Value};

#[repr(u8)]
pub (crate) enum OpCode {
    Constant = 0,
    Nil = 1,
    True = 2,
    False = 3,
    Pop = 4,
    GetLocal = 5,
    SetLocal = 6,
    GetGlobal = 7,
    DefineGlobal = 8,
    SetGlobal = 9,
    GetUpValue = 10,
    SetUpValue = 11,
    GetProperty = 12,
    SetProperty = 13,
    GetSuper = 14,
    Equal = 15,
    Greater = 16,
    Less = 17,
    Add = 18,
    Subtract = 19,
    Multiply = 20,
    Divide = 21,
    Not = 22,
    Negate = 23,
    Print = 24,
    Jump = 25,
    JumpIfFalse = 26,
    Loop = 27,
    Call = 28,
    Invoke = 29,
    SuperInvoke = 30,
    Closure = 31,
    CloseUpValue = 32,
    Return = 33,
    Class = 34,
    Inherit = 35,
    Method = 36,
}

impl OpCode {
    fn as_str(&self) -> &str {
        match self {
            OpCode::Constant => "Constant",
            OpCode::Nil => "Nil",
            OpCode::True => "True",
            OpCode::False => "False",
            OpCode::Pop => "Pop",
            OpCode::GetLocal => "GetLocal",
            OpCode::SetLocal => "SetLocal",
            OpCode::GetGlobal => "GetGlobal",
            OpCode::DefineGlobal => "DefineGlobal",
            OpCode::SetGlobal => "SetGlobal",
            OpCode::GetUpValue => "GetUpValue",
            OpCode::SetUpValue => "SetUpValue",
            OpCode::GetProperty => "GetProperty",
            OpCode::SetProperty => "SetProperty",
            OpCode::GetSuper => "GetSuper",
            OpCode::Equal => "Equal",
            OpCode::Greater => "Greater",
            OpCode::Less => "Less",
            OpCode::Add => "Add",
            OpCode::Subtract => "Subtract",
            OpCode::Multiply => "Multiply",
            OpCode::Divide => "Divide",
            OpCode::Not => "Not",
            OpCode::Negate => "Negate",
            OpCode::Print => "Print",
            OpCode::Jump => "Jump",
            OpCode::JumpIfFalse => "JumpIfFalse",
            OpCode::Loop => "Loop",
            OpCode::Call => "Call",
            OpCode::Invoke => "Invoke",
            OpCode::SuperInvoke => "SuperInvoke",
            OpCode::Closure => "Closure",
            OpCode::CloseUpValue => "CloseUpValue",
            OpCode::Return => "Return",
            OpCode::Class => "Class",
            OpCode::Inherit => "Inherit",
            OpCode::Method => "Method",
        }
    }
}

impl TryFrom<u8> for OpCode {
    type Error = String;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(OpCode::Constant),
            1 => Ok(OpCode::Nil),
            2 => Ok(OpCode::True),
            3 => Ok(OpCode::False),
            4 => Ok(OpCode::Pop),
            5 => Ok(OpCode::GetLocal),
            6 => Ok(OpCode::SetLocal),
            7 => Ok(OpCode::GetGlobal),
            8 => Ok(OpCode::DefineGlobal),
            9 => Ok(OpCode::SetGlobal),
            10 => Ok(OpCode::GetUpValue),
            11 => Ok(OpCode::SetUpValue),
            12 => Ok(OpCode::GetProperty),
            13 => Ok(OpCode::SetProperty),
            14 => Ok(OpCode::GetSuper),
            15 => Ok(OpCode::Equal),
            16 => Ok(OpCode::Greater),
            17 => Ok(OpCode::Less),
            18 => Ok(OpCode::Add),
            19 => Ok(OpCode::Subtract),
            20 => Ok(OpCode::Multiply),
            21 => Ok(OpCode::Divide),
            22 => Ok(OpCode::Not),
            23 => Ok(OpCode::Negate),
            24 => Ok(OpCode::Print),
            25 => Ok(OpCode::Jump),
            26 => Ok(OpCode::JumpIfFalse),
            27 => Ok(OpCode::Loop),
            28 => Ok(OpCode::Call),
            29 => Ok(OpCode::Invoke),
            30 => Ok(OpCode::SuperInvoke),
            31 => Ok(OpCode::Closure),
            32 => Ok(OpCode::CloseUpValue),
            33 => Ok(OpCode::Return),
            34 => Ok(OpCode::Class),
            35 => Ok(OpCode::Inherit),
            36 => Ok(OpCode::Method),
            other => Err(format!("Received invalid opcode: {}", other)),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub (crate) struct Chunk {
    pub (crate) line: usize,
    pub (crate) op: u8,
}

impl Chunk {
    pub (crate) fn disassemble_code<const N: usize>(code: &Vec<Chunk>, constants: &FixedVec<Value, N>, name: &str) {
        println!("== {} ==", name);

        let mut offset = 0;
        while offset < code.len() {
            offset = Chunk::disassemble_instruction(code, offset, constants);
        }
    }

    pub (crate) fn disassemble_instruction<const N: usize>(code: &Vec<Chunk>, index: usize, constants: &FixedVec<Value, N>) -> usize {
        let mut print_val = String::new();
        let result;

        if index > 0 && code[index].line == code[index - 1].line {
            print_val.push_str("   | ");
        } else {
            let num_str = code[index].line.to_string();
            for _ in 0..4 - num_str.len() {
                print_val.push('0');
            }
            print_val.push_str(&num_str);
            print_val.push(' ');
        }
      
      

        let instruction = OpCode::try_from(code[index].op).expect("OpCode to be valid");
        match instruction {
            OpCode::Constant => {
                let constant = code[index + 1].op;
                print_val.push_str("CONSTANT ");
                print_val.push_str(&constant.to_string());
                print_val.push(' ');
                print_val.push_str(&constants.get(constant as usize).unwrap().to_string());
                result = index + 2;
            },
            OpCode::Nil => {
                print_val.push_str("Nil");
                result = index + 1;
            },
            OpCode::True => {
                print_val.push_str("True");
                result = index + 1;
            },
            OpCode::False => {
                print_val.push_str("False");
                result = index + 1;
            },
            OpCode::Pop => {
                print_val.push_str("Pop");
                result = index + 1;
            },
            OpCode::GetLocal => {
                print_val.push_str("GetLocal");
                result = index + 1;
            },
            OpCode::SetLocal => {
                print_val.push_str("SetLocal");
                result = index + 1;
            },
            OpCode::GetGlobal => {
                print_val.push_str("GetGlobal");
                result = index + 1;
            },
            OpCode::DefineGlobal => {
                print_val.push_str("DefineGlobal");
                result = index + 1;
            },
            OpCode::SetGlobal => {
                print_val.push_str("SetGlobal");
                result = index + 1;
            },
            OpCode::GetUpValue => {
                print_val.push_str("GetUpValue");
                result = index + 1;
            },
            OpCode::SetUpValue => {
                print_val.push_str("SetUpValue");
                result = index + 1;
            },
            OpCode::GetProperty => {
                print_val.push_str("GetProperty");
                result = index + 1;
            },
            OpCode::SetProperty => {
                print_val.push_str("SetProperty");
                result = index + 1;
            },
            OpCode::GetSuper => {
                print_val.push_str("GetSuper");
                result = index + 1;
            },
            OpCode::Equal => {
                print_val.push_str("Equal");
                result = index + 1;
            },
            OpCode::Greater => {
                print_val.push_str("Greater");
                result = index + 1;
            },
            OpCode::Less => {
                print_val.push_str("Less");
                result = index + 1;
            },
            OpCode::Add => {
                print_val.push_str("Add");
                result = index + 1;
            },
            OpCode::Subtract => {
                print_val.push_str("Subtract");
                result = index + 1;
            },
            OpCode::Multiply => {
                print_val.push_str("Multiply");
                result = index + 1;
            },
            OpCode::Divide => {
                print_val.push_str("Divide");
                result = index + 1;
            },
            OpCode::Not => {
                print_val.push_str("Not");
                result = index + 1;
            },
            OpCode::Negate => {
                print_val.push_str("Negate");
                result = index + 1;
            },
            OpCode::Print => {
                print_val.push_str("Print");
                result = index + 1;
            },
            OpCode::Jump => {
                print_val.push_str("Jump");
                result = index + 1;
            },
            OpCode::JumpIfFalse => {
                print_val.push_str("JumpIfFalse");
                result = index + 1;
            },
            OpCode::Loop => {
                print_val.push_str("Loop");
                result = index + 1;
            },
            OpCode::Call => {
                print_val.push_str("Call");
                result = index + 1;
            },
            OpCode::Invoke => {
                print_val.push_str("Invoke");
                result = index + 1;
            },
            OpCode::SuperInvoke => {
                print_val.push_str("SuperInvoke");
                result = index + 1;
            },
            OpCode::Closure => {
                print_val.push_str("Closure");
                result = index + 1;
            },
            OpCode::CloseUpValue => {
                print_val.push_str("CloseUpValue");
                result = index + 1;
            },
            OpCode::Return => {
                print_val.push_str("RETURN");
                result = index + 1;
            },
            OpCode::Class => { 
                print_val.push_str("Class"); 
                result = index + 1; },
            OpCode::Inherit => { 
                print_val.push_str("Inherit"); 
                result = index + 1; },
            OpCode::Method => { 
                print_val.push_str("Method"); 
                result = index + 1; },
        }

        println!("{}", print_val);
        
        result
    }
}