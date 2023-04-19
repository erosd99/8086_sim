use std::collections::HashMap;
use std::io::Result;

use crate::instruction_table::{
    generate_instruction_table, Argument, Flag, Instruction, InstructionLookup, Reg, WordField,
};

const REGISTER_ENCODING: [[&'static str; 8]; 2] = [
    ["al", "cl", "dl", "bl", "ah", "ch", "dh", "bh"],
    ["ax", "cx", "dx", "bx", "sp", "bp", "si", "di"],
];

const MEMORY_ENCODING_BASE: [&'static str; 8] = [
    "bx + si", "bx + di", "bp + si", "bp + di", "si", "di", "bp", "bx",
];
#[repr(u8)]
#[derive(Debug)]
enum Mode {
    MemoryModeNoDisplacement,
    MemoryModeByteDisplacement,
    MemoryModeWordDisplacement,
    RegisterMode,
    NoMode,
}
impl Mode {
    fn parse(val: u8) -> Mode {
        match val {
            0 => Mode::MemoryModeNoDisplacement,
            1 => Mode::MemoryModeByteDisplacement,
            2 => Mode::MemoryModeWordDisplacement,
            3 => Mode::RegisterMode,
            _ => Mode::NoMode,
        }
    }
}
fn decode_register(w: usize, reg: usize) -> String {
    return REGISTER_ENCODING[w][reg].to_string();
}

fn decode_memory_address(idx: usize, offset: u16) -> String {
    if offset > 0 {
        return format!("[{} + {}]", MEMORY_ENCODING_BASE[idx], offset);
    }

    return format!("[{}]", MEMORY_ENCODING_BASE[idx]);
}

const MSB_BITMASKS: [u8; 8] = [
    0b10000000, 0b11000000, 0b11100000, 0b11110000, 0b11111000, 0b11111100, 0b11111110, 0b11111111,
];
fn decode_instruction<'a>(
    instruction_table: &'a [(u8, InstructionLookup<'a>)],
    from: &[u8; 2],
) -> Option<&'a Instruction<'a>> {
    for instruction in instruction_table {
        match &instruction.1 {
            InstructionLookup::Instr(ins) => {
                if (from[0] & MSB_BITMASKS[ins.offset - 1]) == instruction.0 {
                    return Some(&ins);
                }
            }
            InstructionLookup::MultiInstr(possible_instructions) => {
                for ins in possible_instructions {
                    let lookup_offset = ins.0.offset;

                    if (from[0] & MSB_BITMASKS[ins.1.offset - 1]) != instruction.0 {
                        continue;
                    }
                    // Currently either the last 3 bits in the first byte
                    // or the 3-5th bits on the second byte can be lookup values.
                    if (lookup_offset <= 6 && (from[0] & 0b111) == ins.0.value)
                        || (((from[1] >> 3) & 0b111) == ins.0.value)
                    {
                        return Some(&ins.1);
                    }
                }
            }
        };
    }
    return None;
}
fn get_argument_value(from: u8, argument_size: u8, offset: u8) -> u8 {
    let offset_byte = from >> (8 - (offset + argument_size));
    match argument_size {
        1 => offset_byte & 1,
        2 => offset_byte & 0b11,
        3 => offset_byte & 0b111,
        _ => 0,
    }
}

fn decode_flags<'a>(
    byte: &'a u8,
    offset: &'a usize,
    flags: &'a [Flag],
) -> HashMap<&'a Flag, usize> {
    // Due to ISA limitations, we can assume that all flags lie on the first bit
    let mut output = HashMap::new();
    for idx in 0..flags.len() {
        let offset_byte = byte >> (8 - (offset + idx + 1));
        let masked_byte = offset_byte & 1;
        output.insert(&flags[idx], masked_byte as usize);
    }

    return output;
}
pub struct DecodedArgument {
    pub operand: String,
    pub source: String,
    pub destination: String,
    byte_count: usize,
}

