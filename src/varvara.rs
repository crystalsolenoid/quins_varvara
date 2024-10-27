pub trait Device {
    fn deo(&mut self, port: u8, value: u8); // port must be 0..16
}
