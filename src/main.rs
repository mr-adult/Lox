use std::{
    env::args, 
    fs::OpenOptions, 
    io::{
        stdout, 
        stdin, 
        Read,
        Write
    }
};

mod fixed_vec;
mod chunk;
mod tokenizer;
mod value;
mod object;
mod compiler;
mod vm;
use vm::run;

const DEBUG_TRACE_EXECUTION: bool = false;
const DEBUG_DUMP_INSTRUCTIONS: bool = false;

fn main() {
    let mut args = args();
    // program location. Throw it away.
    args.next();
    
    match args.next() {
        Some(arg) => {
            let mut file = OpenOptions::new()
                .read(true)
                .write(false)
                .open(arg)
                .expect("Failed to open file.");

            let mut code = String::new();
            file.read_to_string(&mut code).expect("Failed to read file.");
            
            run(&code);
        }
        None => {
            loop {
                let mut stdout = stdout();
                stdout.write_all("> ".as_bytes())
                    .expect("Failed to write to stdout");

                stdout.flush().expect("Failed to write to stdout");

                let mut code = String::new();
                stdin()
                    .read_line(&mut code)
                    .expect("Failed to read from stdin.");

                if code.trim().len() == 0 {
                    break;
                }

                run(&code);
            }
        }
    }
}