use std::io;
use std::io::prelude::*;
use std::fs::File;

use uxn::cpu::Cpu;
use uxn::varvara::Varvara;

fn main() -> io::Result<()> {
    let mut out = std::io::stdout();

    let mut varvara = Varvara::new();
    let mut uxn = Cpu::new();

    let rom_load_area = &mut varvara.main[0x0100..];
    let mut file = File::open("roms/test/SUB2_wrap.rom")?;
    let n = file.read(rom_load_area)?;
    print_bytes(&rom_load_area[..n]);

    loop {
        let raw_code = uxn.next_byte(&varvara);
        let code = parse_code(raw_code);
        match code {
            Code::ADD(f) => {
                if f.short {
                    let b = uxn.work.pop2();
                    let a = uxn.work.pop2();
                    uxn.work.push2(a.wrapping_add(b));
                } else {
                    let b = uxn.work.pop();
                    let a = uxn.work.pop();
                    uxn.work.push(a.wrapping_add(b));
                }
            },
            Code::SUB(f) => {
                if f.short {
                    let b = uxn.work.pop2();
                    let a = uxn.work.pop2();
                    uxn.work.push2(a.wrapping_sub(b));
                } else {
                    let b = uxn.work.pop();
                    let a = uxn.work.pop();
                    uxn.work.push(a.wrapping_sub(b));
                }
            },
            Code::LIT(f) => {
                if f.short {
                    let short = uxn.next_short(&varvara);
                    uxn.work.push2(short);
                } else {
                    let byte = uxn.next_byte(&varvara);
                    uxn.work.push(byte);
                }
            },
            Code::DEO(_f) => {
                let _device = uxn.work.pop();
                let value = uxn.work.pop();
                out.write(&[value])?;
                out.flush()?;
            },
            Code::BRK => {
                break;
            }
        };
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

// They'll all implement the Varvara::Device trait
struct System {}
struct Console {}
struct Screen {}

fn parse_port(byte: u8) -> () {
    let device_nibble = 0xF0 & byte;
    let port_nibble = 0x0F & byte;
    match device_nibble {
        0x00 => todo!("System device not implemented"), // System
        0x10 => {
        },
        _ => todo!()
    }
}
