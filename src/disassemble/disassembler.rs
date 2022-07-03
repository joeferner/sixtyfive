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
        let mut addr = addr;
        let mut offset = addr_to_offset_fn(addr);
        self.code
            .set_label(offset, format!("{}_{}", label_prefix, name).as_str());

        loop {
            if self.code.is_instruction(offset) {
                break;
            }

            let op = self.code.get_u8(offset)?;
            let result = match op {
                // ORA ZP
                0x05 => self.code.replace_with_instr(offset, 1, |args| {
                    Result::Ok(Instruction::ORA_ZP(args[0].to_u8()?))
                }),

                // ASL ZP
                0x06 => self.code.replace_with_instr(offset, 1, |args| {
                    Result::Ok(Instruction::ASL_ZP(args[0].to_u8()?))
                }),

                // ORA IMM
                0x09 => self.code.replace_with_instr(offset, 1, |args| {
                    Result::Ok(Instruction::ORA_IMM(args[0].to_u8()?))
                }),

                // ASL
                0x0a => self
                    .code
                    .replace_with_instr(offset, 0, |_args| Result::Ok(Instruction::ASL)),

                // JSR ABS
                0x20 => {
                    let l = self.code.get_u8(offset + 1)? as u16;
                    let h = self.code.get_u8(offset + 2)? as u16;
                    let jsr_addr = (h << 8) | l;
                    let label = format!("{}_{:04x}", label_prefix, jsr_addr);
                    let jsr_result = self.code.replace_with_instr(offset, 2, |_args| {
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

                // ROL
                0x2a => self
                    .code
                    .replace_with_instr(offset, 0, |_args| Result::Ok(Instruction::ROL)),

                // SEC
                0x38 => self
                    .code
                    .replace_with_instr(offset, 0, |_args| Result::Ok(Instruction::SEC)),

                // PHA
                0x48 => self
                    .code
                    .replace_with_instr(offset, 0, |_args| Result::Ok(Instruction::PHA)),

                // LSR
                0x4a => self
                    .code
                    .replace_with_instr(offset, 0, |_args| Result::Ok(Instruction::LSR)),

                // RTS
                0x60 => {
                    self.code
                        .replace_with_instr(offset, 0, |_args| Result::Ok(Instruction::RTS))?;
                    Result::Ok(0)
                }

                // ADC ZP
                0x65 => self.code.replace_with_instr(offset, 1, |args| {
                    Result::Ok(Instruction::ADC_ZP(args[0].to_u8()?))
                }),

                // PLA
                0x68 => self
                    .code
                    .replace_with_instr(offset, 0, |_args| Result::Ok(Instruction::PLA)),

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

                // BCC
                0x90 => self.branch_relative(
                    offset,
                    addr,
                    label_prefix,
                    addr_to_offset_fn,
                    offset_to_addr_fn,
                    &|rel, label| Instruction::BCC_REL(rel, label),
                ),

                // TYA
                0x98 => self
                    .code
                    .replace_with_instr(offset, 0, |_args| Result::Ok(Instruction::TYA)),

                // LDY IMM
                0xa0 => self.code.replace_with_instr(offset, 1, |args| {
                    Result::Ok(Instruction::LDY_IMM(args[0].to_u8()?))
                }),

                // LDX IMM
                0xa2 => self.code.replace_with_instr(offset, 1, |args| {
                    Result::Ok(Instruction::LDX_IMM(args[0].to_u8()?))
                }),

                // LDA ZP
                0xa5 => self.code.replace_with_instr(offset, 1, |args| {
                    Result::Ok(Instruction::LDA_ZP(args[0].to_u8()?))
                }),

                // LDA IMM
                0xa9 => self.code.replace_with_instr(offset, 1, |args| {
                    Result::Ok(Instruction::LDA_IMM(args[0].to_u8()?))
                }),

                // TAX
                0xaa => self
                    .code
                    .replace_with_instr(offset, 0, |_args| Result::Ok(Instruction::TAX)),

                // LDX ABS
                0xae => self.code.replace_with_instr(offset, 2, |args| {
                    Result::Ok(Instruction::LDX_ABS(to_u16(&args[0], &args[1])?))
                }),

                // BCS REL
                0xb0 => self.branch_relative(
                    offset,
                    addr,
                    label_prefix,
                    addr_to_offset_fn,
                    offset_to_addr_fn,
                    &|rel, label| Instruction::BCS_REL(rel, label),
                ),

                // LDA IND Y
                0xb1 => self.code.replace_with_instr(offset, 1, |args| {
                    Result::Ok(Instruction::LDA_IND_Y(args[0].to_u8()?))
                }),

                // LDA abs,x
                0xbd => self.code.replace_with_instr(offset, 2, |args| {
                    Result::Ok(Instruction::LDA_ABS_X(to_u16(&args[0], &args[1])?))
                }),

                // CPY IMM
                0xc0 => self.code.replace_with_instr(offset, 1, |args| {
                    Result::Ok(Instruction::CPY_IMM(args[0].to_u8()?))
                }),

                // INY
                0xc8 => self
                    .code
                    .replace_with_instr(offset, 0, |_args| Result::Ok(Instruction::INY)),

                // DEX
                0xca => self
                    .code
                    .replace_with_instr(offset, 0, |_args| Result::Ok(Instruction::DEX)),

                // BNE REL
                0xd0 => self.branch_relative(
                    offset,
                    addr,
                    label_prefix,
                    addr_to_offset_fn,
                    offset_to_addr_fn,
                    &|rel, label| Instruction::BNE_REL(rel, label),
                ),

                // Other
                _ => {
                    // TODO i => return Result::Err(DisassembleError::UnhandledInstruction(i))
                    println!("unhandled: 0x{:02x}", op);
                    break;
                }
            };

            match result {
                Result::Ok(size) => {
                    if size == 0 {
                        break;
                    }
                    offset += size;
                    addr += size as u16;
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

    fn branch_relative<
        F1: Fn(u16) -> usize,
        F2: Fn(usize) -> u16,
        F3: Fn(i8, String) -> Instruction,
    >(
        &mut self,
        offset: usize,
        addr: u16,
        label_prefix: &str,
        addr_to_offset_fn: &F1,
        offset_to_addr_fn: &F2,
        to_instruction_fn: &F3,
    ) -> Result<usize, DisassembleError> {
        let rel = self.code.get_i8(offset + 1)?;
        let new_addr = addr.wrapping_add(rel as u16) + 2;
        let label = format!("{}_{:04x}", label_prefix, new_addr);
        let result = self.code.replace_with_instr(offset, 1, |_args| {
            Result::Ok(to_instruction_fn(rel, label.clone()))
        });

        // disassemble jump address
        self.disassemble(
            new_addr,
            format!("{:04x}", new_addr).as_str(),
            label_prefix,
            addr_to_offset_fn,
            offset_to_addr_fn,
        )?;

        return result;
    }
}

fn to_u16(arg0: &AsmCode, arg1: &AsmCode) -> Result<u16, DisassembleError> {
    let l = arg0.to_u8()? as u16;
    let h = arg1.to_u8()? as u16;
    return Result::Ok((h << 8) | l);
}
