#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::io::{BufRead, BufReader};

pub fn assemble_file(filename: &str, output_file: &str) -> io::Result<()> {
    let file = File::open(filename)?;

    let mut bytes: Vec<u8> = Vec::new();
    for (idx, line) in BufReader::new(file).lines().enumerate() {
        let instr = parse_instruction(&line.unwrap()).unwrap();
        let assembled = assemble_instruction(instr);
        bytes.push(assembled.0);
        bytes.push(assembled.1);
    }

    let mut file = File::create(output_file)?;
    file.write_all(&bytes[..])?;

    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Vx(u8);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Addr(u16);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Instruction {
    Cls,
    Ret,
    Sys(Addr),
    Jmp(Addr),
    Call(Addr),
    SkipEq(Vx, u8),
    SkipNotEq(Vx, u8),
    SkipEqVx(Vx, Vx),
    Load(Vx, u8),
    Add(Vx, u8),
    LoadVx(Vx, Vx),
    Or(Vx, Vx),
    And(Vx, Vx),
    XOr(Vx, Vx),
    AddVx(Vx, Vx),
    SubVx(Vx, Vx),
    ShiftRight(Vx),
    SubN(Vx, Vx),
    ShiftLeft(Vx),
    SkipNotEqVx(Vx, Vx),
    LoadI(Addr),
    JmpV0(Addr),
    Rand(Vx, u8),
    Draw(Vx, Vx, u8),
    SkipKeyPressed(Vx),
    SkipKeyNotPressed(Vx),
    LoadDelay(Vx),
    LoadKey(Vx),
    SetDelay(Vx),
    SetSound(Vx),
    AddI(Vx),
    LoadFont(Vx),
    LoadBcd(Vx),
    StoreRegisters(Vx),
    LoadRegisters(Vx),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct AssembledInstruction(u8, u8);

#[derive(Debug)]
enum ParseErr {
    InvalidInstruction(String),
    IncorrectArgumentCount {
        required: u8,
        found: u8,
        msg: String,
    },
    UnimplementedInstruction(String),
}

fn parse_instruction(line: &str) -> Result<Instruction, ParseErr> {
    // TODO(Joshua): Add a way to parse all the instructions
    Err(ParseErr::UnimplementedInstruction(String::from(line)))
}

fn assemble_instruction(instr: Instruction) -> AssembledInstruction {
    // TODO(Joshua): Add a way to assemble all the instructions
    match instr {
        _ => panic!("attempted to assemble an unimplemented instruction")
    }

    fn construct_byte(high_nibble: u8, low_nibble: u8) -> u8 {
        ((high_nibble & 0x0F) << 4) | (low_nibble & 0x0F)
    }
}
