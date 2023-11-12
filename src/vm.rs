use std::process::ExitCode;

use crate::{ 
    fixed_vec::FixedVec, 
    chunk::{Chunk, OpCode}, 
    DEBUG_TRACE_EXECUTION, 
    DEBUG_DUMP_INSTRUCTIONS,
    value::Value, 
    compiler::compile, object::Object
};

pub (crate) fn run<'i>(program: &'i str) -> ExitCode {
    let result;
    match compile(program) {
        Err(errs) => {
            for err in errs {
                println!("{}", err);
            }
            return ExitCode::from(65);
        }
        Ok(chunks) => result = chunks,
    }

    match VM::new(result.0, result.1).run() {
        Ok(()) => return ExitCode::SUCCESS,
        Err(err) => {
            match err {
                VMErr::RuntimeErr(_) => return ExitCode::from(70),
                VMErr::Panic(_) => return ExitCode::FAILURE,
                VMErr::OutOfIterations => return ExitCode::SUCCESS, 
            }
        }
    }
}

const FRAMES_MAX: usize = 64;
pub (crate) const STACK_MAX: usize = u8::MAX as usize; // FRAMES_MAX as usize * u8::MAX as usize;
const U8_MAX: usize = u8::MAX as usize;

pub (crate) enum VMErr {
    RuntimeErr(RunTimeErr),
    Panic(String),
    OutOfIterations,
}

pub (crate) struct RunTimeErr {
    line: usize,
    kind: RunTimeErrKind,
}

pub (crate) enum RunTimeErrKind {
    ArithmeticOnNonNumber,
    ComparisonOnNonNumber,
    BooleanOperationOnObject,
    BooleanOperationOnNumber,
}

struct VM {
    code: Vec<Chunk>,
    compiled_values: FixedVec<Value, STACK_MAX>,
    runtime_values: FixedVec<Value, STACK_MAX>,
    ip: usize,
}

impl VM {
    fn new(code: Vec<Chunk>, values: FixedVec<Value, STACK_MAX>) -> Self {
        Self {
            code,
            ip: 0,
            compiled_values: values,
            runtime_values: FixedVec::<Value, STACK_MAX>::new(),
        }
    }

