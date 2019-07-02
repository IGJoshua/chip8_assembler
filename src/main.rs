use chip8_assembler::assemble_file;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();

    assemble_file(&args[1], &args[2]).unwrap();
}
