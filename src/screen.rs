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
        let mode = (byte & 0b1000_0000).count_ones();
        match mode {
            0 => self.draw_sprite_1bpp(byte, mem),
            1 => self.draw_sprite_2bpp(byte, mem),
            _ => panic!("binary violated"),
        };
    }

    fn draw_sprite_2bpp(&mut self, byte: u8, mem: &[u8]) {
        let high_data = read_bytes(mem, self.addr, 8);
        let low_data = read_bytes(mem, self.addr + 8, 8);
        let color_set = byte & 0b0000_1111;
        let color = match color_set {
            0x0 => (0, 0, 1, 2),
            0x1 => (0, 1, 2, 3),
            0x2 => (0, 2, 3, 1),
            0x3 => (0, 3, 1, 2),
            0x4 => (1, 0, 1, 2),
            0x5 => (0, 1, 2, 3), // TODO add transparency
            0x6 => (1, 2, 3, 1),
            0x7 => (1, 3, 1, 2),
            0x8 => (2, 0, 1, 2),
            0x9 => (2, 1, 2, 3),
            0xa => (0, 2, 3, 1), // TODO add transparency
            0xb => (2, 3, 1, 2),
            0xc => (3, 0, 1, 2),
            0xd => (3, 1, 2, 3),
            0xe => (3, 2, 3, 1),
            0xf => (0, 3, 1, 2), // TODO add transparency
            _ => panic!("violated binary"),
        };

        let index = self.x as usize + WIDTH * self.y as usize;
        (0..8).for_each(|y| {
            let mut pixel_mask = 0b10000000;
            self.buffer[y * WIDTH + index..y * WIDTH + index + 8]
                .iter_mut()
                .for_each(|p| {
                    let high = (pixel_mask & high_data[y]).count_ones();
                    let low = (pixel_mask & low_data[y]).count_ones();
                    let color = match (high, low) {
                        (0, 0) => color.0,
                        (0, 1) => color.1,
                        (1, 0) => color.2,
                        (1, 1) => color.3,
                        _ => panic!("binary math failed"),
                    };
                    *p = color;
                    pixel_mask = pixel_mask >> 1;
                });
        });
    }

    fn draw_sprite_1bpp(&mut self, byte: u8, mem: &[u8]) {
        let sprite_data = read_bytes(mem, self.addr, 8);
        let _layer = byte & 0b0100_0000;
        let _flip_y = byte & 0b0010_0000;
        let _flip_x = byte & 0b0001_0000;
        let fg_color = byte & 0b0000_0011;
        let bg_color = (byte & 0b0000_1100) >> 2;
        let index = self.x as usize + WIDTH * self.y as usize;
        (0..8).for_each(|y| {
            let mut pixel_mask = 0b10000000;
            self.buffer[y * WIDTH + index..y * WIDTH + index + 8]
                .iter_mut()
                .for_each(|p| {
                    let pixel = pixel_mask & sprite_data[y];
                    let color = match pixel.count_ones() {
                        0 => bg_color,
                        1 => fg_color,
                        _ => panic!("binary math failed"),
                    };
                    *p = color;
                    pixel_mask = pixel_mask >> 1;
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
