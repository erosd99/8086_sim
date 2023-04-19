mod decoder;
mod instruction_table;
mod simulator;

use std::env;
use std::fs;
use std::io::Result;

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
        let input: Vec<&str> = fs::read_to_string(source_file)?.split("\n").collect();
    }
    Ok(())
}
