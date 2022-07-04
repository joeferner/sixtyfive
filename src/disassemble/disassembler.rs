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
            let mut set_addr: Option<u16> = Option::None;
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

                // BPL
                0x10 => self.branch_relative(
                    offset,
                    addr,
                    label_prefix,
                    addr_to_offset_fn,
                    offset_to_addr_fn,
                    &|rel, label| Instruction::BPL_REL(rel, label),
                ),

                // CLC
                0x18 => self
                    .code
                    .replace_with_instr(offset, 0, |_args| Result::Ok(Instruction::CLC)),

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

                // BIT ZP
                0x24 => self.code.replace_with_instr(offset, 1, |args| {
                    Result::Ok(Instruction::BIT_ZP(args[0].to_u8()?))
                }),

                // AND IMM
                0x29 => self.code.replace_with_instr(offset, 1, |args| {
                    Result::Ok(Instruction::AND_IMM(args[0].to_u8()?))
                }),

                // ROL
                0x2a => self
                    .code
                    .replace_with_instr(offset, 0, |_args| Result::Ok(Instruction::ROL)),

                // AND ZP,x
                0x35 => self.code.replace_with_instr(offset, 1, |args| {
                    Result::Ok(Instruction::AND_ZP_X(args[0].to_u8()?))
                }),

                // SEC
                0x38 => self
                    .code
                    .replace_with_instr(offset, 0, |_args| Result::Ok(Instruction::SEC)),

                // PHA
                0x48 => self
                    .code
                    .replace_with_instr(offset, 0, |_args| Result::Ok(Instruction::PHA)),

                // EOR IMM
                0x49 => self.code.replace_with_instr(offset, 1, |args| {
                    Result::Ok(Instruction::EOR_IMM(args[0].to_u8()?))
                }),

                // LSR
                0x4a => self
                    .code
                    .replace_with_instr(offset, 0, |_args| Result::Ok(Instruction::LSR)),

                // JMP ABS
                0x4c => {
                    let l = self.code.get_u8(offset + 1)? as u16;
                    let h = self.code.get_u8(offset + 2)? as u16;
                    let jmp_addr = (h << 8) | l;
                    let label = format!("{}_{:04x}", label_prefix, jmp_addr);
                    self.code.replace_with_instr(offset, 2, |_args| {
                        Result::Ok(Instruction::JMP_ABS(jmp_addr, label.clone()))
                    })?;

                    set_addr = Option::Some(jmp_addr);
                    Result::Ok(0)
                }

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

                // ADC IMM
                0x69 => self.code.replace_with_instr(offset, 1, |args| {
                    Result::Ok(Instruction::ADC_IMM(args[0].to_u8()?))
                }),

                // ROR
                0x6a => self
                    .code
                    .replace_with_instr(offset, 0, |_args| Result::Ok(Instruction::ROR)),

                // ADC ABS
                0x6d => self.code.replace_with_instr(offset, 2, |args| {
                    Result::Ok(Instruction::ADC_ABS(to_u16(&args[0], &args[1])?))
                }),

                // STY ZP
                0x84 => self.code.replace_with_instr(offset, 1, |args| {
                    Result::Ok(Instruction::STY_ZP(args[0].to_u8()?))
                }),

                // STA ZP
                0x85 => self.code.replace_with_instr(offset, 1, |args| {
                    Result::Ok(Instruction::STA_ZP(args[0].to_u8()?))
                }),

                // STX ZP
                0x86 => self.code.replace_with_instr(offset, 1, |args| {
                    Result::Ok(Instruction::STX_ZP(args[0].to_u8()?))
                }),

                // DEY
                0x88 => self
                    .code
                    .replace_with_instr(offset, 0, |_args| Result::Ok(Instruction::DEY)),

                // TXA
                0x8a => self
                    .code
                    .replace_with_instr(offset, 0, |_args| Result::Ok(Instruction::TXA)),

                // STY ABS
                0x8c => self.code.replace_with_instr(offset, 2, |args| {
                    Result::Ok(Instruction::STY_ABS(to_u16(&args[0], &args[1])?))
                }),

                // STA ABS
                0x8d => self.code.replace_with_instr(offset, 2, |args| {
                    Result::Ok(Instruction::STA_ABS(to_u16(&args[0], &args[1])?))
                }),

                // STX ABS
                0x8e => self.code.replace_with_instr(offset, 2, |args| {
                    Result::Ok(Instruction::STX_ABS(to_u16(&args[0], &args[1])?))
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

                // STA ZP,x
                0x95 => self.code.replace_with_instr(offset, 1, |args| {
                    Result::Ok(Instruction::STA_ZP_X(args[0].to_u8()?))
                }),

                // TYA
                0x98 => self
                    .code
                    .replace_with_instr(offset, 0, |_args| Result::Ok(Instruction::TYA)),

                // STA ABS,x
                0x9d => self.code.replace_with_instr(offset, 2, |args| {
                    Result::Ok(Instruction::STA_ABS_X(to_u16(&args[0], &args[1])?))
                }),

                // LDY IMM
                0xa0 => self.code.replace_with_instr(offset, 1, |args| {
                    Result::Ok(Instruction::LDY_IMM(args[0].to_u8()?))
                }),

                // LDX IMM
                0xa2 => self.code.replace_with_instr(offset, 1, |args| {
                    Result::Ok(Instruction::LDX_IMM(args[0].to_u8()?))
                }),

                // LDY ZP
                0xa4 => self.code.replace_with_instr(offset, 1, |args| {
                    Result::Ok(Instruction::LDY_ZP(args[0].to_u8()?))
                }),

                // LDA ZP
                0xa5 => self.code.replace_with_instr(offset, 1, |args| {
                    Result::Ok(Instruction::LDA_ZP(args[0].to_u8()?))
                }),

                // LDX ZP
                0xa6 => self.code.replace_with_instr(offset, 1, |args| {
                    Result::Ok(Instruction::LDX_ZP(args[0].to_u8()?))
                }),

                // LDA IMM
                0xa9 => self.code.replace_with_instr(offset, 1, |args| {
                    Result::Ok(Instruction::LDA_IMM(args[0].to_u8()?))
                }),

                // TAX
                0xaa => self
                    .code
                    .replace_with_instr(offset, 0, |_args| Result::Ok(Instruction::TAX)),

                // TAY
                0xa8 => self
                    .code
                    .replace_with_instr(offset, 0, |_args| Result::Ok(Instruction::TAY)),

                // LDY ABS
                0xac => self.code.replace_with_instr(offset, 2, |args| {
                    Result::Ok(Instruction::LDY_ABS(to_u16(&args[0], &args[1])?))
                }),

                // LDA ABS
                0xad => self.code.replace_with_instr(offset, 2, |args| {
                    Result::Ok(Instruction::LDA_ABS(to_u16(&args[0], &args[1])?))
                }),

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

                // LDY ZP,x
                0xb4 => self.code.replace_with_instr(offset, 1, |args| {
                    Result::Ok(Instruction::LDY_ZP_X(args[0].to_u8()?))
                }),

                // LDA ABS,y
                0xb9 => self.code.replace_with_instr(offset, 2, |args| {
                    Result::Ok(Instruction::LDA_ABS_Y(to_u16(&args[0], &args[1])?))
                }),

                // LDY ABS,x
                0xbc => self.code.replace_with_instr(offset, 2, |args| {
                    Result::Ok(Instruction::LDY_ABS_X(to_u16(&args[0], &args[1])?))
                }),

                // LDA abs,x
                0xbd => self.code.replace_with_instr(offset, 2, |args| {
                    Result::Ok(Instruction::LDA_ABS_X(to_u16(&args[0], &args[1])?))
                }),

                // CPY IMM
                0xc0 => self.code.replace_with_instr(offset, 1, |args| {
                    Result::Ok(Instruction::CPY_IMM(args[0].to_u8()?))
                }),

                // DEC ZP
                0xc6 => self.code.replace_with_instr(offset, 1, |args| {
                    Result::Ok(Instruction::DEC_ZP(args[0].to_u8()?))
                }),

                // INY
                0xc8 => self
                    .code
                    .replace_with_instr(offset, 0, |_args| Result::Ok(Instruction::INY)),

                // CMP IMM
                0xc9 => self.code.replace_with_instr(offset, 1, |args| {
                    Result::Ok(Instruction::CMP_IMM(args[0].to_u8()?))
                }),

                // DEX
                0xca => self
                    .code
                    .replace_with_instr(offset, 0, |_args| Result::Ok(Instruction::DEX)),

                // DEC ABS
                0xce => self.code.replace_with_instr(offset, 2, |args| {
                    Result::Ok(Instruction::DEC_ABS(to_u16(&args[0], &args[1])?))
                }),

                // BNE REL
                0xd0 => self.branch_relative(
                    offset,
                    addr,
                    label_prefix,
                    addr_to_offset_fn,
                    offset_to_addr_fn,
                    &|rel, label| Instruction::BNE_REL(rel, label),
                ),

                // INC ZP
                0xe6 => self.code.replace_with_instr(offset, 1, |args| {
                    Result::Ok(Instruction::INC_ZP(args[0].to_u8()?))
                }),

                // INX
                0xe8 => self
                    .code
                    .replace_with_instr(offset, 0, |_args| Result::Ok(Instruction::INX)),

                // BEQ
                0xf0 => self.branch_relative(
                    offset,
                    addr,
                    label_prefix,
                    addr_to_offset_fn,
                    offset_to_addr_fn,
                    &|rel, label| Instruction::BEQ_REL(rel, label),
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
                        if let Option::Some(new_addr) = set_addr {
                            offset = addr_to_offset_fn(new_addr);
                            addr = new_addr;
                        } else {
                            break;
                        }
                    } else {
                        offset += size;
                        addr += size as u16;
                    }
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
