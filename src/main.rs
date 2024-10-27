use std::io;
use std::io::prelude::*;
use std::fs::File;

use uxn::cpu::Cpu;

fn main() -> io::Result<()> {
    let mut out = std::io::stdout();

    let mut main: [u8; 0xFFFF] = [0; 0xFFFF];
    let mut _io: [u8; 0xFF] = [0; 0xFF];

    let mut uxn = Cpu::new();

    let rom_load_area = &mut main[0x0100..];
    let mut file = File::open("roms/test/SUB2_wrap.rom")?;
    let n = file.read(rom_load_area)?;
    print_bytes(&rom_load_area[..n]);

    loop {
        let raw_code = main[uxn.counter as usize];
        let code = parse_code(raw_code);
        match code {
            Code::ADD(f) => {
                if f.short {
                    let low_b = uxn.work.pop().unwrap();
                    let high_b = uxn.work.pop().unwrap();
                    let b = u16::from_be_bytes([high_b, low_b]);

                    let low_a = uxn.work.pop().unwrap();
                    let high_a = uxn.work.pop().unwrap();
                    let a = u16::from_be_bytes([high_a, low_a]);

                    let [high, low] = a.wrapping_add(b).to_be_bytes();

                    uxn.work.push(high);
                    uxn.work.push(low);
                } else {
                    let b = uxn.work.pop().unwrap();
                    let a = uxn.work.pop().unwrap();
                    uxn.work.push(a.wrapping_add(b));
                }
            },
            Code::SUB(f) => {
                if f.short {
                    let low_b = uxn.work.pop().unwrap();
                    let high_b = uxn.work.pop().unwrap();
                    let b = u16::from_be_bytes([high_b, low_b]);

                    let low_a = uxn.work.pop().unwrap();
                    let high_a = uxn.work.pop().unwrap();
                    let a = u16::from_be_bytes([high_a, low_a]);

                    let [high, low] = a.wrapping_sub(b).to_be_bytes();

                    uxn.work.push(high);
                    uxn.work.push(low);
                } else {
                    let b = uxn.work.pop().unwrap();
                    let a = uxn.work.pop().unwrap();
                    uxn.work.push(a.wrapping_sub(b));
                }
            },
            Code::LIT(f) => {
                if f.short {
                    uxn.counter += 1;
                    uxn.work.push(main[uxn.counter as usize]);
                    uxn.counter += 1;
                    uxn.work.push(main[uxn.counter as usize]);
                } else {
                    uxn.counter += 1;
                    uxn.work.push(main[uxn.counter as usize]);
                }
            },
            Code::DEO(_f) => {
                let _device = uxn.work.pop().unwrap();
                let value = uxn.work.pop().unwrap();
                out.write(&[value])?;
                out.flush()?;
            },
            Code::BRK => {
                break;
            }
        };
        uxn.counter += 1;
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
