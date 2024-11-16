pub const BASE_OPCODES: [&str; 7] = ["BRK", "INC", "DEO", "DEI", "ADD", "SUB", "LIT"];

pub struct CodeFlags {
    pub keep: bool,
    pub ret: bool,
    pub short: bool,
}

pub struct LitFlags {
    pub ret: bool,
    pub short: bool,
}

pub enum Code {
    BRK,
    INC(CodeFlags),
    DEO(CodeFlags),
    DEI(CodeFlags),
    ADD(CodeFlags),
    SUB(CodeFlags),
    LIT(LitFlags),
}

pub fn encode_base_code(code: &str) -> u8 {
    match code {
        "BRK" => 0x00,
        "INC" => 0x01,
        "DEI" => 0x16,
        "DEO" => 0x17,
        "ADD" => 0x18,
        "SUB" => 0x19,
        "LIT" => 0x80,
        _ => panic!("Unrecognized opcode: {code}"),
    }
}

pub fn parse_code(byte: u8) -> Code {
    let code = 0b000_11111 & byte;
    let short = 0b001_00000 & byte != 0;
    let ret = 0b010_00000 & byte != 0;
    if ret {
        todo!("Return flag not yet implemented! Code: {byte:0>2x?}");
    }
    let keep = 0b100_00000 & byte != 0;
    if keep && code != 0x00 {
        todo!("Keep flag not yet implemented! Code: {byte:0>2x?}");
    }

    let flags = CodeFlags { keep, ret, short };
    match code {
        0x00 => {
            if keep {
                Code::LIT(LitFlags { ret, short })
            } else {
                Code::BRK
            }
        }
        0x01 => Code::INC(flags),
        0x16 => Code::DEI(flags),
        0x17 => Code::DEO(flags),
        0x18 => Code::ADD(flags),
        0x19 => Code::SUB(flags),
        _ => todo!("Missing opcode! Code: {byte:0>2x?}"),
    }
}
