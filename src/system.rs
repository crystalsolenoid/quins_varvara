use std::io;

use super::varvara::Device;

pub struct System {
    _out: std::io::Stdout,
    colors: [u32; 4],
}

impl System {
    pub fn new() -> Self {
        let out = std::io::stdout();
        let colors = [0xFFFFFF, 0x000000, 0x77ddbb, 0xff6622];
        Self { _out: out, colors }
    }

    fn _print_debug(&mut self) -> io::Result<()> {
        todo!();
    }

    fn update_color(&mut self, io: &[u8]) {
        let reds = u16::from_be_bytes([io[0x8], io[0x9]]);
        let greens = u16::from_be_bytes([io[0xa], io[0xb]]);
        let blues = u16::from_be_bytes([io[0xc], io[0xd]]);
        self.colors = [0, 1, 2, 3].map(|i| shorts_to_0rgb(reds, greens, blues, i));
    }

    pub fn index_to_0rgb(&self, color: u8) -> u32 {
        self.colors[color as usize] as u32
    }
}

impl Device for System {
    fn notify_deo(&mut self, _io: &[u8], addr: u8, _byte: u8) {
        let port = addr & 0x0F;
        match port {
            0xe => todo!("debug port"),
            0xf => todo!("state port"),
            _ => panic!("Don't know how to write to port {port}!"),
        };
    }

    fn notify_deo2(&mut self, io: &[u8], addr: u8, _short: u16) {
        let port = addr & 0x0F;
        match port {
            0x7..0xe => self.update_color(&io),
            _ => (),
        };
    }
}

fn shorts_to_0rgb(reds: u16, greens: u16, blues: u16, index: usize) -> u32 {
    let shift = 12 - 4 * index;
    let red = (reds as u32 >> shift) & 0xF;
    let green = (greens as u32 >> shift) & 0xF;
    let blue = (blues as u32 >> shift) & 0xF;

    let mut color = 0;
    color |= red << 20 | red << 16;
    color |= green << 12 | green << 8;
    color |= blue << 4 | blue;
    color
}
