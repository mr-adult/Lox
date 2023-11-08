use std::{collections::{HashMap, HashSet}, process::ExitCode};

use crate::{ 
    fixed_vec::FixedVec, 
    chunk::{Chunk, OpCode}, 
    DEBUG_TRACE_EXECUTION, 
    DEBUG_DUMP_INSTRUCTIONS,
    value::Value
};

pub (crate) fn run<'i>(_program: &'i str) -> ExitCode {
    let mut vm = VM::new(vec![
        OpCode::Constant as u8,
        0,
        OpCode::Return as u8,
    ].into_iter()
    .map(|code| Chunk { op: code, line: 123 })
    .collect());
    vm.values.push(Value::Number(2.31));
    match vm.run() {
        Ok(()) => return ExitCode::SUCCESS,
        Err(err) => {
            match err {
                VMErr::CompileErr => return ExitCode::from(65),
                VMErr::RuntimeErr(_) => return ExitCode::from(70),
                VMErr::Panic(_) => return ExitCode::FAILURE,
                VMErr::OutOfIterations => return ExitCode::SUCCESS, 
            }
        }
    }
}

const FRAMES_MAX: usize = 64;
const STACK_MAX: usize = u8::MAX as usize; // FRAMES_MAX as usize * u8::MAX as usize;
const U8_MAX: usize = u8::MAX as usize;

pub (crate) enum VMErr {
    CompileErr,
    RuntimeErr(RunTimeErr),
    Panic(String),
    OutOfIterations,
}

pub (crate) enum RunTimeErr {
    ArithmeticOnNonNumber,
}

struct VM {
    code: Vec<Chunk>,
    values: FixedVec<Value, STACK_MAX>,
    ip: usize,
}

impl VM {
    fn new(code: Vec<Chunk>) -> Self {
        Self {
            code,
            ip: 0,
            values: FixedVec::new(),
        }
    }

    fn run(&mut self) -> Result<(), VMErr> {
        if DEBUG_DUMP_INSTRUCTIONS {
            Chunk::disassemble_code(&self.code, &self.values, "code");
        }

        for _ in 0..1_000_000 { // put an upper limit of 1M iterations
            let op_result = OpCode::try_from(self.code[self.ip].op);
            match op_result {
                Err(msg) => {
                    println!("{}", msg);
                    return Err(VMErr::Panic(msg));
                }
                Ok(op) => {
                    if DEBUG_TRACE_EXECUTION {
                        let mut stack_str = String::new();
                        stack_str.push_str("          ");
                        for slot in self.values.iter() {
                            stack_str.push_str("[ ");
                            stack_str.push_str(&slot.to_string());
                            stack_str.push_str(" ]");
                        }
                        println!("{}", stack_str);

                        Chunk::disassemble_instruction(&self.code, self.ip, &self.values);
                    }

                    match op {
                        OpCode::Constant => {
                            let constant = self.read_constant();
                            println!("{}", constant.to_string());
                            self.ip += 1;
                        },
                        OpCode::Nil => todo!(),
                        OpCode::True => todo!(),
                        OpCode::False => todo!(),
                        OpCode::Pop => todo!(),
                        OpCode::GetLocal => todo!(),
                        OpCode::SetLocal => todo!(),
                        OpCode::GetGlobal => todo!(),
                        OpCode::DefineGlobal => todo!(),
                        OpCode::SetGlobal => todo!(),
                        OpCode::GetUpValue => todo!(),
                        OpCode::SetUpValue => todo!(),
                        OpCode::GetProperty => todo!(),
                        OpCode::SetProperty => todo!(),
                        OpCode::GetSuper => todo!(),
                        OpCode::Equal => todo!(),
                        OpCode::Greater => todo!(),
                        OpCode::Less => todo!(),
                        OpCode::Add => {
                            if let Value::Number(b) = self.pop_value() {
                                if let Value::Number(a) = self.pop_value() {
                                    self.push_value(Value::Number(a + b));
                                } else {
                                    return Err(VMErr::RuntimeErr(RunTimeErr::ArithmeticOnNonNumber));
                                }
                            } else {
                                return Err(VMErr::RuntimeErr(RunTimeErr::ArithmeticOnNonNumber));
                            }
                        },
                        OpCode::Subtract => {
                            if let Value::Number(b) = self.pop_value() {
                                if let Value::Number(a) = self.pop_value() {
                                    self.push_value(Value::Number(a - b));
                                } else {
                                    return Err(VMErr::RuntimeErr(RunTimeErr::ArithmeticOnNonNumber));
                                }
                            } else {
                                return Err(VMErr::RuntimeErr(RunTimeErr::ArithmeticOnNonNumber));
                            }
                        },
                        OpCode::Multiply => {
                            if let Value::Number(b) = self.pop_value() {
                                if let Value::Number(a) = self.pop_value() {
                                    self.push_value(Value::Number(a * b));
                                } else {
                                    return Err(VMErr::RuntimeErr(RunTimeErr::ArithmeticOnNonNumber));
                                }
                            } else {
                                return Err(VMErr::RuntimeErr(RunTimeErr::ArithmeticOnNonNumber));
                            }
                        },
                        OpCode::Divide => {
                            if let Value::Number(b) = self.pop_value() {
                                if let Value::Number(a) = self.pop_value() {
                                    self.push_value(Value::Number(a / b));
                                } else {
                                    return Err(VMErr::RuntimeErr(RunTimeErr::ArithmeticOnNonNumber));
                                }
                            } else {
                                return Err(VMErr::RuntimeErr(RunTimeErr::ArithmeticOnNonNumber));
                            }
                        },
                        OpCode::Not => todo!(),
                        OpCode::Negate => {
                            if let Value::Number(num) = self.pop_value() {
                                self.push_value(Value::Number(-num));
                            } else {
                                return Err(VMErr::RuntimeErr(RunTimeErr::ArithmeticOnNonNumber));
                            }
                        },
                        OpCode::Print => todo!(),
                        OpCode::Jump => todo!(),
                        OpCode::JumpIfFalse => todo!(),
                        OpCode::Loop => todo!(),
                        OpCode::Call => todo!(),
                        OpCode::Invoke => todo!(),
                        OpCode::SuperInvoke => todo!(),
                        OpCode::Closure => todo!(),
                        OpCode::CloseUpValue => todo!(),
                        OpCode::Return => return Ok(()),
                        OpCode::Class => todo!(),
                        OpCode::Inherit => todo!(),
                        OpCode::Method => todo!(),
                    }
                }
            }
        }

        return Err(VMErr::OutOfIterations);
    }

    fn read_constant(&mut self) -> Value {
        let value = 
            self.values
                .get(self.ip)
                .expect("Value pointed to to be populated.")
                .clone();

        self.ip += 1;
        value
    }

    fn push_value(&mut self, val: Value) {
        self.values.push(val);
    }

    fn pop_value(&mut self) -> Value {
        self.values
            .pop()
            .expect("Popped value to be populated.")
    }
}