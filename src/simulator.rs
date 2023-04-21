use std::collections::HashMap;

use crate::decoder::DecodedArgument;
use std::io::Result;

pub struct ExecutedOperation {
    pub dest_reg: String,
    pub dest_start: u16,
    pub dest_end: u16,
}

pub struct SimulationResult {
    pub steps: Vec<ExecutedOperation>,
    pub final_status: HashMap<String, Register>,
}

#[derive(Hash, Copy, Clone)]
pub struct Register {
    pub low: u8,
    pub high: u8,
}

enum Source {
    Register(Register),
    Immediate(u16),
}

enum RegisterMode {
    High,
    Low,
    Universal,
}
impl RegisterMode {
    fn parse(val: &str) -> Option<RegisterMode> {
        match val.to_lowercase().as_str() {
            "ah" | "bh" | "ch" | "dh" => Some(RegisterMode::High),
            "al" | "bl" | "cl" | "dl" => Some(RegisterMode::Low),
            "ax" | "bx" | "cx" | "dx" | "sp" | "bp" | "si" | "di" | "ss" | "cs" | "ds" | "es" => {
                Some(RegisterMode::Universal)
            }
            _ => None,
        }
    }
}

fn get_parent_register(reg: &str) -> String {
    match reg {
        "ah" | "al" => "ax".to_string(),
        "bh" | "bl" => "bx".to_string(),
        "ch" | "cl" => "cx".to_string(),
        "dh" | "dl" => "dx".to_string(),
        _ => reg.to_string(),
    }
}

fn get_source(registers: &HashMap<String, Register>, source: &str) -> Source {
    if source.starts_with("0x") {
        let without_prefix = &source[2..];
        return Source::Immediate(u16::from_str_radix(without_prefix, 16).unwrap());
    }
    match source.parse::<i16>() {
        Ok(immediate) => Source::Immediate(immediate as u16),
        Err(_) => {
            let source_parent = match RegisterMode::parse(source).expect("ERROR: Invalid register!")
            {
                RegisterMode::High | RegisterMode::Low => get_parent_register(source),
                RegisterMode::Universal => source.to_string(),
            };

            let source_register = registers
                .get(source_parent.as_str())
                .expect("ERROR: Register not found!");
            return Source::Register(Register {
                high: source_register.high,
                low: source_register.low,
            });
        }
    }
}
pub fn get_register_value(reg: &Register) -> u16 {
    return ((reg.high as u16) << 8) | (reg.low as u16);
}

pub fn get_source_part(reg: &str, value: &Register) -> u8 {
    let source_register_mode = RegisterMode::parse(reg).expect("ERROR: Source must be a register");
    match source_register_mode {
        RegisterMode::High => value.high,
        RegisterMode::Low | RegisterMode::Universal => value.low,
    }
}
pub fn execute_register_register_mov(
    instruction: &DecodedArgument,
    registers: &mut HashMap<String, Register>,
) -> ExecutedOperation {
    let source = get_source(&registers, &instruction.source);

    let destination_key = get_parent_register(instruction.destination.as_str());
    let destination = registers
        .get_mut(&destination_key)
        .expect("ERROR: Destination register not in Register set.");
    let destination_starting_value = get_register_value(&destination);

    let register_mode = RegisterMode::parse(instruction.destination.as_str())
        .expect("ERROR: Destination must be a register!");
    match register_mode {
        RegisterMode::High => {
            destination.high = match source {
                Source::Register(reg) => get_source_part(&instruction.source, &reg),
                Source::Immediate(immed) => immed as u8,
            };
        }

        RegisterMode::Low => {
            destination.low = match source {
                Source::Register(reg) => get_source_part(&instruction.source, &reg),
                Source::Immediate(immed) => immed as u8,
            };
        }

        RegisterMode::Universal => {
            destination.high = match source {
                Source::Register(reg) => reg.high,
                Source::Immediate(immed) => {
                    let high_mask = u16::MAX - 255;
                    let high_masked = immed & high_mask;
                    (high_masked >> 8) as u8
                }
            };

            destination.low = match source {
                Source::Register(reg) => reg.low,
                Source::Immediate(immed) => immed as u8,
            };
        }
    }
    return ExecutedOperation {
        dest_reg: destination_key,
        dest_start: destination_starting_value,
        dest_end: get_register_value(destination),
    };
}

pub fn execute_instructions(instructions: &Vec<DecodedArgument>) -> Result<SimulationResult> {
    let mut registers: HashMap<String, Register> = HashMap::from([
        (String::from("ax"), Register { low: 0, high: 0 }),
        (String::from("bx"), Register { low: 0, high: 0 }),
        (String::from("cx"), Register { low: 0, high: 0 }),
        (String::from("dx"), Register { low: 0, high: 0 }),
        (String::from("sp"), Register { low: 0, high: 0 }),
        (String::from("bp"), Register { low: 0, high: 0 }),
        (String::from("si"), Register { low: 0, high: 0 }),
        (String::from("di"), Register { low: 0, high: 0 }),
        (String::from("cs"), Register { low: 0, high: 0 }),
        (String::from("ss"), Register { low: 0, high: 0 }),
        (String::from("ds"), Register { low: 0, high: 0 }),
        (String::from("es"), Register { low: 0, high: 0 }),
    ]);

    let mut flags: [u8; 16] = [0; 16];
    let mut execution_history: Vec<ExecutedOperation> = Vec::new();

    for instruction in instructions {
        match instruction.operand.as_str() {
            "mov" => {
                execution_history.push(execute_register_register_mov(instruction, &mut registers));
            }
            "add" => {
                execution_history.push(execute_register_register_add(instruction, &mut registers, &mut flags));
            }
            "sub" => {
                execution_history.push(execute_register_register_sub(instruction, &mut registers, &mut flags));
            }
            "cmp" => {
                execution_history.push(execute_register_register_cmp(instruction, &mut registers, &mut flags));
            }
            _ => (),
        }
    }
    return Ok(SimulationResult {
        steps: execution_history,
        final_status: registers,
    });
}

