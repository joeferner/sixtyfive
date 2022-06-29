use crate::code::Code;

pub struct Disassembler {
    pub code: Code,
}

impl Disassembler {
    pub fn new(data: Vec<u8>) -> Disassembler {
        return Disassembler {
            code: Code::new(data),
        };
    }
}
