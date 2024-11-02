use std::io;
use std::io::prelude::*;

use super::varvara::Device;

pub struct System {
    out: std::io::Stdout,
    colors: [u32; 4],
}

impl System {
    pub fn new() -> Self {
        let out = std::io::stdout();
        let colors = [0xFFFFFF, 0x000000, 0x77ddbb, 0xff6622];
        Self { out, colors }
    }

    fn print_debug(&mut self) -> io::Result<()> {
        todo!();
    }

    fn update_color(&mut self) {
    }

    pub fn index_to_0rgb(&self, color: u8) -> u32 {
        self.colors[color as usize] as u32
    }
}

impl Device for System {
    fn notify_deo(&mut self, io: &[u8], addr: u8, byte: u8) {
        let port = addr & 0x0F;
        match port {
            0xe => todo!("debug port"),
            0xf => todo!("state port"),
            _ => panic!("Don't know how to write to port {port}!")
        };
    }

    fn notify_deo2(&mut self, io: &[u8], addr: u8, short: u16) {
        let port = addr & 0x0F;
        match port {
            0x7..0xe => self.update_color(),
            _ => (),
        };
    }
}