fn execute_register_register_cmp(
    instruction: &DecodedArgument,
    registers: &HashMap<String, Register>,
    flags: &mut [u8;16]
) -> ExecutedOperation {
    let source = get_source(&registers, &instruction.source);

    let destination_key = get_parent_register(instruction.destination.as_str());
    // Clone should dereference the value pair :C
    let mut destination = registers
        .get(&destination_key)
        .expect("ERROR: Destination register not in Register set.")
        .clone();
    let destination_starting_value = get_register_value(&destination);

    let register_mode = RegisterMode::parse(instruction.destination.as_str())
        .expect("ERROR: Destination must be a register!");
    match register_mode {
        RegisterMode::High => {
            destination.high -= match source {
                Source::Register(reg) => get_source_part(&instruction.source, &reg),
                Source::Immediate(immed) => immed as u8,
            };
        }

        RegisterMode::Low => {
            destination.low -= match source {
                Source::Register(reg) => get_source_part(&instruction.source, &reg),
                Source::Immediate(immed) => immed as u8,
            };
        }

        RegisterMode::Universal => {
            destination.high -= match source {
                Source::Register(reg) => reg.high,
                Source::Immediate(immed) => {
                    let high_mask = u16::MAX - 255;
                    let high_masked = immed & high_mask;
                    (high_masked >> 8) as u8
                }
            };

            destination.low -= match source {
                Source::Register(reg) => reg.low,
                Source::Immediate(immed) => immed as u8,
            };
        }
    }

    set_flags(get_register_value(&destination), flags);

    return ExecutedOperation {
        dest_reg: destination_key,
        dest_start: destination_starting_value,
        dest_end: get_register_value(&destination),
    };
}

fn execute_register_register_add(
    instruction: &DecodedArgument,
    registers: &mut HashMap<String, Register>,
    flags: &mut [u8;16]
) -> ExecutedOperation {
    let source = get_source(&registers, &instruction.source);

    let destination_key = get_parent_register(instruction.destination.as_str());
    let destination = registers
        .get_mut(&destination_key)
        .expect("ERROR: Destination register not in Register set.");
    let destination_starting_value = get_register_value(&destination);

    let register_mode = RegisterMode::parse(instruction.destination.as_str())
        .expect("ERROR: Destination must be a register!");
    match register_mode {
        RegisterMode::High => {
            destination.high += match source {
                Source::Register(reg) => get_source_part(&instruction.source, &reg),
                Source::Immediate(immed) => immed as u8,
            };
        }

        RegisterMode::Low => {
            destination.low += match source {
                Source::Register(reg) => get_source_part(&instruction.source, &reg),
                Source::Immediate(immed) => immed as u8,
            };
        }

        RegisterMode::Universal => {
            destination.high += match source {
                Source::Register(reg) => reg.high,
                Source::Immediate(immed) => {
                    let high_mask = u16::MAX - 255;
                    let high_masked = immed & high_mask;
                    (high_masked >> 8) as u8
                }
            };

            destination.low += match source {
                Source::Register(reg) => reg.low,
                Source::Immediate(immed) => immed as u8,
            };
        }
    }

    set_flags(get_register_value(destination), flags);

    return ExecutedOperation {
        dest_reg: destination_key,
        dest_start: destination_starting_value,
        dest_end: get_register_value(destination),
    };
}

fn execute_register_register_sub(
    instruction: &DecodedArgument,
    registers: &mut HashMap<String, Register>,
    flags: &mut [u8;16]
) -> ExecutedOperation {
    let source = get_source(&registers, &instruction.source);

    let destination_key = get_parent_register(instruction.destination.as_str());
    let destination = registers
        .get_mut(&destination_key)
        .expect("ERROR: Destination register not in Register set.");
    let destination_starting_value = get_register_value(&destination);

    let register_mode = RegisterMode::parse(instruction.destination.as_str())
        .expect("ERROR: Destination must be a register!");
    match register_mode {
        RegisterMode::High => {
            destination.high -= match source {
                Source::Register(reg) => get_source_part(&instruction.source, &reg),
                Source::Immediate(immed) => immed as u8,
            };
        }

        RegisterMode::Low => {
            destination.low -= match source {
                Source::Register(reg) => get_source_part(&instruction.source, &reg),
                Source::Immediate(immed) => immed as u8,
            };
        }

        RegisterMode::Universal => {
            destination.high -= match source {
                Source::Register(reg) => reg.high,
                Source::Immediate(immed) => {
                    let high_mask = u16::MAX - 255;
                    let high_masked = immed & high_mask;
                    (high_masked >> 8) as u8
                }
            };

            destination.low -= match source {
                Source::Register(reg) => reg.low,
                Source::Immediate(immed) => immed as u8,
            };
        }
    }

    set_flags(get_register_value(destination), flags);

    return ExecutedOperation {
        dest_reg: destination_key,
        dest_start: destination_starting_value,
        dest_end: get_register_value(destination),
    };
}
fn set_flags(destination: u16, flags: &mut [u8]) {
    // For now ZF is at 0, and SF is at 1
    flags[0] = (destination == 0) as u8;
    flags[1] = (destination >= 32768) as u8; // Sign bit, we're watching if the highest bit is set to 1
}
