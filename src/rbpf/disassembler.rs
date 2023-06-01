//A copy of the https://github.com/qmonnet/rbpf/src/disassembler.rs to avoid using parts
//of the crate which reference libc
use crate::rbpf::ebpf;

#[inline]
fn alu_imm_str(name: &str, insn: &ebpf::Insn) -> String {
    format!("{name} r{}, {:#x}", insn.dst, insn.imm)
}

#[inline]
fn alu_reg_str(name: &str, insn: &ebpf::Insn) -> String {
    format!("{name} r{}, r{}", insn.dst, insn.src)
}

#[inline]
fn byteswap_str(name: &str, insn: &ebpf::Insn) -> String {
    match insn.imm {
        16 | 32 | 64 => {},
        _ => println!("[Disassembler] Warning: Invalid offset value for {name} insn")
    }
    format!("{name}{} r{}", insn.imm, insn.dst)
}

#[inline]
fn ld_st_imm_str(name: &str, insn: &ebpf::Insn) -> String {
    if insn.off >= 0 {
        format!("{name} [r{}+{:#x}], {:#x}", insn.dst, insn.off, insn.imm)
    } else {
        format!("{name} [r{}-{:#x}], {:#x}", insn.dst, -insn.off, insn.imm)
    }
}

#[inline]
fn ld_reg_str(name: &str, insn: &ebpf::Insn) -> String {
    if insn.off >= 0 {
        format!("{name} r{}, [r{}+{:#x}]", insn.dst, insn.src, insn.off)
    } else {
        format!("{name} r{}, [r{}-{:#x}]", insn.dst, insn.src, -insn.off)
    }
}

#[inline]
fn st_reg_str(name: &str, insn: &ebpf::Insn) -> String {
    if insn.off >= 0 {
        format!("{name} [r{}+{:#x}], r{}", insn.dst, insn.off, insn.src)
    } else {
        format!("{name} [r{}-{:#x}], r{}", insn.dst, -insn.off, insn.src)
    }
}

#[inline]
fn ldabs_str(name: &str, insn: &ebpf::Insn) -> String {
    format!("{name} {:#x}", insn.imm)
}

#[inline]
fn ldind_str(name: &str, insn: &ebpf::Insn) -> String {
    format!("{name} r{}, {:#x}", insn.src, insn.imm)
}

#[inline]
fn jmp_imm_str(name: &str, insn: &ebpf::Insn) -> String {
    if insn.off >= 0 {
        format!("{name} r{}, {:#x}, +{:#x}", insn.dst, insn.imm, insn.off)
    } else {
        format!("{name} r{}, {:#x}, -{:#x}", insn.dst, insn.imm, -insn.off)
    }
}

#[inline]
fn jmp_reg_str(name: &str, insn: &ebpf::Insn) -> String {
    if insn.off >= 0 {
        format!("{name} r{}, r{}, +{:#x}", insn.dst, insn.src, insn.off)
    } else {
        format!("{name} r{}, r{}, -{:#x}", insn.dst, insn.src, -insn.off)
    }
}

/// High-level representation of an eBPF instruction.
///
/// In addition to standard operation code and various operand, this struct has the following
/// properties:
///
/// * It stores a name, corresponding to a mnemonic for the operation code.
/// * It also stores a description, which is a mnemonic for the full instruction, using the actual
///   values of the relevant operands, and that can be used for disassembling the eBPF program for
///   example.
/// * Immediate values are stored in an `i64` instead of a traditional i32, in order to merge the
///   two parts of (otherwise double-length) `LD_DW_IMM` instructions.
///
/// See <https://www.kernel.org/doc/Documentation/networking/filter.txt> for the Linux kernel
/// documentation about eBPF, or <https://github.com/iovisor/bpf-docs/blob/master/eBPF.md> for a
/// more concise version.
#[derive(Debug, PartialEq, Eq)]
pub struct HLInsn {
    /// Operation code.
    pub opc:  u8,
    /// Name (mnemonic). This name is not canon.
    pub name: String,
    /// Description of the instruction. This is not canon.
    pub desc: String,
    /// Destination register operand.
    pub dst:  u8,
    /// Source register operand.
    pub src:  u8,
    /// Offset operand.
    pub off:  i16,
    /// Immediate value operand. For `LD_DW_IMM` instructions, contains the whole value merged from
    /// the two 8-bytes parts of the instruction.
    pub imm:  i64,
}

