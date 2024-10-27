use std::io;
use std::io::prelude::*;

use super::varvara::Device;

pub struct System {
    out: std::io::Stdout
}

impl System {
    pub fn new() -> Self {
        let out = std::io::stdout();
        Self { out }
    }

    fn print_debug(&mut self) -> io::Result<()> {
        todo!();
    }

    fn update_color(&mut self) {
        todo!();
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
