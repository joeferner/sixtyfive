use crate::code::{AsmCode, Code, Instruction};

use super::DisassembleError;

pub struct Disassembler {
    pub code: Code,
}

impl Disassembler {
    pub fn new(data: Vec<u8>) -> Disassembler {
        return Disassembler {
            code: Code::new(data),
        };
    }

    pub fn disassemble<F1: Fn(u16) -> usize, F2: Fn(usize) -> u16>(
        &mut self,
        addr: u16,
        name: &str,
        label_prefix: &str,
        addr_to_offset_fn: &F1,
        offset_to_addr_fn: &F2,
    ) -> Result<(), DisassembleError> {
        let mut offset = addr_to_offset_fn(addr);
        self.code
            .set_label(offset, format!("{}_{}", label_prefix, name).as_str());

        loop {
            if self.code.is_used(offset) {
                break;
            }

            let op = self.code.get_u8(offset)?;
            let result = match op {
                // ASL ZP
                0x06 => self.code.replace_with_instr(offset, 1, |args| {
                    Result::Ok(Instruction::ASL_ZP(args[0].to_u8()?))
                }),

                // JSR ABS
                0x20 => {
                    let l = self.code.get_u8(offset + 1)? as u16;
                    let h = self.code.get_u8(offset + 2)? as u16;
                    let jsr_addr = (h << 8) | l;
                    let label = format!("{}_{:04x}", label_prefix, jsr_addr);
                    let jsr_result = self.code.replace_with_instr(offset, 2, |_| {
                        Result::Ok(Instruction::JSR_ABS(jsr_addr, label.clone()))
                    });

                    // disassemble jump address
                    self.disassemble(
                        jsr_addr,
                        format!("{:04x}", jsr_addr).as_str(),
                        label_prefix,
                        addr_to_offset_fn,
                        offset_to_addr_fn,
                    )?;

                    jsr_result
                }

                // AND IMM
                0x29 => self.code.replace_with_instr(offset, 1, |args| {
                    Result::Ok(Instruction::AND_IMM(args[0].to_u8()?))
                }),

                // PHA
                0x48 => self
                    .code
                    .replace_with_instr(offset, 0, |_| Result::Ok(Instruction::PHA)),

                // STA ZP
                0x85 => self.code.replace_with_instr(offset, 1, |args| {
                    Result::Ok(Instruction::STA_ZP(args[0].to_u8()?))
                }),

                // DEY
                0x88 => self
                    .code
                    .replace_with_instr(offset, 0, |_args| Result::Ok(Instruction::DEY)),

                // STA ABS
                0x8d => self.code.replace_with_instr(offset, 2, |args| {
                    Result::Ok(Instruction::STA_ABS(to_u16(&args[0], &args[1])?))
                }),

                // LDY IMM
                0xa0 => self.code.replace_with_instr(offset, 1, |args| {
                    Result::Ok(Instruction::LDY_IMM(args[0].to_u8()?))
                }),

                // LDA ZP
                0xa5 => self.code.replace_with_instr(offset, 1, |args| {
                    Result::Ok(Instruction::LDA_ZP(args[0].to_u8()?))
                }),

                // LDA IMM
                0xa9 => self.code.replace_with_instr(offset, 1, |args| {
                    Result::Ok(Instruction::LDA_IMM(args[0].to_u8()?))
                }),

                // LDX ABS
                0xae => self.code.replace_with_instr(offset, 2, |args| {
                    Result::Ok(Instruction::LDX_ABS(to_u16(&args[0], &args[1])?))
                }),

                // LDA IND Y
                0xb1 => self.code.replace_with_instr(offset, 1, |args| {
                    Result::Ok(Instruction::LDA_IND_Y(args[0].to_u8()?))
                }),

                // CPY IMM
                0xc0 => self.code.replace_with_instr(offset, 1, |args| {
                    Result::Ok(Instruction::CPY_IMM(args[0].to_u8()?))
                }),

                // BNE REL
                0xd0 => {
                    let rel = self.code.get_i8(offset)?;
                    let bne_addr = addr.wrapping_add(rel as u16) + 2 - 8; // TODO where does the 8 come from
                    let label = format!("{}_{:04x}", label_prefix, bne_addr);
                    let bne_result = self.code.replace_with_instr(offset, 1, |_| {
                        Result::Ok(Instruction::BNE_REL(label.clone()))
                    });

                    // disassemble jump address
                    self.disassemble(
                        bne_addr,
                        format!("{:04x}", bne_addr).as_str(),
                        label_prefix,
                        addr_to_offset_fn,
                        offset_to_addr_fn,
                    )?;

                    bne_result
                }

                // Other
                _ => {
                    // TODO i => return Result::Err(DisassembleError::UnhandledInstruction(i))
                    println!("unhandled: 0x{:02x}", op);
                    break;
                }
            };

            match result {
                Result::Ok(size) => {
                    offset += size;
                }
                Result::Err(err) => {
                    return Result::Err(DisassembleError::WrappedError(format!(
                        "{} at offset ${:04x} (addr ${:04x})",
                        err,
                        offset,
                        offset_to_addr_fn(offset)
                    )));
                }
            }
        }

        return Result::Ok(());
    }
}

fn to_u16(arg0: &AsmCode, arg1: &AsmCode) -> Result<u16, DisassembleError> {
    let l = arg0.to_u8()? as u16;
    let h = arg1.to_u8()? as u16;
    return Result::Ok((h << 8) | l);
}
