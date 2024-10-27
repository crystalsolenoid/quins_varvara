use std::io;
use std::io::prelude::*;
use std::fs::File;

use uxn::cpu::{self, Cpu, CodeFlags, LitFlags, Code};
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
        let code = cpu::parse_code(raw_code);
        match code {
            Code::ADD(f) => uxn.add(f),
            Code::SUB(f) => uxn.sub(f),
            Code::LIT(f) => uxn.lit(f, &varvara),
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
