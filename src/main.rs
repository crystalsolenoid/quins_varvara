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
        match main[counter as usize] {
            0x18 => { // ADD
                let b = work.pop().unwrap();
                let a = work.pop().unwrap();
                work.push(a + b);
            },
            0x19 => { // SUB
                let b = work.pop().unwrap();
                let a = work.pop().unwrap();
                work.push(a - b);
            },
            0x80 => { // LIT
                counter += 1;
                work.push(main[counter as usize]);
            },
            0x17 => { // DEO
                let _device = work.pop().unwrap();
                let value = work.pop().unwrap();
                out.write(&[value])?;
                out.flush()?;
            },
            0x00 => { // BRK
                break;
            }
            code => panic!("Unexpected instruction code {code}")
        };
        counter += 1;
    }

    Ok(())
}

fn print_bytes(data: &[u8]) {
    println!("{:0>2x?}", &data);
}
