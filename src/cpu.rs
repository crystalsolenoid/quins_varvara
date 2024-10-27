use super::varvara::Varvara;

pub struct Stack {
    bytes: Vec::<u8>,
}

impl Stack {
    pub fn new() -> Self {
        Self { bytes: Vec::with_capacity(0xFF) }
    }

    pub fn pop(&mut self) -> u8 {
        self.bytes.pop().unwrap()
    }

    pub fn pop2(&mut self) -> u16 {
        let low_byte = self.pop();
        let high_byte = self.pop();
        u16::from_be_bytes([high_byte, low_byte])
    }

    pub fn push(&mut self, byte: u8) {
        self.bytes.push(byte);
    }

    pub fn push2(&mut self, short: u16) {
        let [high, low] = short.to_be_bytes();
        self.push(high);
        self.push(low);
    }
}

pub struct Cpu {
    /// Working stack
    pub work: Stack,
    /// Return stack
    pub ret: Stack,
    /// Instruction pointer
    pub counter: u16,
}

impl Cpu {
    pub fn new() -> Self {
        let work = Stack::new();
        let ret = Stack::new();
        let counter = 0x0100;
        Self { work, ret, counter }
    }

    pub fn next_byte(&mut self, varvara: &Varvara) -> u8 {
        let byte = varvara.main[self.counter as usize];
        self.counter = self.counter.wrapping_add(1);
        byte
    }

    pub fn next_short(&mut self, varvara: &Varvara) -> u16 {
        let high_byte = self.next_byte(varvara);
        let low_byte = self.next_byte(varvara);
        return u16::from_be_bytes([high_byte, low_byte])
    }

//    pub fn operate(&mut varvara: Varvara) {
//
//    }
}