fn decode_arguments<'a>(
    from: &'a [u8],
    flags: &'a HashMap<&'a Flag, usize>,
    starting_offset: &usize,
    extra_args: &'a [Argument],
) -> Result<DecodedArgument> {
    let mut output = DecodedArgument {
        operand: String::from(""),
        source: String::from(""),
        destination: String::from(""),
        byte_count: starting_offset / 8,
    };

    let mut offset = *starting_offset;
    let mut mode = Mode::NoMode;
    let mut has_querky_displacement = false;
    let mut is_reg_explicit = false;

    for argument in extra_args {
        let source_byte = offset / 8;
        let byte_offset = offset % 8;
        match argument {
            Argument::Mode => {
                mode = Mode::parse(get_argument_value(from[source_byte], 2, byte_offset as u8));
                offset += 2;
            }
            Argument::Rm => {
                let rm_idx = get_argument_value(from[source_byte], 3, byte_offset as u8) as usize;
                match mode {
                    Mode::MemoryModeNoDisplacement => {
                        if rm_idx == 0b110 {
                            assert!(source_byte + 2 < from.len(), "ERROR: Trying to read immediate field, which is not in instruction stream");
                            output.source = (((from[source_byte + 2] as u16) << 8)
                                | (from[source_byte + 1] as u16))
                                .to_string();
                            has_querky_displacement = true;
                        } else if is_reg_explicit {
                            output.destination = decode_memory_address(rm_idx, 0);
                        } else {
                            output.source = decode_memory_address(rm_idx, 0);
                        }
                    }
                    Mode::MemoryModeByteDisplacement => {
                        assert!(
                            source_byte + 1 < from.len(),
                            "ERROR: Trying to read unset displacement byte!"
                        );
                        let displacement = (255 as u16) & from[source_byte + 1] as u16;
                        if is_reg_explicit {
                            output.destination = decode_memory_address(rm_idx, displacement);
                        } else {
                            output.source = decode_memory_address(rm_idx, displacement);
                        }
                    }
                    Mode::MemoryModeWordDisplacement => {
                        assert!(source_byte + 2 < from.len(), "ERROR: Trying to read immediate field, which is not in instruction stream");
                        let disp_lo = from[source_byte + 1];
                        let disp_hi = from[source_byte + 2];
                        let displacement = ((disp_hi as u16) << 8) | (disp_lo as u16);
                        if is_reg_explicit {
                            output.destination = decode_memory_address(rm_idx, displacement);
                        } else {
                            output.source = decode_memory_address(rm_idx, displacement);
                        }
                    }
                    Mode::RegisterMode => {
                        let w = flags.get(&Flag::W);
                        match w {
                            Some(w) => {
                                if is_reg_explicit {
                                    output.destination = decode_register(*w, rm_idx);
                                } else {
                                    output.source = decode_register(*w, rm_idx);
                                }
                            }
                            None => {
                                panic!("ERROR: Trying to decode reg field without w flag being set")
                            }
                        }
                    }
                    Mode::NoMode => {
                        panic!("ERROR: Trying to decode RM field without Mode being set.")
                    }
                }
                offset += 3;
            }
            Argument::Reg(reg) => match reg {
                Reg::Implicit => {
                    let w = flags.get(&Flag::W);
                    match w {
                        Some(w) => {
                            let reg = (get_argument_value(from[source_byte], 3, byte_offset as u8))
                                as usize;
                            output.destination = decode_register(*w, reg);
                        }
                        None => {
                            panic!("ERROR: Trying to decode reg field without w flag being set")
                        }
                    }
                    offset += 3;
                }
                Reg::Explicit(_) => {
                    is_reg_explicit = true;
                    offset += 3;
                }
            },
            Argument::Word(data) => match data {
                WordField::Data => {
                    let w = flags.get(&Flag::W);
                    match w {
                        Some(w) => {
                            let s = flags.get(&Flag::S);
                            match s {
                                Some(s) => {
                                    if *w == 1 && *s == 0 {
                                        assert!(source_byte + 1 < from.len(), "ERROR: Trying to read an immediate field which is not in instruction stream");

                                        let data_lo = from[source_byte];
                                        let data_hi = from[source_byte + 1];
                                        let displacement =
                                            ((data_hi as u16) << 8) | (data_lo as u16);

                                        output.source = displacement.to_string();
                                        offset += 16;
                                    } else if *s == 1 {
                                        assert!(source_byte < from.len(), "ERROR: Trying to read an immediate field which is not in instruction stream");

                                        if from[source_byte] >= 128 {
                                            output.source = (0b1111111100000000
                                                | from[source_byte] as u16)
                                                .to_string();
                                        } else {
                                            //
                                            output.source = (from[source_byte] as u16).to_string();
                                        }
                                        offset += 8;
                                    } else {
                                        assert!(source_byte < from.len(), "ERROR: Trying to read an immediate field which is not in instruction stream");

                                        output.source = from[source_byte].to_string();
                                        offset += 8;
                                    }
                                }
                                None => {
                                    if *w == 1 {
                                        assert!(source_byte + 2 < from.len(), "ERROR: Trying to read an immediate field which is not in instruction stream");
                                        let data_lo = from[source_byte];
                                        let data_hi = from[source_byte + 1];
                                        let displacement =
                                            ((data_hi as u16) << 8) | (data_lo as u16);
                                        output.source = displacement.to_string();
                                        offset += 16;
                                    } else {
                                        output.source = from[source_byte].to_string();
                                        offset += 8;
                                    }
                                }
                            }
                        }
                        None => {
                            output.source = from[source_byte].to_string();
                            offset += 8;
                        }
                    }
                }
                WordField::Disp => match mode {
                    Mode::MemoryModeByteDisplacement => {
                        offset += 8;
                    }
                    Mode::MemoryModeWordDisplacement => {
                        offset += 16;
                    }
                    Mode::MemoryModeNoDisplacement => {
                        if has_querky_displacement {
                            offset += 16;
                        }
                    }
                    _ => {}
                },
                _ => {
                    offset += 16;
                }
            },
            Argument::SegReg => offset += 2,
            Argument::FixedBit(_) => offset += 1,
            Argument::Byte => {
                output.destination = from[source_byte].to_string();
                offset += 8;
            }
        }
    }

    if output.destination == "" {
        let w = flags.get(&Flag::W);
        match w {
            Some(w) => {
                if *w == 1 {
                    output.destination = "ax".to_string();
                } else {
                    output.destination = "al".to_string();
                }
            }
            None => (),
        }
    }
    output.byte_count = offset / 8;
    return Ok(output);
}
pub fn decode_bitstream(input: Vec<u8>) -> Result<Vec<DecodedArgument>> {
    let instruction_table = generate_instruction_table();
    let mut output: Vec<DecodedArgument> = Vec::from([]);
    let mut idx = 0;
    while idx < input.len() - 1 {
        let lookup_bytes = [input[idx], input[idx + 1]];
        match decode_instruction(&instruction_table, &lookup_bytes) {
            None => {
                idx += 1;
            }
            Some(instruction) => {
                assert!(
                    instruction.offset + instruction.flags.len() <= 8,
                    "Flags can't bleed into the second byte!"
                );

                let flags = decode_flags(&lookup_bytes[0], &instruction.offset, &instruction.flags);
                let last_byte = std::cmp::min(idx + instruction.max_byte_count + 1, input.len());
                let mut arguments = decode_arguments(
                    &input[idx..last_byte],
                    &flags,
                    &(instruction.offset + flags.len()),
                    &instruction.extra_args,
                )?;
                if let Some(d) = flags.get(&Flag::D) {
                    if *d == 0 {
                        let temp = arguments.source;
                        arguments.source = arguments.destination;
                        arguments.destination = temp;
                    }
                }

                arguments.operand = instruction.operand.to_string();
                idx += &arguments.byte_count;
                output.push(arguments);
            }
        }
    }
    return Ok(output);
}
