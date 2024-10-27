use super::varvara::Varvara;

pub struct CodeFlags {
    pub keep: bool,
    pub ret: bool,
    pub short: bool,
}

pub struct LitFlags {
    pub ret: bool,
    pub short: bool,
}

pub enum Code {
    BRK,
    DEO(CodeFlags),
    ADD(CodeFlags),
    SUB(CodeFlags),
    LIT(LitFlags),
}

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

    /// Do one operation
    pub fn step(&mut self, varvara: &mut Varvara) {
        let raw_code = self.next_byte(varvara);
        todo!();
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
    pub fn deo(&mut self, f: CodeFlags) {
        let _device = self.work.pop();
        let value = self.work.pop();
        todo!();
        //out.write(&[value])?;
        //out.flush()?;
    }
}

pub fn parse_code(byte: u8) -> Code {
    let code = 0b000_11111 & byte;
    let short = 0b001_00000 & byte != 0;
    let ret = 0b010_00000 & byte != 0;
    if ret { todo!("Return flag not yet implemented! Code: {byte:0>2x?}"); }
    let keep = 0b100_00000 & byte != 0;
    if keep && code != 0x00 { todo!("Keep flag not yet implemented! Code: {byte:0>2x?}"); }

    let flags = CodeFlags {keep, ret, short};
    match code {
        0x00 => if keep {
                Code::LIT(LitFlags {ret, short})
            } else {
                Code::BRK
            },
        0x17 => Code::DEO(flags),
        0x18 => Code::ADD(flags),
        0x19 => Code::SUB(flags),
        _ => todo!("Missing opcode! Code: {byte:0>2x?}"),
    }
}
