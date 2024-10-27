use super::system::System;
use super::console::Console;

pub trait Device {
    fn deo(&mut self, port: u8, value: u8); // port must be 0..16
}

pub struct Varvara {
    pub main: [u8; 0xFFFF],
    pub io: [u8; 0xFF],
    pub system: System,
    pub console: Console,
}

impl Varvara {
    pub fn new() -> Self {
        let main = [0; 0xFFFF];
        let io = [0; 0xFF];
        let system = System::new();
        let console = Console::new();
        Self { main, io, system, console }
    }

    pub fn deo(&mut self, addr: u8, byte: u8) {
        self.io[addr as usize] = byte;
        match addr {
            0x00..0x10 => self.system.deo(addr, byte),
            0x10..0x20 => self.console.deo(addr, byte),
            //0x20..0x30 => self.screen,
            _ => todo!(),
        }
    }
}
