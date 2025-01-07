use std::fs::File;
use std::io;
use std::io::prelude::*;

use minifb::Key;

use uxn::cpu::Cpu;
use uxn::tal;
use uxn::varvara::Varvara;

fn main() -> io::Result<()> {
    let mut varvara = Varvara::new();
    let mut uxn = Cpu::new();

    tal::assemble(
        "roms/test/hello_2bpp_sprites_sq.tal",
        "roms/test/hello_2bpp_sprites_sq.rom",
    )
    .expect("failed to assemble");

    let rom_load_area = &mut varvara.main[0x0100..];
    let mut file =
        File::open("roms/test/hello_2bpp_sprites_sq.rom").expect("failed to open rom file");
    let _n = file.read(rom_load_area).expect("failed to read rom file");

    for i in varvara.screen.buffer.iter_mut() {
        *i = 0x00;
    }

    loop {
        let terminate = uxn.step(&mut varvara);
        if terminate {
            break;
        }
    }

    while varvara.window.is_open() && !varvara.window.is_key_down(Key::Escape) {
        varvara.update_window();
    }

    Ok(())
}

fn _print_bytes(data: &[u8]) {
    println!("{:0>2x?}", &data);
}
