use super::system::System;
use super::console::Console;

pub trait Device {
    fn notify_deo(&mut self, io: &[u8], port: u8, value: u8) {}
    fn notify_deo2(&mut self, io: &[u8], addr: u8, short: u16) {}
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
            0x00..0x10 => self.system.notify_deo(&self.io, addr, byte),
            0x10..0x20 => self.console.notify_deo(&self.io, addr, byte),
            //0x20..0x30 => self.screen,
            _ => todo!(),
        }
    }

    pub fn deo2(&mut self, addr: u8, short: u16) {
        write_short(&mut self.io, addr, short);
        match addr {
            // panicking if 0x_F because writing a short to that address would
            // mean writing half to one device and half to another
            0x00..0x0F => self.system.notify_deo2(&self.io, addr, short),
            0x10..0x1F => self.console.notify_deo2(&self.io, addr, short),
            _ => todo!(),
        }
    }
}

pub fn write_short(mem: &mut [u8], addr: u8, short: u16) {
    let addr_high = addr;
    let addr_low = addr.wrapping_add(1);
    let [high, low] = short.to_be_bytes();
    mem[addr_high as usize] = high;
    mem[addr_low as usize] = low;
}
