use std::io;
use std::io::prelude::*;
use std::fs::File;

fn main() -> io::Result<()> {
    let mut out = std::io::stdout();

    let mut main: [u8; 0xFFFF] = [0; 0xFFFF];
    let mut _io: [u8; 0xFF] = [0; 0xFF];
    let mut work = Vec::<u8>::with_capacity(0xFF);
    let mut _ret = Vec::<u8>::with_capacity(0xFF);
    let mut counter: u16 = 0x0100;

    let rom_load_area = &mut main[0x0100..];

    let mut f = File::open("roms/test/add_sub.rom")?;
    let n = f.read(rom_load_area)?;
    print_bytes(&rom_load_area[..n]);

    loop {
        let raw_code = main[counter as usize];
        let code = parse_code(raw_code);
        match code {
            Code::ADD(_f) => {
                let b = work.pop().unwrap();
                let a = work.pop().unwrap();
                work.push(a + b);
            },
            Code::SUB(_f) => {
                let b = work.pop().unwrap();
                let a = work.pop().unwrap();
                work.push(a - b);
            },
            Code::LIT(_f) => {
                counter += 1;
                work.push(main[counter as usize]);
            },
            Code::DEO(_f) => {
                let _device = work.pop().unwrap();
                let value = work.pop().unwrap();
                out.write(&[value])?;
                out.flush()?;
            },
            Code::BRK => {
                break;
            }
        };
        counter += 1;
    }

    Ok(())
}

fn print_bytes(data: &[u8]) {
    println!("{:0>2x?}", &data);
}

struct CodeFlags {
    keep: bool,
    ret: bool,
    short: bool,
}

struct LitFlags {
    ret: bool,
    short: bool,
}

enum Code {
    BRK,
    DEO(CodeFlags),
    ADD(CodeFlags),
    SUB(CodeFlags),
    LIT(LitFlags),
}

fn parse_code(byte: u8) -> Code {
    let code = 0b000_11111 & byte;
    let short = 0b001_00000 & byte != 0;
    let ret = 0b010_00000 & byte != 0;
    if ret { todo!("Return flag not yet implemented! Code: {byte:0>2x?}"); }
    let keep = 0b100_00000 & byte != 0;
    if keep && code != 0x00 { todo!("Keep flag not yet implemented! Code: {byte:0>2x?}"); }

    let flags = CodeFlags {keep, ret, short};
    match code {
        0x00 => if keep {
                Code::LIT(LitFlags {ret, short})
            } else {
                Code::BRK
            },
        0x17 => Code::DEO(flags),
        0x18 => Code::ADD(flags),
        0x19 => Code::SUB(flags),
        _ => todo!("Missing opcode! Code: {byte:0>2x?}"),
    }
}
