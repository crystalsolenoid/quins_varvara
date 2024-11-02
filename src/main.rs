use std::io;
use std::io::prelude::*;
use std::fs::File;

use minifb::{Key, Window, WindowOptions};

use uxn::cpu::Cpu;
use uxn::varvara::Varvara;

const WIDTH: usize = 640;
const HEIGHT: usize = 360;

fn main() -> io::Result<()> {
    let mut buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];
    let mut window = Window::new(
        "Test - ESC to exit",
        WIDTH,
        HEIGHT,
        WindowOptions::default(),
    )
    .unwrap_or_else(|e| {
        panic!("{}", e);
    });

    window.set_target_fps(30);

    let mut varvara = Varvara::new();
    let mut uxn = Cpu::new();

    let rom_load_area = &mut varvara.main[0x0100..];
    let mut file = File::open("roms/test/SUB2_wrap.rom").expect("failed to open rom file");
    let n = file.read(rom_load_area).expect("failed to read rom file");
    print_bytes(&rom_load_area[..n]);

    while window.is_open() && !window.is_key_down(Key::Escape) {
        loop {
            let terminate = uxn.step(&mut varvara);
            if terminate { break; }
        }

        for i in buffer.iter_mut() {
            *i = 0;
        }

        window
            .update_with_buffer(&buffer, WIDTH, HEIGHT)
            .unwrap();
    }

    Ok(())
}

fn print_bytes(data: &[u8]) {
    println!("{:0>2x?}", &data);
}
