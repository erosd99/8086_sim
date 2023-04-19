#[derive(Eq, PartialEq, Hash, Debug)]
pub enum Flag {
    S,
    W,
    D,
}

#[derive(Copy, Clone, Debug)]
pub enum WordField {
    Addr,
    Disp,
    Data,
}


#[derive(Copy, Clone, Debug)]
pub enum Reg {
    Implicit,
    Explicit(u8),
}


#[derive(Debug)]
pub enum Argument {
    Mode,
    Rm,
    Reg(Reg),
    Word(WordField),
    Byte,
    SegReg,
    FixedBit(u8),
}

pub struct Instruction<'a> {
    pub operand: &'a str,
    pub offset: usize,
    pub max_byte_count: usize,
    pub flags: Vec<Flag>,
    pub extra_args: Vec<Argument>,
}

pub struct SecondaryOperand {
    pub offset: usize, // The one-indexed starting bit of the operand. E.g. if the operand starts at the second bit of
    // the second byte, the offset is 9
    pub value: u8,
}

pub enum InstructionLookup<'a> {
    Instr(Instruction<'a>),
    MultiInstr(Vec<(SecondaryOperand, Instruction<'a>)>),
}
pub fn generate_instruction_table<'a>() -> Vec<(u8, InstructionLookup<'a>)> {
    return vec![
        (
            0b10001000,
            InstructionLookup::Instr(Instruction {
                operand: "mov",
                offset: 6,
                max_byte_count: 4,
                flags: vec![Flag::D, Flag::W],
                extra_args: vec![
                    Argument::Mode,
                    Argument::Reg(Reg::Implicit),
                    Argument::Rm,
                    Argument::Word(WordField::Disp),
                ],
            }),
        ),
        (
            0b11000110,
            InstructionLookup::Instr(Instruction {
                operand: "mov",
                offset: 7,
                max_byte_count: 6,
                flags: vec![Flag::W],
                extra_args: vec![
                    Argument::Mode,
                    Argument::Reg(Reg::Explicit(0)),
                    Argument::Rm,
                    Argument::Word(WordField::Disp),
                    Argument::Word(WordField::Data),
                ],
            }),
        ),
        (
            0b10110000,
            InstructionLookup::Instr(Instruction {
                operand: "mov",
                offset: 4,
                max_byte_count: 3,
                flags: vec![Flag::W],
                extra_args: vec![
                    Argument::Reg(Reg::Implicit),
                    Argument::Word(WordField::Data),
                ],
            }),
        ),
        (
            0b10100000,
            InstructionLookup::Instr(Instruction {
                operand: "mov",
                offset: 7,
                max_byte_count: 3,
                flags: vec![Flag::W],
                extra_args: vec![Argument::Word(WordField::Addr)],
            }),
        ),
        (
            0b10100010,
            InstructionLookup::Instr(Instruction {
                operand: "mov",
                offset: 7,
                max_byte_count: 3,
                flags: vec![Flag::W],
                extra_args: vec![Argument::Word(WordField::Addr)],
            }),
        ),
        (
            0b10001110,
            InstructionLookup::Instr(Instruction {
                operand: "mov",
                offset: 8,
                max_byte_count: 4,
                flags: vec![],
                extra_args: vec![
                    Argument::Mode,
                    Argument::FixedBit(0),
                    Argument::SegReg,
                    Argument::Rm,
                    Argument::Word(WordField::Disp),
                ],
            }),
        ),
        (
            0b10001100,
            InstructionLookup::Instr(Instruction {
                operand: "mov",
                offset: 8,
                max_byte_count: 4,
                flags: vec![],
                extra_args: vec![
                    Argument::Mode,
                    Argument::FixedBit(0),
                    Argument::SegReg,
                    Argument::Rm,
                    Argument::Word(WordField::Disp),
                ],
            }),
        ),
        (
            0b00000000,
            InstructionLookup::Instr(Instruction {
                operand: "add",
                offset: 6,
                max_byte_count: 4,
                flags: vec![Flag::D, Flag::W],
                extra_args: vec![
                    Argument::Mode,
                    Argument::Reg(Reg::Implicit),
                    Argument::Rm,
                    Argument::Word(WordField::Disp),
                ],
            }),
        ),
        (
            0b10000000,
            InstructionLookup::MultiInstr(vec![
                (
                    SecondaryOperand {
                        offset: 11,
                        value: 0b000,
                    },
                    Instruction {
                        operand: "add",
                        offset: 6,
                        max_byte_count: 6,
                        flags: vec![Flag::S, Flag::W],
                        extra_args: vec![
                            Argument::Mode,
                            Argument::Reg(Reg::Explicit(0b000)),
                            Argument::Rm,
                            Argument::Word(WordField::Disp),
                            Argument::Word(WordField::Data),
                        ],
                    },
                ),
                (
                    SecondaryOperand {
                        offset: 11,
                        value: 0b101,
                    },
                    Instruction {
                        operand: "sub",
                        offset: 6,
                        max_byte_count: 6,
                        flags: vec![Flag::S, Flag::W],
                        extra_args: vec![
                            Argument::Mode,
                            Argument::Reg(Reg::Explicit(0b101)),
                            Argument::Rm,
                            Argument::Word(WordField::Disp),
                            Argument::Word(WordField::Data),
                        ],
                    },
                ),
                (
                    SecondaryOperand {
                        offset: 11,
                        value: 0b111,
                    },
                    Instruction {
                        operand: "cmp",
                        offset: 6,
                        max_byte_count: 6,
                        flags: vec![Flag::S, Flag::W],
                        extra_args: vec![
                            Argument::Mode,
                            Argument::Reg(Reg::Explicit(0b111)),
                            Argument::Rm,
                            Argument::Word(WordField::Disp),
                            Argument::Word(WordField::Data),
                        ],
                    },
                ),
            ]),
        ),
        (
            0b00000100,
            InstructionLookup::Instr(Instruction {
                operand: "add",
                offset: 7,
                max_byte_count: 3,
                flags: vec![Flag::W],
                extra_args: vec![Argument::Word(WordField::Data)],
            }),
        ),
        (
            0b00101000,
            InstructionLookup::Instr(Instruction {
                operand: "sub",
                offset: 6,
                max_byte_count: 4,
                flags: vec![Flag::D, Flag::W],
                extra_args: vec![
                    Argument::Mode,
                    Argument::Reg(Reg::Implicit),
                    Argument::Rm,
                    Argument::Word(WordField::Disp),
                ],
            }),
        ),
        (
            0b00101100,
            InstructionLookup::Instr(Instruction {
                operand: "sub",
                offset: 7,
                max_byte_count: 3,
                flags: vec![Flag::W],
                extra_args: vec![Argument::Word(WordField::Data)],
            }),
        ),
        (
            0b00111000,
            InstructionLookup::Instr(Instruction {
                operand: "cmp",
                offset: 6,
                max_byte_count: 4,
                flags: vec![Flag::D, Flag::W],
                extra_args: vec![
                    Argument::Mode,
                    Argument::Reg(Reg::Implicit),
                    Argument::Rm,
                    Argument::Word(WordField::Disp),
                ],
            }),
        ),
        (
            0b00111100,
            InstructionLookup::Instr(Instruction {
                operand: "cmp",
                offset: 7,
                max_byte_count: 3,
                flags: vec![Flag::W],
                extra_args: vec![Argument::Word(WordField::Data)],
            }),
        ),
        (
            0b01110100,
            InstructionLookup::Instr(Instruction {
                operand: "je",
                offset: 8,
                max_byte_count: 2,
                flags: vec![],
                extra_args: vec![Argument::Byte],
            }),
        ),
        (
            0b01111100,
            InstructionLookup::Instr(Instruction {
                operand: "jl",
                offset: 8,
                max_byte_count: 2,
                flags: vec![],
                extra_args: vec![Argument::Byte],
            }),
        ),
        (
            0b01111110,
            InstructionLookup::Instr(Instruction {
                operand: "jle",
                offset: 8,
                max_byte_count: 2,
                flags: vec![],
                extra_args: vec![Argument::Byte],
            }),
        ),
        (
            0b01110010,
            InstructionLookup::Instr(Instruction {
                operand: "jb",
                offset: 8,
                max_byte_count: 2,
                flags: vec![],
                extra_args: vec![Argument::Byte],
            }),
        ),
        (
            0b01110110,
            InstructionLookup::Instr(Instruction {
                operand: "jbe",
                offset: 8,
                max_byte_count: 2,
                flags: vec![],
                extra_args: vec![Argument::Byte],
            }),
        ),
        (
            0b01111010,
            InstructionLookup::Instr(Instruction {
                operand: "jp",
                offset: 8,
                max_byte_count: 2,
                flags: vec![],
                extra_args: vec![Argument::Byte],
            }),
        ),
        (
            0b01110000,
            InstructionLookup::Instr(Instruction {
                operand: "jo",
                offset: 8,
                max_byte_count: 2,
                flags: vec![],
                extra_args: vec![Argument::Byte],
            }),
        ),
        (
            0b01111000,
            InstructionLookup::Instr(Instruction {
                operand: "js",
                offset: 8,
                max_byte_count: 2,
                flags: vec![],
                extra_args: vec![Argument::Byte],
            }),
        ),
        (
            0b01110101,
            InstructionLookup::Instr(Instruction {
                operand: "jne",
                offset: 8,
                max_byte_count: 2,
                flags: vec![],
                extra_args: vec![Argument::Byte],
            }),
        ),
        (
            0b01111101,
            InstructionLookup::Instr(Instruction {
                operand: "jnl",
                offset: 8,
                max_byte_count: 2,
                flags: vec![],
                extra_args: vec![Argument::Byte],
            }),
        ),
        (
            0b01111111,
            InstructionLookup::Instr(Instruction {
                operand: "jnle",
                offset: 8,
                max_byte_count: 2,
                flags: vec![],
                extra_args: vec![Argument::Byte],
            }),
        ),
        (
            0b01110011,
            InstructionLookup::Instr(Instruction {
                operand: "jnb",
                offset: 8,
                max_byte_count: 2,
                flags: vec![],
                extra_args: vec![Argument::Byte],
            }),
        ),
        (
            0b01110111,
            InstructionLookup::Instr(Instruction {
                operand: "jnbe",
                offset: 8,
                max_byte_count: 2,
                flags: vec![],
                extra_args: vec![Argument::Byte],
            }),
        ),
        (
            0b01111011,
            InstructionLookup::Instr(Instruction {
                operand: "jnp",
                offset: 8,
                max_byte_count: 2,
                flags: vec![],
                extra_args: vec![Argument::Byte],
            }),
        ),
        (
            0b01110001,
            InstructionLookup::Instr(Instruction {
                operand: "jno",
                offset: 8,
                max_byte_count: 2,
                flags: vec![],
                extra_args: vec![Argument::Byte],
            }),
        ),
        (
            0b01111001,
            InstructionLookup::Instr(Instruction {
                operand: "jns",
                offset: 8,
                max_byte_count: 2,
                flags: vec![],
                extra_args: vec![Argument::Byte],
            }),
        ),
        (
            0b11100010,
            InstructionLookup::Instr(Instruction {
                operand: "loop",
                offset: 8,
                max_byte_count: 2,
                flags: vec![],
                extra_args: vec![Argument::Byte],
            }),
        ),
        (
            0b11100001,
            InstructionLookup::Instr(Instruction {
                operand: "loopz",
                offset: 8,
                max_byte_count: 2,
                flags: vec![],
                extra_args: vec![Argument::Byte],
            }),
        ),
        (
            0b11100000,
            InstructionLookup::Instr(Instruction {
                operand: "loopnz",
                offset: 8,
                max_byte_count: 2,
                flags: vec![],
                extra_args: vec![Argument::Byte],
            }),
        ),
        (
            0b11100011,
            InstructionLookup::Instr(Instruction {
                operand: "jcxz",
                offset: 8,
                max_byte_count: 2,
                flags: vec![],
                extra_args: vec![Argument::Byte],
            }),
        ),
    ];
}
