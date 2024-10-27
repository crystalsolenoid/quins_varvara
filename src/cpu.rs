//use super::varvara::Varvara;

pub struct Cpu {
    /// Working stack
    pub work: Vec::<u8>,
    /// Return stack
    pub ret: Vec::<u8>,
    /// Instruction pointer
    pub counter: u16,
}

impl Cpu {
    pub fn new() -> Self {
        let work = Vec::with_capacity(0xFF);
        let ret = Vec::with_capacity(0xFF);
        let counter = 0x0100;
        Self { work, ret, counter }
    }

//    pub fn operate(&mut varvara: Varvara) {
//
//    }
}