    fn run(&mut self) -> Result<(), VMErr> {
        if DEBUG_DUMP_INSTRUCTIONS {
            Chunk::disassemble_code(&self.code, &self.compiled_values, "code");
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
                        for slot in self.runtime_values.iter() {
                            stack_str.push_str("[ ");
                            stack_str.push_str(&slot.to_string());
                            stack_str.push_str(" ]");
                        }
                        println!("{}", stack_str);
                        Chunk::disassemble_instruction(&self.code, self.ip, &self.compiled_values);
                    }

                    match op {
                        OpCode::Constant => {
                            self.ip += 1;
                            self.read_constant();
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
                        OpCode::Equal => {
                            match self.pop_value() {
                                Value::Nil => {
                                    if let Value::Nil = self.pop_value() {
                                        self.push_value(Value::Boolean(true))
                                    } else {
                                        self.push_value(Value::Boolean(false))
                                    }
                                },
                                Value::Boolean(b) => {
                                    if let Value::Boolean(a) = self.pop_value() {
                                        self.push_value(Value::Boolean(a == b))
                                    } else {
                                        self.push_value(Value::Boolean(false))
                                    }
                                },
                                Value::Number(b) => {
                                    if let Value::Number(a) = self.pop_value() {
                                        self.push_value(Value::Boolean(a == b))
                                    } else {
                                        self.push_value(Value::Boolean(false))
                                    }
                                },
                                Value::Object(b) => {
                                    if let Value::Object(a) = self.pop_value() {
                                        // check reference equality.
                                        self.push_value(Value::Boolean(&*a as *const Object == &*b as *const Object))
                                    }
                                },
                            }
                            self.ip += 1;
                        },
                        OpCode::Greater => {
                            if let Value::Number(b) = self.pop_value() {
                                if let Value::Number(a) = self.pop_value() {
                                    self.push_value(Value::Boolean(a > b))
                                } else {
                                    return Err(self.runtime_err(RunTimeErrKind::ComparisonOnNonNumber));
                                }
                            } else {
                                return Err(self.runtime_err(RunTimeErrKind::ComparisonOnNonNumber));
                            }
                            self.ip += 1;
                        },
                        OpCode::Less => {
                            if let Value::Number(b) = self.pop_value() {
                                if let Value::Number(a) = self.pop_value() {
                                    self.push_value(Value::Boolean(a < b))
                                } else {
                                    return Err(self.runtime_err(RunTimeErrKind::ComparisonOnNonNumber));
                                }
                            } else {
                                return Err(self.runtime_err(RunTimeErrKind::ComparisonOnNonNumber));
                            }
                            self.ip += 1;
                        },
                        OpCode::Add => {
                            if let Value::Number(b) = self.pop_value() {
                                if let Value::Number(a) = self.pop_value() {
                                    self.push_value(Value::Number(a + b));
                                } else {
                                    return Err(self.runtime_err(RunTimeErrKind::ArithmeticOnNonNumber));
                                }
                            } else {
                                return Err(self.runtime_err(RunTimeErrKind::ArithmeticOnNonNumber));
                            }
                            self.ip += 1;
                        },
                        OpCode::Subtract => {
                            if let Value::Number(b) = self.pop_value() {
                                if let Value::Number(a) = self.pop_value() {
                                    self.push_value(Value::Number(a - b));
                                } else {
                                    return Err(self.runtime_err(RunTimeErrKind::ArithmeticOnNonNumber));
                                }
                            } else {
                                return Err(self.runtime_err(RunTimeErrKind::ArithmeticOnNonNumber));
                            }
                            self.ip += 1;
                        },
                        OpCode::Multiply => {
                            if let Value::Number(b) = self.pop_value() {
                                if let Value::Number(a) = self.pop_value() {
                                    self.push_value(Value::Number(a * b));
                                } else {
                                    return Err(self.runtime_err(RunTimeErrKind::ArithmeticOnNonNumber));
                                }
                            } else {
                                return Err(self.runtime_err(RunTimeErrKind::ArithmeticOnNonNumber));
                            }
                            self.ip += 1;
                        },
                        OpCode::Divide => {
                            if let Value::Number(b) = self.pop_value() {
                                if let Value::Number(a) = self.pop_value() {
                                    self.push_value(Value::Number(a / b));
                                } else {
                                    return Err(self.runtime_err(RunTimeErrKind::ArithmeticOnNonNumber));
                                }
                            } else {
                                return Err(self.runtime_err(RunTimeErrKind::ArithmeticOnNonNumber));
                            }
                            self.ip += 1;
                        },
                        OpCode::Not => {
                            match self.pop_value() {
                                Value::Nil => self.push_value(Value::Boolean(true)),
                                Value::Boolean(bool) => self.push_value(Value::Boolean(!bool)),
                                other => {
                                    match other {
                                        Value::Number(_) => return Err(self.runtime_err(RunTimeErrKind::BooleanOperationOnNumber)),
                                        Value::Object(_) => return Err(self.runtime_err(RunTimeErrKind::BooleanOperationOnObject)),
                                        Value::Boolean(_) | Value::Nil => {}
                                    }
                                }
                            }
                            self.ip += 1;
                        },
                        OpCode::Negate => {
                            if let Value::Number(num) = self.pop_value() {
                                self.push_value(Value::Number(-num));
                            } else {
                                return Err(self.runtime_err(RunTimeErrKind::ArithmeticOnNonNumber));
                            }
                            self.ip += 1;
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
                        OpCode::Return => {
                            println!("{}", self.runtime_values.get(0).unwrap_or(&Value::Nil).to_string());
                            return Ok(())
                        },
                        OpCode::Class => todo!(),
                        OpCode::Inherit => todo!(),
                        OpCode::Method => todo!(),
                        OpCode::Unknown => panic!("Found unknown OpCode"),
                    }
                }
            }
        }

        return Err(VMErr::OutOfIterations);
    }

    fn read_constant(&mut self) {
        let value = 
            self.compiled_values
                .get(self.code[self.ip].op as usize)
                .expect("Value pointed to to be populated.")
                .clone();

        self.runtime_values.push(value);
        self.ip += 1;
    }

    fn push_value(&mut self, val: Value) {
        self.runtime_values.push(val);
    }

    fn pop_value(&mut self) -> Value {
        self.runtime_values
            .pop()
            .expect("Popped value to be populated.")
    }

    fn runtime_err(&self, kind: RunTimeErrKind) -> VMErr {
        VMErr::RuntimeErr(
            RunTimeErr { 
                line: self.code[self.ip].line, 
                kind,
            }
        )
    }
}