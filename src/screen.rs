use super::varvara::{Varvara, Device};

pub struct Screen {
    pub buffer: Vec<u32>
}

const WIDTH: usize = 512;
const HEIGHT: usize = 320;

impl Screen {
    pub fn new() -> Self {
        let buffer = vec![0; WIDTH * HEIGHT];
        Self { buffer }
    }
}

impl Device for Screen {
    fn notify_deo(&mut self, io: &[u8], addr: u8, byte: u8) {
        let port = addr & 0x0F;
        let _ = match port {
            0x8 => todo!(),
            _ => panic!("Don't know how to write to port {port}!")
        };
    }

    fn notify_deo2(&mut self, io: &[u8], _addr: u8, _short: u16) {
        panic!("You can't write a short to the console.");
    }
}
