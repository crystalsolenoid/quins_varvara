use super::varvara::{read_bytes, Device};

pub struct Screen {
    pub buffer: Vec<u8>,
    x: u16,
    y: u16,
    addr: u16,
}

const WIDTH: usize = 512;
const HEIGHT: usize = 320;

impl Screen {
    pub fn new() -> Self {
        let buffer = vec![0; WIDTH * HEIGHT];
        Self {
            buffer,
            x: 0,
            y: 0,
            addr: 0,
        }
    }

    pub fn draw_pixel(&mut self, byte: u8) {
        let color = 0b00000011 & byte;
        let index = self.x as usize + WIDTH * self.y as usize;
        self.buffer[index] = color;
    }

    pub fn draw_sprite(&mut self, byte: u8, mem: &[u8]) {
        let sprite_data = read_bytes(mem, self.addr, 8);
        let _mode = byte & 0b1000_0000;
        let _layer = byte & 0b0100_0000;
        let _flip_y = byte & 0b0010_0000;
        let _flip_x = byte & 0b0001_0000;
        let fg_color = byte & 0b0000_0011;
        let bg_color = (byte & 0b0000_1100) >> 2;
        let index = self.x as usize + WIDTH * self.y as usize;
        (0..8).for_each(|y| {
            let mut pixel_mask = 1;
            self.buffer[y * WIDTH + index..y * WIDTH + index + 8]
                .iter_mut()
                .enumerate()
                .for_each(|(x, p)| {
                    let pixel = (pixel_mask & sprite_data[y]) >> x;
                    let color = match pixel {
                        0 => bg_color,
                        1 => fg_color,
                        _ => panic!("binary math failed"),
                    };
                    *p = color;
                    pixel_mask = pixel_mask << 1;
                });
        });
    }
}

impl Default for Screen {
    fn default() -> Self {
        Self::new()
    }
}

impl Device for Screen {
    fn notify_deo(&mut self, _io: &[u8], main: &[u8], addr: u8, byte: u8) {
        let port = addr & 0x0F;
        match port {
            0xe => self.draw_pixel(byte),
            0xf => self.draw_sprite(byte, main),
            _ => panic!("Don't know how to write to port {port}!"),
        };
    }

    fn notify_deo2(&mut self, _io: &[u8], _main: &[u8], addr: u8, short: u16) {
        let port = addr & 0x0F;
        match port {
            0x8 => self.x = short,
            0xa => self.y = short,
            0xc => self.addr = short,
            _ => panic!("Don't know how to write to port {port}!"),
        };
    }
}
