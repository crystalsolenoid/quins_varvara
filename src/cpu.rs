use super::opcode::{parse_code, Code, CodeFlags, LitFlags};
use super::varvara::Varvara;

pub struct Stack {
    bytes: Vec<u8>,
}

impl Stack {
    pub fn new() -> Self {
        Self {
            bytes: Vec::with_capacity(0xFF),
        }
    }

    pub fn peek_mut(&mut self) -> &mut u8 {
        self.bytes.last_mut().unwrap()
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

impl Default for Stack {
    fn default() -> Self {
        Self::new()
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
        u16::from_be_bytes([high_byte, low_byte])
    }

    /// Do one operation
    pub fn step(&mut self, varvara: &mut Varvara) -> bool {
        let raw_code = self.next_byte(varvara);
        let code = parse_code(raw_code);
        match code {
            Code::INC(f) => self.inc(f),
            Code::ADD(f) => self.add(f),
            Code::SUB(f) => self.sub(f),
            Code::LIT(f) => self.lit(f, varvara),
            Code::DEO(f) => self.deo(f, varvara),
            Code::DEI(f) => self.dei(f, varvara),
            Code::BRK => return true,
        }
        false
    }

    /// Execute INC
    pub fn inc(&mut self, f: CodeFlags) {
        if f.short {
            let a = self.work.pop2();
            self.work.push2(a.wrapping_add(1));
        } else {
            let top = self.work.peek_mut();
            *top = top.wrapping_add(1);
        }
    }

    /// Execute ADD
    pub fn add(&mut self, f: CodeFlags) {
        if f.short {
            let b = self.work.pop2();
            let a = self.work.pop2();
            self.work.push2(a.wrapping_add(b));
        } else {
            let b = self.work.pop();
            let a = self.work.pop();
            self.work.push(a.wrapping_add(b));
        }
    }

    /// Execute SUB
    pub fn sub(&mut self, f: CodeFlags) {
        if f.short {
            let b = self.work.pop2();
            let a = self.work.pop2();
            self.work.push2(a.wrapping_sub(b));
        } else {
            let b = self.work.pop();
            let a = self.work.pop();
            self.work.push(a.wrapping_sub(b));
        }
    }

    /// Execute LIT
    pub fn lit(&mut self, f: LitFlags, varvara: &Varvara) {
        if f.short {
            let short = self.next_short(varvara);
            self.work.push2(short);
        } else {
            let byte = self.next_byte(varvara);
            self.work.push(byte);
        }
    }

    /// Execute DEO
    pub fn deo(&mut self, f: CodeFlags, varvara: &mut Varvara) {
        let addr = self.work.pop();
        if f.short {
            let short = self.work.pop2();
            varvara.deo2(addr, short);
        } else {
            let byte = self.work.pop();
            varvara.deo(addr, byte);
        }
    }

    /// Execute DEI
    pub fn dei(&mut self, f: CodeFlags, varvara: &mut Varvara) {
        let addr = self.work.pop();
        if f.short {
            self.work.push2(varvara.dei2(addr));
        } else {
            self.work.push(varvara.dei(addr));
        }
    }
}

impl Default for Cpu {
    fn default() -> Self {
        Self::new()
    }
}
