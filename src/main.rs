use std::io;
use std::io::prelude::*;
use std::fs::File;

use minifb::{Key};

use uxn::cpu::Cpu;
use uxn::varvara::Varvara;

fn main() -> io::Result<()> {
    let mut varvara = Varvara::new();
    let mut uxn = Cpu::new();

    let rom_load_area = &mut varvara.main[0x0100..];
    let mut file = File::open("roms/test/SUB2_wrap.rom").expect("failed to open rom file");
    let n = file.read(rom_load_area).expect("failed to read rom file");
    print_bytes(&rom_load_area[..n]);

    while varvara.window.is_open() && !varvara.window.is_key_down(Key::Escape) {
        loop {
            let terminate = uxn.step(&mut varvara);
            if terminate { break; }
        }

        for i in varvara.screen.buffer.iter_mut() {
            *i = 0;
        }

        varvara.update_window();
    }

    Ok(())
}

fn print_bytes(data: &[u8]) {
    println!("{:0>2x?}", &data);
}
