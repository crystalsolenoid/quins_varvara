use std::io;
use std::io::prelude::*;

use super::varvara::Device;

pub struct Console {
    out: std::io::Stdout,
}

impl Console {
    pub fn new() -> Self {
        let out = std::io::stdout();
        Self { out }
    }

    fn write(&mut self, byte: u8) -> io::Result<()> {
        self.out.write_all(&[byte])?;
        self.out.flush()?;
        Ok(())
    }
}

impl Default for Console {
    fn default() -> Self {
        Self::new()
    }
}

impl Device for Console {
    fn notify_deo(&mut self, _io: &[u8], _main: &[u8], addr: u8, byte: u8) {
        let port = addr & 0x0F;
        let _ = match port {
            0x8 => self.write(byte),
            _ => panic!("Don't know how to write to port {port}!"),
        };
    }

    fn notify_deo2(&mut self, _io: &[u8], _main: &[u8], _addr: u8, _short: u16) {
        panic!("You can't write a short to the console.");
    }
}
