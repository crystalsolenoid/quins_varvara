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
        let terminate = uxn.step(&mut varvara);
        if terminate { break; }
    }

    Ok(())
}

fn print_bytes(data: &[u8]) {
    println!("{:0>2x?}", &data);
}
