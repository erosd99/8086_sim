use std::collections::HashMap;

use crate::decoder::DecodedArgument;
use std::io::Result;

pub struct ExecutedOperation {
    pub input: String,
    pub dest_reg: String,
    pub dest_start: u16,
    pub dest_end: u16,
}

pub struct SimulationResult {
    pub steps: Vec<ExecutedOperation>,
    pub final_status: HashMap<String, Register>,
}

#[derive(Hash)]
pub struct Register {
    pub low: u8,
    pub high: u8,
}
pub fn execute_instructions(instructions: Vec<&DecodedArgument>) -> Result<SimulationResult> {
    let mut registers: HashMap<String, Register> = HashMap::from([
        (String::from("ax"), Register { low: 0, high: 0 }),
        (String::from("bx"), Register { low: 0, high: 0 }),
        (String::from("cx"), Register { low: 0, high: 0 }),
        (String::from("dx"), Register { low: 0, high: 0 }),
        (String::from("sp"), Register { low: 0, high: 0 }),
        (String::from("dp"), Register { low: 0, high: 0 }),
        (String::from("si"), Register { low: 0, high: 0 }),
        (String::from("di"), Register { low: 0, high: 0 }),
    ]);

    let mut execution_history: Vec<ExecutedOperation> = Vec::new();

    for instruction in instructions {
        match instruction.operand.as_str() {
            "mov" => {
                // currently only working for register to register / immediate to register
                match instruction.destination.chars().last().unwrap(){
                    'h' => {
                        let parent_register = instruction.destination.chars()[0];
                    }
                    'l' => (),
                    _ => (),
                }
            }
            _ => (),
        }
    }
    return Ok(SimulationResult {
        steps: execution_history,
        final_status: registers,
    });
}