pub fn to_insn_vec(prog: &[u8]) -> Vec<HLInsn> {
    if prog.len() % ebpf::INSN_SIZE != 0 {
        panic!("[Disassembler] Error: eBPF program length must be a multiple of {:?} octets",
               ebpf::INSN_SIZE);
    }
    if prog.is_empty() {
        return vec![];
    }

    let mut res = vec![];
    let mut insn_ptr:usize = 0;

    while insn_ptr * ebpf::INSN_SIZE < prog.len() {
        let insn = ebpf::get_insn(prog, insn_ptr);

        let name;
        let desc;
        let mut imm = insn.imm as i64;
        match insn.opc {

            // BPF_LD class
            ebpf::LD_ABS_B   => { name = "ldabsb";  desc = ldabs_str(name, &insn); },
            ebpf::LD_ABS_H   => { name = "ldabsh";  desc = ldabs_str(name, &insn); },
            ebpf::LD_ABS_W   => { name = "ldabsw";  desc = ldabs_str(name, &insn); },
            ebpf::LD_ABS_DW  => { name = "ldabsdw"; desc = ldabs_str(name, &insn); },
            ebpf::LD_IND_B   => { name = "ldindb";  desc = ldind_str(name, &insn); },
            ebpf::LD_IND_H   => { name = "ldindh";  desc = ldind_str(name, &insn); },
            ebpf::LD_IND_W   => { name = "ldindw";  desc = ldind_str(name, &insn); },
            ebpf::LD_IND_DW  => { name = "ldinddw"; desc = ldind_str(name, &insn); },

            ebpf::LD_DW_IMM  => {
                insn_ptr += 1;
                let next_insn = ebpf::get_insn(prog, insn_ptr);
                imm = ((insn.imm as u32) as u64 + ((next_insn.imm as u64) << 32)) as i64;
                name = "lddw"; desc = format!("{name} r{:}, {imm:#x}", insn.dst);
            },

            // BPF_LDX class
            ebpf::LD_B_REG   => { name = "ldxb";  desc = ld_reg_str(name, &insn); },
            ebpf::LD_H_REG   => { name = "ldxh";  desc = ld_reg_str(name, &insn); },
            ebpf::LD_W_REG   => { name = "ldxw";  desc = ld_reg_str(name, &insn); },
            ebpf::LD_DW_REG  => { name = "ldxdw"; desc = ld_reg_str(name, &insn); },

            // BPF_ST class
            ebpf::ST_B_IMM   => { name = "stb";  desc = ld_st_imm_str(name, &insn); },
            ebpf::ST_H_IMM   => { name = "sth";  desc = ld_st_imm_str(name, &insn); },
            ebpf::ST_W_IMM   => { name = "stw";  desc = ld_st_imm_str(name, &insn); },
            ebpf::ST_DW_IMM  => { name = "stdw"; desc = ld_st_imm_str(name, &insn); },

            // BPF_STX class
            ebpf::ST_B_REG   => { name = "stxb";      desc = st_reg_str(name, &insn); },
            ebpf::ST_H_REG   => { name = "stxh";      desc = st_reg_str(name, &insn); },
            ebpf::ST_W_REG   => { name = "stxw";      desc = st_reg_str(name, &insn); },
            ebpf::ST_DW_REG  => { name = "stxdw";     desc = st_reg_str(name, &insn); },
            ebpf::ST_W_XADD  => { name = "stxxaddw";  desc = st_reg_str(name, &insn); },
            ebpf::ST_DW_XADD => { name = "stxxadddw"; desc = st_reg_str(name, &insn); },

            // BPF_ALU class
            ebpf::ADD32_IMM  => { name = "add32";  desc = alu_imm_str(name, &insn);  },
            ebpf::ADD32_REG  => { name = "add32";  desc = alu_reg_str(name, &insn);  },
            ebpf::SUB32_IMM  => { name = "sub32";  desc = alu_imm_str(name, &insn);  },
            ebpf::SUB32_REG  => { name = "sub32";  desc = alu_reg_str(name, &insn);  },
            ebpf::MUL32_IMM  => { name = "mul32";  desc = alu_imm_str(name, &insn);  },
            ebpf::MUL32_REG  => { name = "mul32";  desc = alu_reg_str(name, &insn);  },
            ebpf::DIV32_IMM  => { name = "div32";  desc = alu_imm_str(name, &insn);  },
            ebpf::DIV32_REG  => { name = "div32";  desc = alu_reg_str(name, &insn);  },
            ebpf::OR32_IMM   => { name = "or32";   desc = alu_imm_str(name, &insn);  },
            ebpf::OR32_REG   => { name = "or32";   desc = alu_reg_str(name, &insn);  },
            ebpf::AND32_IMM  => { name = "and32";  desc = alu_imm_str(name, &insn);  },
            ebpf::AND32_REG  => { name = "and32";  desc = alu_reg_str(name, &insn);  },
            ebpf::LSH32_IMM  => { name = "lsh32";  desc = alu_imm_str(name, &insn);  },
            ebpf::LSH32_REG  => { name = "lsh32";  desc = alu_reg_str(name, &insn);  },
            ebpf::RSH32_IMM  => { name = "rsh32";  desc = alu_imm_str(name, &insn);  },
            ebpf::RSH32_REG  => { name = "rsh32";  desc = alu_reg_str(name, &insn);  },
            ebpf::NEG32      => { name = "neg32";  desc = format!("{name} r{:}", insn.dst); },
            ebpf::MOD32_IMM  => { name = "mod32";  desc = alu_imm_str(name, &insn);  },
            ebpf::MOD32_REG  => { name = "mod32";  desc = alu_reg_str(name, &insn);  },
            ebpf::XOR32_IMM  => { name = "xor32";  desc = alu_imm_str(name, &insn);  },
            ebpf::XOR32_REG  => { name = "xor32";  desc = alu_reg_str(name, &insn);  },
            ebpf::MOV32_IMM  => { name = "mov32";  desc = alu_imm_str(name, &insn);  },
            ebpf::MOV32_REG  => { name = "mov32";  desc = alu_reg_str(name, &insn);  },
            ebpf::ARSH32_IMM => { name = "arsh32"; desc = alu_imm_str(name, &insn);  },
            ebpf::ARSH32_REG => { name = "arsh32"; desc = alu_reg_str(name, &insn);  },
            ebpf::LE         => { name = "le";     desc = byteswap_str(name, &insn); },
            ebpf::BE         => { name = "be";     desc = byteswap_str(name, &insn); },

            // BPF_ALU64 class
            ebpf::ADD64_IMM  => { name = "add64";  desc = alu_imm_str(name, &insn); },
            ebpf::ADD64_REG  => { name = "add64";  desc = alu_reg_str(name, &insn); },
            ebpf::SUB64_IMM  => { name = "sub64";  desc = alu_imm_str(name, &insn); },
            ebpf::SUB64_REG  => { name = "sub64";  desc = alu_reg_str(name, &insn); },
            ebpf::MUL64_IMM  => { name = "mul64";  desc = alu_imm_str(name, &insn); },
            ebpf::MUL64_REG  => { name = "mul64";  desc = alu_reg_str(name, &insn); },
            ebpf::DIV64_IMM  => { name = "div64";  desc = alu_imm_str(name, &insn); },
            ebpf::DIV64_REG  => { name = "div64";  desc = alu_reg_str(name, &insn); },
            ebpf::OR64_IMM   => { name = "or64";   desc = alu_imm_str(name, &insn); },
            ebpf::OR64_REG   => { name = "or64";   desc = alu_reg_str(name, &insn); },
            ebpf::AND64_IMM  => { name = "and64";  desc = alu_imm_str(name, &insn); },
            ebpf::AND64_REG  => { name = "and64";  desc = alu_reg_str(name, &insn); },
            ebpf::LSH64_IMM  => { name = "lsh64";  desc = alu_imm_str(name, &insn); },
            ebpf::LSH64_REG  => { name = "lsh64";  desc = alu_reg_str(name, &insn); },
            ebpf::RSH64_IMM  => { name = "rsh64";  desc = alu_imm_str(name, &insn); },
            ebpf::RSH64_REG  => { name = "rsh64";  desc = alu_reg_str(name, &insn); },
            ebpf::NEG64      => { name = "neg64";  desc = format!("{name} r{:}", insn.dst); },
            ebpf::MOD64_IMM  => { name = "mod64";  desc = alu_imm_str(name, &insn); },
            ebpf::MOD64_REG  => { name = "mod64";  desc = alu_reg_str(name, &insn); },
            ebpf::XOR64_IMM  => { name = "xor64";  desc = alu_imm_str(name, &insn); },
            ebpf::XOR64_REG  => { name = "xor64";  desc = alu_reg_str(name, &insn); },
            ebpf::MOV64_IMM  => { name = "mov64";  desc = alu_imm_str(name, &insn); },
            ebpf::MOV64_REG  => { name = "mov64";  desc = alu_reg_str(name, &insn); },
            ebpf::ARSH64_IMM => { name = "arsh64"; desc = alu_imm_str(name, &insn); },
            ebpf::ARSH64_REG => { name = "arsh64"; desc = alu_reg_str(name, &insn); },

            // BPF_JMP class
            ebpf::JA         => { name = "ja";   desc = if insn.off >= 0 { format!("{name} +{:#x}", insn.off) } else { format!("{name} -{:#x}", -insn.off) } },
            ebpf::JEQ_IMM    => { name = "jeq";  desc = jmp_imm_str(name, &insn); },
            ebpf::JEQ_REG    => { name = "jeq";  desc = jmp_reg_str(name, &insn); },
            ebpf::JGT_IMM    => { name = "jgt";  desc = jmp_imm_str(name, &insn); },
            ebpf::JGT_REG    => { name = "jgt";  desc = jmp_reg_str(name, &insn); },
            ebpf::JGE_IMM    => { name = "jge";  desc = jmp_imm_str(name, &insn); },
            ebpf::JGE_REG    => { name = "jge";  desc = jmp_reg_str(name, &insn); },
            ebpf::JLT_IMM    => { name = "jlt";  desc = jmp_imm_str(name, &insn); },
            ebpf::JLT_REG    => { name = "jlt";  desc = jmp_reg_str(name, &insn); },
            ebpf::JLE_IMM    => { name = "jle";  desc = jmp_imm_str(name, &insn); },
            ebpf::JLE_REG    => { name = "jle";  desc = jmp_reg_str(name, &insn); },
            ebpf::JSET_IMM   => { name = "jset"; desc = jmp_imm_str(name, &insn); },
            ebpf::JSET_REG   => { name = "jset"; desc = jmp_reg_str(name, &insn); },
            ebpf::JNE_IMM    => { name = "jne";  desc = jmp_imm_str(name, &insn); },
            ebpf::JNE_REG    => { name = "jne";  desc = jmp_reg_str(name, &insn); },
            ebpf::JSGT_IMM   => { name = "jsgt"; desc = jmp_imm_str(name, &insn); },
            ebpf::JSGT_REG   => { name = "jsgt"; desc = jmp_reg_str(name, &insn); },
            ebpf::JSGE_IMM   => { name = "jsge"; desc = jmp_imm_str(name, &insn); },
            ebpf::JSGE_REG   => { name = "jsge"; desc = jmp_reg_str(name, &insn); },
            ebpf::JSLT_IMM   => { name = "jslt"; desc = jmp_imm_str(name, &insn); },
            ebpf::JSLT_REG   => { name = "jslt"; desc = jmp_reg_str(name, &insn); },
            ebpf::JSLE_IMM   => { name = "jsle"; desc = jmp_imm_str(name, &insn); },
            ebpf::JSLE_REG   => { name = "jsle"; desc = jmp_reg_str(name, &insn); },
            ebpf::CALL       => { name = "call"; desc = format!("{name} {:#x}", insn.imm); },
            ebpf::TAIL_CALL  => { name = "tail_call"; desc = name.to_string(); },
            ebpf::EXIT       => { name = "exit";      desc = name.to_string(); },

            // BPF_JMP32 class
            ebpf::JEQ_IMM32  => { name = "jeq32";  desc = jmp_imm_str(name, &insn); },
            ebpf::JEQ_REG32  => { name = "jeq32";  desc = jmp_reg_str(name, &insn); },
            ebpf::JGT_IMM32  => { name = "jgt32";  desc = jmp_imm_str(name, &insn); },
            ebpf::JGT_REG32  => { name = "jgt32";  desc = jmp_reg_str(name, &insn); },
            ebpf::JGE_IMM32  => { name = "jge32";  desc = jmp_imm_str(name, &insn); },
            ebpf::JGE_REG32  => { name = "jge32";  desc = jmp_reg_str(name, &insn); },
            ebpf::JLT_IMM32  => { name = "jlt32";  desc = jmp_imm_str(name, &insn); },
            ebpf::JLT_REG32  => { name = "jlt32";  desc = jmp_reg_str(name, &insn); },
            ebpf::JLE_IMM32  => { name = "jle32";  desc = jmp_imm_str(name, &insn); },
            ebpf::JLE_REG32  => { name = "jle32";  desc = jmp_reg_str(name, &insn); },
            ebpf::JSET_IMM32 => { name = "jset32"; desc = jmp_imm_str(name, &insn); },
            ebpf::JSET_REG32 => { name = "jset32"; desc = jmp_reg_str(name, &insn); },
            ebpf::JNE_IMM32  => { name = "jne32";  desc = jmp_imm_str(name, &insn); },
            ebpf::JNE_REG32  => { name = "jne32";  desc = jmp_reg_str(name, &insn); },
            ebpf::JSGT_IMM32 => { name = "jsgt32"; desc = jmp_imm_str(name, &insn); },
            ebpf::JSGT_REG32 => { name = "jsgt32"; desc = jmp_reg_str(name, &insn); },
            ebpf::JSGE_IMM32 => { name = "jsge32"; desc = jmp_imm_str(name, &insn); },
            ebpf::JSGE_REG32 => { name = "jsge32"; desc = jmp_reg_str(name, &insn); },
            ebpf::JSLT_IMM32 => { name = "jslt32"; desc = jmp_imm_str(name, &insn); },
            ebpf::JSLT_REG32 => { name = "jslt32"; desc = jmp_reg_str(name, &insn); },
            ebpf::JSLE_IMM32 => { name = "jsle32"; desc = jmp_imm_str(name, &insn); },
            ebpf::JSLE_REG32 => { name = "jsle32"; desc = jmp_reg_str(name, &insn); },

            _                => {
                panic!("[Disassembler] Error: unknown eBPF opcode {:#2x} (insn #{:?})",
                       insn.opc, insn_ptr);
            },
        };

        let hl_insn = HLInsn {
            opc:  insn.opc,
            name: name.to_string(),
            desc: desc,
            dst:  insn.dst,
            src:  insn.src,
            off:  insn.off,
            imm:  imm,
        };

        res.push(hl_insn);

        insn_ptr += 1;
    };
    res
}