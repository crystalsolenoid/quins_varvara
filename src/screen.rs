use super::varvara::Device;

pub struct Screen {
    pub buffer: Vec<u8>,
    x: u16,
    y: u16,
}

const WIDTH: usize = 512;
const HEIGHT: usize = 320;

impl Screen {
    pub fn new() -> Self {
        let buffer = vec![0; WIDTH * HEIGHT];
        Self { buffer, x: 0, y: 0 }
    }

    pub fn draw_pixel(&mut self, byte: u8) {
        let color = 0b00000011 & byte;
        let index = self.x as usize + WIDTH * self.y as usize;
        self.buffer[index] = color;
    }
}

impl Device for Screen {
    fn notify_deo(&mut self, _io: &[u8], addr: u8, byte: u8) {
        let port = addr & 0x0F;
        let _ = match port {
            0xe => self.draw_pixel(byte),
            _ => panic!("Don't know how to write to port {port}!"),
        };
    }

    fn notify_deo2(&mut self, _io: &[u8], addr: u8, short: u16) {
        let port = addr & 0x0F;
        match port {
            0x8 => self.x = short,
            0xa => self.y = short,
            _ => panic!("Don't know how to write to port {port}!"),
        };
    }
}
