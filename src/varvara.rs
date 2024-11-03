use minifb::{Key, Window, WindowOptions};

use super::system::System;
use super::console::Console;
use super::screen::Screen;

pub trait Device {
    fn notify_deo(&mut self, io: &[u8], port: u8, value: u8) {}
    fn notify_deo2(&mut self, io: &[u8], addr: u8, short: u16) {}
}

pub struct Varvara {
    pub main: [u8; 0xFFFF],
    pub io: [u8; 0xFF],
    pub system: System,
    pub console: Console,
    pub screen: Screen,

    pub window: Window,
}

const WIDTH: usize = 512;
const HEIGHT: usize = 320;

impl Varvara {
    pub fn new() -> Self {
        let main = [0; 0xFFFF];
        let io = [0; 0xFF];
        let system = System::new();
        let console = Console::new();
        let screen = Screen::new();

        let mut window = Window::new(
            "Test - ESC to exit",
            WIDTH,
            HEIGHT,
            WindowOptions{
                scale: minifb::Scale::X4,
                ..WindowOptions::default()
            },
        )
        .unwrap_or_else(|e| {
            panic!("{}", e);
        });
        window.set_target_fps(30);

        Self { main, io, system, console, screen, window }
    }

    pub fn deo(&mut self, addr: u8, byte: u8) {
        self.io[addr as usize] = byte;
        match addr {
            0x00..0x10 => self.system.notify_deo(&self.io, addr, byte),
            0x10..0x20 => self.console.notify_deo(&self.io, addr, byte),
            0x20..0x30 => self.screen.notify_deo(&self.io, addr, byte),
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
            0x20..0x2F => self.screen.notify_deo2(&self.io, addr, short),
            _ => todo!(),
        }
    }

    pub fn dei(&self, addr: u8) -> u8 {
        self.io[addr as usize]
    }

    pub fn dei2(&self, addr: u8) -> u16 {
        read_short(&self.io, addr)
    }

    pub fn update_window(&mut self) {
        let rgb_buffer: Vec<_> = self.screen.buffer.iter()
            .map(|&i| self.system.index_to_0rgb(i))
            .collect();
        self.window
            .update_with_buffer(&rgb_buffer, WIDTH, HEIGHT)
            .unwrap();
    }
}

pub fn write_short(mem: &mut [u8], addr: u8, short: u16) {
    let addr_high = addr;
    let addr_low = addr.wrapping_add(1);
    let [high, low] = short.to_be_bytes();
    mem[addr_high as usize] = high;
    mem[addr_low as usize] = low;
}

pub fn read_short(mem: &[u8], addr: u8) -> u16 {
    let addr_high = addr;
    let addr_low = addr.wrapping_add(1);
    let high = mem[addr_high as usize];
    let low = mem[addr_low as usize];
    u16::from_be_bytes([high, low])
}
