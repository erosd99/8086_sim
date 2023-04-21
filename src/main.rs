mod decoder;
mod instruction_table;
mod simulator;

use std::env;
use std::fs;
use std::io::Result;

use self::decoder::DecodedArgument;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    let source_file = &args[1];
    let mode = &args[2];

    let input = fs::read(source_file)?;

    if mode == "decode" {
        let decoded = decoder::decode_bitstream(input)?;

        for line in &decoded {
            if line.source == "" {
                println!("{} {}", line.operand, line.destination);
            } else {
                println!("{} {}, {}", line.operand, line.destination, line.source);
            }
        }
    } else if mode == "execute" {
        let input: Vec<DecodedArgument> = fs::read_to_string(source_file)?
            .split("\n")
            .filter(|s| !s.is_empty())
            .map(|s| DecodedArgument::parse(s).expect("The argument couldn't be decoded. Please verify that your formatting is correct."))
            .collect();

        for line in &input {
                println!("{} {}, {}", line.operand, line.destination, line.source);
        }
        let result = simulator::execute_instructions(&input)?;

        assert!(&input.len() == &result.steps.len(), "ERROR: Not all instructions could be decoded.");

        for idx in 0..input.len() {
            println!("{} {}, {}; {}: {:x} --> {:x}", input[idx].operand, input[idx].destination, input[idx].source, 
                     result.steps[idx].dest_reg, result.steps[idx].dest_start, result.steps[idx].dest_end);
        }
        println!("\n Final registers:");
        for (key, value) in result.final_status {
            println!("{}: {:x}", key, simulator::get_register_value(&value));
        }

    }
    Ok(())
}
