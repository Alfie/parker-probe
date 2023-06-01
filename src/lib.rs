mod utils;
//TODO: fork rbpf and write rust implementations of necessary libc functions
pub mod rbpf;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use elf::section::SectionHeader;
use elf::ElfBytes;
use elf::endian::LittleEndian;
use wasm_bindgen::prelude::*;
// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

extern crate web_sys;

// A macro to provide `println!(..)`-style syntax for `console.log` logging.
macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}

extern crate elf;

#[derive(Serialize, Deserialize, Debug, Clone)]
enum Element {
    Opcode {
        text: String,
        op: String,
        operand1: String,
        operand2: String,
        comment: String,
    },
    Text {
        text: String,
    },
    Include {
        text: String,
        target: String,
    },
    Macro {
        label: String,
        texts: Vec<String>,
        text: String,
    },
}

#[derive(Serialize, Deserialize)]
pub struct Parser {
    //text: Vec<String>,
    line: usize,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Node {
    pub global: String,
    pub text: String,
    elements: Vec<Element>,
    pub next: String,
    pub next_cond: String,
    pub calls: Vec<String>,
}

pub type Nodes = HashMap<String, Node>;

impl Parser {
    pub fn new() -> Self{


        return Self {
            //text: text,
            line: 0,
        };
    }

}

//TODO: maybe add section header data to the struct for easy access
#[wasm_bindgen]
pub struct ElfFile {
    raw_data: Vec<u8>,
}

impl ElfFile {

}

#[wasm_bindgen]
impl ElfFile {
    pub fn new()  -> ElfFile {
        utils::set_panic_hook();
        let init = Vec::new();
        ElfFile { raw_data: init }
    }

    pub fn load(&mut self, data: Vec<u8>){
        self.raw_data = data;
    }

    //TODO: print w/o section header
    pub fn list_sections(&mut self){
        let data =self.raw_data.as_slice();

        //TODO: handle error on the front end
        let file =  match ElfBytes::<LittleEndian>::minimal_parse(&data[45..]){
            Ok(f) => f,
            Err(e) =>  panic!("Problem opening the file: {:?}", e),
        };
        
        let (shdrs_opt, strtab_opt) = file
            .section_headers_with_strtab().unwrap();
        let (shdrs, strtab) = (
            shdrs_opt.expect("Should have shdrs"),
            strtab_opt.expect("Should have strtab")
        );

        // Parse the shdrs and collect them into a map keyed on their zero-copied name
        let with_names: HashMap<&str, SectionHeader> = shdrs
            .iter()
            .map(|shdr| {
                (
                    strtab.get(shdr.sh_name as usize).expect("Failed to get section name"),
                    shdr,
                )
            })
            .collect();

        log!("{:?}", with_names);
        
    }

    //TODO: take a section header as an argument
    pub fn disassemble(&mut self){
        let data =self.raw_data.as_slice();

        //TODO: handle error on the front end
        let file =  match ElfBytes::<LittleEndian>::minimal_parse(&data[45..]){
            Ok(f) => f,
            Err(e) =>  panic!("Problem opening the file: {:?}", e),
        };

        //TODO: turn this into a function for readabilitu and reuasbility.
        let (shdrs_opt, strtab_opt) = file
            .section_headers_with_strtab().unwrap();
        let (shdrs, strtab) = (
            shdrs_opt.expect("Should have shdrs"),
            strtab_opt.expect("Should have strtab")
        );

        //TODO: get section data from hashmap and place it in the function
        // Parse the shdrs and collect them into a map keyed on their zero-copied name
        let with_names: HashMap<&str, SectionHeader> = shdrs
            .iter()
            .map(|shdr| {
                (
                    strtab.get(shdr.sh_name as usize).expect("Failed to get section name"),
                    shdr,
                )
            })
            .collect();

        let scn = with_names.get(".text").unwrap();
        
        let (text_scn, _) = match file.section_data(scn){
            Ok(f) => f,
            Err(e) => panic!("Problem reading section: {:?}",e),
        };

        let mut idx = 0;
        for insn in rbpf::disassembler::to_insn_vec(text_scn){
            log!("{}", insn.desc.replace(" ", "\t"));

            if is_jmp(&insn) {
                log!("\t(jump to {})", idx + insn.off as isize);
            }

            if is_wide_op(&insn) {
                idx += 1;
            }

            println!();
            idx += 1;
        }
    }
}

//TODO: This likely needs to be wrapped in an Impl block
// Check if the instruction is a jump
// (but not a call, tailcall or exit)
//
// Jumps are relative from the current instruction pointer
fn is_jmp(insn: &rbpf::disassembler::HLInsn) -> bool {
    (insn.opc & 0x07) == rbpf::ebpf::BPF_JMP &&
        (insn.opc != rbpf::ebpf::CALL) &&
        (insn.opc != rbpf::ebpf::TAIL_CALL) &&
        (insn.opc != rbpf::ebpf::EXIT)
}

// Check if the instruction is spread across two instructions
//
// Currently only a wide load uses a second instruction for the u64 field.
fn is_wide_op(insn: &rbpf::disassembler::HLInsn) -> bool {
    insn.opc == rbpf::ebpf::LD_DW_IMM
}