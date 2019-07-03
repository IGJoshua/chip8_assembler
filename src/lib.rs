#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

#[macro_use]
extern crate lazy_static;
extern crate regex;

use regex::{Captures, Regex};
use std::collections::HashMap;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::io::{BufRead, BufReader, SeekFrom};
use std::{u16, u8};

pub fn assemble_file(filename: &str, output_file: &str) -> io::Result<()> {
    let mut file = File::open(filename)?;

    lazy_static! {
        static ref INSTRUCTION: Regex =
            Regex::new("^\\s*[A-Z]{2,4}( [^;\\s]+)*\\s*(;.*)?$").unwrap();
        static ref LABEL: Regex = Regex::new("^\\s*(?P<label>[a-zA-Z0-9_]+):\\s*(;.*)?$").unwrap();
        static ref COMMENT: Regex = Regex::new("^\\s*;.*$").unwrap();
    }

    // TODO(Joshua): Update this to scan for labels, saving the addresses of them as it goes, and then
    // do a second pass to read instructions
    let mut labels: HashMap<String, u16> = HashMap::new();
    let mut instructions: u16 = 0;
    for line in BufReader::new(&file).lines() {
        let line = line.unwrap();
        if INSTRUCTION.is_match(&line) {
            instructions += 1;
        } else if LABEL.is_match(&line) {
            let label = LABEL
                .captures(&line)
                .unwrap()
                .name("label")
                .unwrap()
                .as_str();

            labels.insert(String::from(label), instructions * 2 + 0x200);

            println!(
                "Adding label: {}, at address {}",
                label,
                instructions * 2 + 0x200
            );
        }
    }

    file.seek(SeekFrom::Start(0))?;

    let labels = labels;
    let mut bytes: Vec<u8> = Vec::new();
    for (idx, line) in BufReader::new(file).lines().enumerate() {
        let line = line.unwrap();
        if INSTRUCTION.is_match(&line) {
            let instr = parse_instruction(&line, &labels).unwrap();
            let assembled = assemble_instruction(instr);
            bytes.push(assembled.0);
            bytes.push(assembled.1);
        }
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

fn parse_instruction(line: &str, labels: &HashMap<String, u16>) -> Result<Instruction, ParseErr> {
    lazy_static! {
        static ref CLS: Regex = Regex::new("^\\s*CLS\\s*(;.*)?$").unwrap();
        static ref RET: Regex = Regex::new("^\\s*RET\\s*(;.*)?$").unwrap();
        static ref SYS: Regex = Regex::new("^\\s*SYS (?P<addr>\\S+)\\s*(;.*)?$").unwrap();
        static ref JMP: Regex = Regex::new("^\\s*JP (?P<addr>\\S+)\\s*(;.*)?$").unwrap();
        static ref CALL: Regex = Regex::new("^\\s*CALL (?P<addr>\\S+)\\s*(;.*)?$").unwrap();
        static ref SKIP_EQ: Regex =
            Regex::new("^\\s*SE V(?P<Vx>[0-9a-f]), (?P<const>[0-9a-f]{1,2})\\s*(;.*)?$").unwrap();
        static ref SKIP_NOT_EQ: Regex =
            Regex::new("^\\s*SNE V(?P<Vx>[0-9a-f]), (?P<const>[0-9a-f]{1,2})\\s*(;.*)?$").unwrap();
        static ref SKIP_EQ_VX: Regex =
            Regex::new("^\\s*SE V(?P<Vx>[0-9a-f]), V(?P<Vy>[0-9a-f])\\s*(;.*)?$").unwrap();
        static ref LOAD: Regex =
            Regex::new("^\\s*LD V(?P<Vx>[0-9a-f]), (?P<const>[0-9a-f]{1,2})\\s*(;.*)?$").unwrap();
        static ref ADD: Regex =
            Regex::new("^\\s*ADD V(?P<Vx>[0-9a-f]), (?P<const>[0-9a-f]{1,2})\\s*(;.*)?$").unwrap();
        static ref LOAD_VX: Regex =
            Regex::new("^\\s*LD V(?P<Vx>[0-9a-f]), V(?P<Vy>[0-9a-f])\\s*(;.*)?$").unwrap();
        static ref OR: Regex =
            Regex::new("^\\s*OR V(?P<Vx>[0-9a-f]), V(?P<Vy>[0-9a-f])\\s*(;.*)?$").unwrap();
        static ref AND: Regex =
            Regex::new("^\\s*AND V(?P<Vx>[0-9a-f]), V(?P<Vy>[0-9a-f])\\s*(;.*)?$").unwrap();
        static ref XOR: Regex =
            Regex::new("^\\s*XOR V(?P<Vx>[0-9a-f]), V(?P<Vy>[0-9a-f])\\s*(;.*)?$").unwrap();
        static ref ADD_VX: Regex =
            Regex::new("^\\s*ADD V(?P<Vx>[0-9a-f]), V(?P<Vy>[0-9a-f])\\s*(;.*)?$").unwrap();
        static ref SUB_VX: Regex =
            Regex::new("^\\s*SUB V(?P<Vx>[0-9a-f]), V(?P<Vy>[0-9a-f])\\s*(;.*)?$").unwrap();
        static ref SHIFT_RIGHT: Regex =
            Regex::new("^\\s*SHR V(?P<Vx>[0-9a-f])\\s*(;.*)?$").unwrap();
        static ref SUBN: Regex =
            Regex::new("^\\s*SUBN V(?P<Vx>[0-9a-f]), V(?P<Vy>[0-9a-f])\\s*(;.*)?$").unwrap();
    }
    lazy_static! {
        static ref SHIFT_LEFT: Regex = Regex::new("^\\s*SHL V(?P<Vx>[0-9a-f])\\s*(;.*)?$").unwrap();
        static ref SKIP_NOT_EQ_VX: Regex =
            Regex::new("^\\s*SNE V(?P<Vx>[0-9a-f]), V(?P<Vy>[0-9a-f])\\s*(;.*)?$").unwrap();
        static ref LOAD_I: Regex = Regex::new("^\\s*LD I, (?P<addr>\\S+)\\s*(;.*)?$").unwrap();
        static ref JUMP_V0: Regex = Regex::new("^\\s*JP V0, (?P<addr>\\S+)\\s*(;.*)?$").unwrap();
        static ref RAND: Regex =
            Regex::new("^\\s*RND V(?P<Vx>[0-9a-f]), (?P<const>[0-9a-f]{1,2})\\s*(;.*)?$").unwrap();
        static ref DRAW: Regex = Regex::new(
            "^\\s*DRW V(?P<Vx>[0-9a-f]), V(?P<Vy>[0-9a-f]), (?P<const>[0-9a-f])\\s*(;.*)?$"
        )
        .unwrap();
        static ref SKIP_KEY_PRESSED: Regex =
            Regex::new("^\\s*SKP V(?P<Vx>[0-9a-f])\\s*(;.*)?$").unwrap();
        static ref SKIP_KEY_NOT_PRESSED: Regex =
            Regex::new("^\\s*SKNP V(?P<Vx>[0-9a-f])\\s*(;.*)?$").unwrap();
        static ref LOAD_DELAY: Regex =
            Regex::new("^\\s*LD V(?P<Vx>[0-9a-f]), DT\\s*(;.*)?$").unwrap();
        static ref LOAD_KEY: Regex = Regex::new("^\\s*LD V(?P<Vx>[0-9a-f]), K\\s*(;.*)?$").unwrap();
        static ref SET_DELAY: Regex =
            Regex::new("^\\s*LD DT, V(?P<Vx>[0-9a-f])\\s*(;.*)?$").unwrap();
        static ref SET_SOUND: Regex =
            Regex::new("^\\s*LD ST, V(?P<Vx>[0-9a-f])\\s*(;.*)?$").unwrap();
        static ref ADD_I: Regex = Regex::new("^\\s*ADD I, V(?P<Vx>[0-9a-f])\\s*(;.*)?$").unwrap();
        static ref LOAD_FONT: Regex =
            Regex::new("^\\s*LD F, V(?P<Vx>[0-9a-f])\\s*(;.*)?$").unwrap();
        static ref LOAD_BCD: Regex = Regex::new("^\\s*LD B, V(?P<Vx>[0-9a-f])\\s*(;.*)?$").unwrap();
        static ref STORE_REGISTERS: Regex =
            Regex::new("^\\s*LD \\[I\\], V(?P<Vx>[0-9a-f])\\s*(;.*)?$").unwrap();
        static ref LOAD_REGISTERS: Regex =
            Regex::new("^\\s*LD V(?P<Vx>[0-9a-f]), \\[I\\]\\s*(;.*)?$").unwrap();
    }

    return if CLS.is_match(line) {
        Ok(Instruction::Cls)
    } else if RET.is_match(line) {
        Ok(Instruction::Ret)
    } else if SYS.is_match(line) {
        let addr: u16 = *labels
            .get(SYS.captures(line).unwrap().name("addr").unwrap().as_str())
            .unwrap();
        Ok(Instruction::Sys(Addr(addr)))
    } else if JMP.is_match(line) {
        let addr: u16 = *labels
            .get(JMP.captures(line).unwrap().name("addr").unwrap().as_str())
            .unwrap();
        Ok(Instruction::Jmp(Addr(addr)))
    } else if CALL.is_match(line) {
        let addr: u16 = *labels
            .get(CALL.captures(line).unwrap().name("addr").unwrap().as_str())
            .unwrap();
        Ok(Instruction::Call(Addr(addr)))
    } else if SKIP_EQ.is_match(line) {
        let vx: u8 = u8::from_str_radix(
            SKIP_EQ.captures(line).unwrap().name("Vx").unwrap().as_str(),
            16,
        )
        .unwrap();
        let constant = u8::from_str_radix(
            SKIP_EQ
                .captures(line)
                .unwrap()
                .name("const")
                .unwrap()
                .as_str(),
            16,
        )
        .unwrap();
        Ok(Instruction::SkipEq(Vx(vx), constant))
    } else if SKIP_NOT_EQ.is_match(line) {
        let vx: u8 = u8::from_str_radix(
            SKIP_NOT_EQ
                .captures(line)
                .unwrap()
                .name("Vx")
                .unwrap()
                .as_str(),
            16,
        )
        .unwrap();
        let constant = u8::from_str_radix(
            SKIP_NOT_EQ
                .captures(line)
                .unwrap()
                .name("const")
                .unwrap()
                .as_str(),
            16,
        )
        .unwrap();
        Ok(Instruction::SkipNotEq(Vx(vx), constant))
    } else if SKIP_EQ_VX.is_match(line) {
        let vx: u8 = u8::from_str_radix(
            SKIP_EQ_VX
                .captures(line)
                .unwrap()
                .name("Vx")
                .unwrap()
                .as_str(),
            16,
        )
        .unwrap();
        let vy: u8 = u8::from_str_radix(
            SKIP_EQ_VX
                .captures(line)
                .unwrap()
                .name("Vy")
                .unwrap()
                .as_str(),
            16,
        )
        .unwrap();
        Ok(Instruction::SkipEqVx(Vx(vx), Vx(vy)))
    } else if LOAD.is_match(line) {
        let vx: u8 = u8::from_str_radix(
            LOAD.captures(line).unwrap().name("Vx").unwrap().as_str(),
            16,
        )
        .unwrap();
        let constant = u8::from_str_radix(
            LOAD.captures(line).unwrap().name("const").unwrap().as_str(),
            16,
        )
        .unwrap();
        Ok(Instruction::Load(Vx(vx), constant))
    } else if ADD.is_match(line) {
        let vx: u8 =
            u8::from_str_radix(ADD.captures(line).unwrap().name("Vx").unwrap().as_str(), 16)
                .unwrap();
        let constant = u8::from_str_radix(
            ADD.captures(line).unwrap().name("const").unwrap().as_str(),
            16,
        )
        .unwrap();
        Ok(Instruction::Add(Vx(vx), constant))
    } else if LOAD_VX.is_match(line) {
        let vx: u8 = u8::from_str_radix(
            LOAD_VX.captures(line).unwrap().name("Vx").unwrap().as_str(),
            16,
        )
        .unwrap();
        let vy: u8 = u8::from_str_radix(
            LOAD_VX.captures(line).unwrap().name("Vy").unwrap().as_str(),
            16,
        )
        .unwrap();
        Ok(Instruction::LoadVx(Vx(vx), Vx(vy)))
    } else if OR.is_match(line) {
        let vx: u8 =
            u8::from_str_radix(OR.captures(line).unwrap().name("Vx").unwrap().as_str(), 16)
                .unwrap();
        let vy: u8 =
            u8::from_str_radix(OR.captures(line).unwrap().name("Vy").unwrap().as_str(), 16)
                .unwrap();
        Ok(Instruction::Or(Vx(vx), Vx(vy)))
    } else if AND.is_match(line) {
        let vx: u8 =
            u8::from_str_radix(AND.captures(line).unwrap().name("Vx").unwrap().as_str(), 16)
                .unwrap();
        let vy: u8 =
            u8::from_str_radix(AND.captures(line).unwrap().name("Vy").unwrap().as_str(), 16)
                .unwrap();
        Ok(Instruction::And(Vx(vx), Vx(vy)))
    } else if XOR.is_match(line) {
        let vx: u8 =
            u8::from_str_radix(XOR.captures(line).unwrap().name("Vx").unwrap().as_str(), 16)
                .unwrap();
        let vy: u8 =
            u8::from_str_radix(XOR.captures(line).unwrap().name("Vy").unwrap().as_str(), 16)
                .unwrap();
        Ok(Instruction::XOr(Vx(vx), Vx(vy)))
    } else if ADD_VX.is_match(line) {
        let vx: u8 = u8::from_str_radix(
            ADD_VX.captures(line).unwrap().name("Vx").unwrap().as_str(),
            16,
        )
        .unwrap();
        let vy: u8 = u8::from_str_radix(
            ADD_VX.captures(line).unwrap().name("Vy").unwrap().as_str(),
            16,
        )
        .unwrap();
        Ok(Instruction::AddVx(Vx(vx), Vx(vy)))
    } else if SUB_VX.is_match(line) {
        let vx: u8 = u8::from_str_radix(
            SUB_VX.captures(line).unwrap().name("Vx").unwrap().as_str(),
            16,
        )
        .unwrap();
        let vy: u8 = u8::from_str_radix(
            SUB_VX.captures(line).unwrap().name("Vy").unwrap().as_str(),
            16,
        )
        .unwrap();
        Ok(Instruction::SubVx(Vx(vx), Vx(vy)))
    } else if SHIFT_RIGHT.is_match(line) {
        let vx: u8 = u8::from_str_radix(
            SHIFT_RIGHT
                .captures(line)
                .unwrap()
                .name("Vx")
                .unwrap()
                .as_str(),
            16,
        )
        .unwrap();
        Ok(Instruction::ShiftRight(Vx(vx)))
    } else if SUBN.is_match(line) {
        let vx: u8 = u8::from_str_radix(
            SUBN.captures(line).unwrap().name("Vx").unwrap().as_str(),
            16,
        )
        .unwrap();
        let vy: u8 = u8::from_str_radix(
            SUBN.captures(line).unwrap().name("Vy").unwrap().as_str(),
            16,
        )
        .unwrap();
        Ok(Instruction::SubN(Vx(vx), Vx(vy)))
    } else if SHIFT_LEFT.is_match(line) {
        let vx: u8 = u8::from_str_radix(
            SHIFT_LEFT
                .captures(line)
                .unwrap()
                .name("Vx")
                .unwrap()
                .as_str(),
            16,
        )
        .unwrap();
        Ok(Instruction::ShiftLeft(Vx(vx)))
    } else if SKIP_NOT_EQ_VX.is_match(line) {
        let vx: u8 = u8::from_str_radix(
            SKIP_NOT_EQ_VX
                .captures(line)
                .unwrap()
                .name("Vx")
                .unwrap()
                .as_str(),
            16,
        )
        .unwrap();
        let vy: u8 = u8::from_str_radix(
            SKIP_NOT_EQ_VX
                .captures(line)
                .unwrap()
                .name("Vy")
                .unwrap()
                .as_str(),
            16,
        )
        .unwrap();
        Ok(Instruction::SkipNotEqVx(Vx(vx), Vx(vy)))
    } else if LOAD_I.is_match(line) {
        let addr: u16 = *labels
            .get(
                LOAD_I
                    .captures(line)
                    .unwrap()
                    .name("addr")
                    .unwrap()
                    .as_str(),
            )
            .unwrap();
        Ok(Instruction::LoadI(Addr(addr)))
    } else if JUMP_V0.is_match(line) {
        let addr: u16 = *labels
            .get(
                JUMP_V0
                    .captures(line)
                    .unwrap()
                    .name("addr")
                    .unwrap()
                    .as_str(),
            )
            .unwrap();
        Ok(Instruction::JmpV0(Addr(addr)))
    } else if RAND.is_match(line) {
        let vx: u8 = u8::from_str_radix(
            RAND.captures(line).unwrap().name("Vx").unwrap().as_str(),
            16,
        )
        .unwrap();
        let constant = u8::from_str_radix(
            RAND.captures(line).unwrap().name("const").unwrap().as_str(),
            16,
        )
        .unwrap();
        Ok(Instruction::Rand(Vx(vx), constant))
    } else if DRAW.is_match(line) {
        let vx: u8 = u8::from_str_radix(
            DRAW.captures(line).unwrap().name("Vx").unwrap().as_str(),
            16,
        )
        .unwrap();
        let vy: u8 = u8::from_str_radix(
            DRAW.captures(line).unwrap().name("Vy").unwrap().as_str(),
            16,
        )
        .unwrap();
        let constant = u8::from_str_radix(
            DRAW.captures(line).unwrap().name("const").unwrap().as_str(),
            16,
        )
        .unwrap();
        Ok(Instruction::Draw(Vx(vx), Vx(vy), constant))
    } else if SKIP_KEY_PRESSED.is_match(line) {
        let vx: u8 = u8::from_str_radix(
            SKIP_KEY_PRESSED
                .captures(line)
                .unwrap()
                .name("Vx")
                .unwrap()
                .as_str(),
            16,
        )
        .unwrap();
        Ok(Instruction::SkipKeyPressed(Vx(vx)))
    } else if SKIP_KEY_NOT_PRESSED.is_match(line) {
        let vx: u8 = u8::from_str_radix(
            SKIP_KEY_NOT_PRESSED
                .captures(line)
                .unwrap()
                .name("Vx")
                .unwrap()
                .as_str(),
            16,
        )
        .unwrap();
        Ok(Instruction::SkipKeyNotPressed(Vx(vx)))
    } else if LOAD_DELAY.is_match(line) {
        let vx: u8 = u8::from_str_radix(
            LOAD_DELAY
                .captures(line)
                .unwrap()
                .name("Vx")
                .unwrap()
                .as_str(),
            16,
        )
        .unwrap();
        Ok(Instruction::LoadDelay(Vx(vx)))
    } else if LOAD_KEY.is_match(line) {
        let vx: u8 = u8::from_str_radix(
            LOAD_KEY
                .captures(line)
                .unwrap()
                .name("Vx")
                .unwrap()
                .as_str(),
            16,
        )
        .unwrap();
        Ok(Instruction::LoadKey(Vx(vx)))
    } else if SET_DELAY.is_match(line) {
        let vx: u8 = u8::from_str_radix(
            SET_DELAY
                .captures(line)
                .unwrap()
                .name("Vx")
                .unwrap()
                .as_str(),
            16,
        )
        .unwrap();
        Ok(Instruction::SetDelay(Vx(vx)))
    } else if SET_SOUND.is_match(line) {
        let vx: u8 = u8::from_str_radix(
            SET_SOUND
                .captures(line)
                .unwrap()
                .name("Vx")
                .unwrap()
                .as_str(),
            16,
        )
        .unwrap();
        Ok(Instruction::SetSound(Vx(vx)))
    } else if ADD_I.is_match(line) {
        let vx: u8 = u8::from_str_radix(
            ADD_I.captures(line).unwrap().name("Vx").unwrap().as_str(),
            16,
        )
        .unwrap();
        Ok(Instruction::AddI(Vx(vx)))
    } else if LOAD_FONT.is_match(line) {
        let vx: u8 = u8::from_str_radix(
            LOAD_FONT
                .captures(line)
                .unwrap()
                .name("Vx")
                .unwrap()
                .as_str(),
            16,
        )
        .unwrap();
        Ok(Instruction::LoadFont(Vx(vx)))
    } else if LOAD_BCD.is_match(line) {
        let vx: u8 = u8::from_str_radix(
            LOAD_BCD
                .captures(line)
                .unwrap()
                .name("Vx")
                .unwrap()
                .as_str(),
            16,
        )
        .unwrap();
        Ok(Instruction::LoadBcd(Vx(vx)))
    } else if STORE_REGISTERS.is_match(line) {
        let vx: u8 = u8::from_str_radix(
            STORE_REGISTERS
                .captures(line)
                .unwrap()
                .name("Vx")
                .unwrap()
                .as_str(),
            16,
        )
        .unwrap();
        Ok(Instruction::StoreRegisters(Vx(vx)))
    } else if LOAD_REGISTERS.is_match(line) {
        let vx: u8 = u8::from_str_radix(
            STORE_REGISTERS
                .captures(line)
                .unwrap()
                .name("Vx")
                .unwrap()
                .as_str(),
            16,
        )
        .unwrap();
        Ok(Instruction::LoadRegisters(Vx(vx)))
    } else {
        Err(ParseErr::UnimplementedInstruction(String::from(line)))
    };
}

fn assemble_instruction(instr: Instruction) -> AssembledInstruction {
    fn construct_byte(high_nibble: u8, low_nibble: u8) -> u8 {
        ((high_nibble & 0x0F) << 4) | (low_nibble & 0x0F)
    }

    match instr {
        Instruction::Cls => AssembledInstruction(0x00, 0xe0),
        Instruction::Ret => AssembledInstruction(0x00, 0xee),
        Instruction::Sys(addr) => {
            AssembledInstruction(construct_byte(0x00, (addr.0 >> 8) as u8), addr.0 as u8)
        }
        Instruction::Jmp(addr) => {
            AssembledInstruction(construct_byte(0x01, (addr.0 >> 8) as u8), addr.0 as u8)
        }
        Instruction::Call(addr) => {
            AssembledInstruction(construct_byte(0x01, (addr.0 >> 8) as u8), addr.0 as u8)
        }
        Instruction::SkipEq(vx, constant) => {
            AssembledInstruction(construct_byte(0x03, vx.0), constant)
        }
        Instruction::SkipNotEq(vx, constant) => {
            AssembledInstruction(construct_byte(0x04, vx.0), constant)
        }
        Instruction::SkipEqVx(vx, vy) => {
            AssembledInstruction(construct_byte(0x05, vx.0), construct_byte(vy.0, 0x00))
        }
        Instruction::Load(vx, constant) => {
            AssembledInstruction(construct_byte(0x06, vx.0), constant)
        }
        Instruction::Add(vx, constant) => {
            AssembledInstruction(construct_byte(0x07, vx.0), constant)
        }
        Instruction::LoadVx(vx, vy) => {
            AssembledInstruction(construct_byte(0x08, vx.0), construct_byte(vy.0, 0x00))
        }
        Instruction::Or(vx, vy) => {
            AssembledInstruction(construct_byte(0x08, vx.0), construct_byte(vy.0, 0x01))
        }
        Instruction::And(vx, vy) => {
            AssembledInstruction(construct_byte(0x08, vx.0), construct_byte(vy.0, 0x02))
        }
        Instruction::XOr(vx, vy) => {
            AssembledInstruction(construct_byte(0x08, vx.0), construct_byte(vy.0, 0x03))
        }
        Instruction::AddVx(vx, vy) => {
            AssembledInstruction(construct_byte(0x08, vx.0), construct_byte(vy.0, 0x04))
        }
        Instruction::SubVx(vx, vy) => {
            AssembledInstruction(construct_byte(0x08, vx.0), construct_byte(vy.0, 0x05))
        }
        Instruction::ShiftRight(vx) => {
            AssembledInstruction(construct_byte(0x08, vx.0), construct_byte(0x00, 0x06))
        }
        Instruction::SubN(vx, vy) => {
            AssembledInstruction(construct_byte(0x08, vx.0), construct_byte(vy.0, 0x07))
        }
        Instruction::ShiftLeft(vx) => {
            AssembledInstruction(construct_byte(0x08, vx.0), construct_byte(0x00, 0x0E))
        }
        Instruction::SkipNotEqVx(vx, vy) => {
            AssembledInstruction(construct_byte(0x09, vx.0), construct_byte(vy.0, 0x00))
        }
        Instruction::LoadI(addr) => {
            AssembledInstruction(construct_byte(0x0A, (addr.0 >> 8) as u8), addr.0 as u8)
        }
        Instruction::JmpV0(addr) => {
            AssembledInstruction(construct_byte(0x0B, (addr.0 >> 8) as u8), addr.0 as u8)
        }
        Instruction::Rand(vx, constant) => {
            AssembledInstruction(construct_byte(0x0C, vx.0), constant)
        }
        Instruction::Draw(vx, vy, constant) => {
            AssembledInstruction(construct_byte(0x0D, vx.0), construct_byte(vy.0, constant))
        }
        Instruction::SkipKeyPressed(vx) => AssembledInstruction(construct_byte(0x0E, vx.0), 0x9E),
        Instruction::SkipKeyNotPressed(vx) => {
            AssembledInstruction(construct_byte(0x0E, vx.0), 0xA1)
        }
        Instruction::LoadDelay(vx) => AssembledInstruction(construct_byte(0x0F, vx.0), 0x07),
        Instruction::LoadKey(vx) => AssembledInstruction(construct_byte(0x0F, vx.0), 0x0A),
        Instruction::SetDelay(vx) => AssembledInstruction(construct_byte(0x0F, vx.0), 0x15),
        Instruction::SetSound(vx) => AssembledInstruction(construct_byte(0x0F, vx.0), 0x18),
        Instruction::AddI(vx) => AssembledInstruction(construct_byte(0x0F, vx.0), 0x1E),
        Instruction::LoadFont(vx) => AssembledInstruction(construct_byte(0x0F, vx.0), 0x29),
        Instruction::LoadBcd(vx) => AssembledInstruction(construct_byte(0x0F, vx.0), 0x33),
        Instruction::StoreRegisters(vx) => AssembledInstruction(construct_byte(0x0F, vx.0), 0x55),
        Instruction::LoadRegisters(vx) => AssembledInstruction(construct_byte(0x0F, vx.0), 0x65),
    }
}
