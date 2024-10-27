pub trait Device {
    fn deo(&mut self, port: u8, value: u8); // port must be 0..16
}

pub struct Varvara {
    pub main: [u8; 0xFFFF],
    pub io: [u8; 0xFF],
}

impl Varvara {
    pub fn new() -> Self {
        let main = [0; 0xFFFF];
        let io = [0; 0xFF];
        Self { main, io }
    }
}
