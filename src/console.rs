use std::io;
use std::io::prelude::*;

use super::varvara::Device;

struct Console {
    out: std::io::Stdout,
}

impl Console {
    pub fn new() -> Self {
        let out = std::io::stdout();
        Self { out }
    }

    fn write(&mut self, byte: u8) -> io::Result<()> {
        self.out.write(&[byte])?;
        self.out.flush()?;
        Ok(())
    }
}

impl Device for Console {
    fn deo(&mut self, port: u8, value: u8) {
        match port {
            0x7 => self.write(value),
            _ => panic!("Don't know how to write to port {port}!")
        };
    }
}
