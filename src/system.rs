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
}

impl Device for System {
    fn deo(&mut self, port: u8, value: u8) {
        match port {
            0xe => todo!("debug port"),
            0xf => todo!("state port"),
            _ => panic!("Don't know how to write to port {port}!")
        };
    }
}
