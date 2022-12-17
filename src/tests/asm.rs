use crate::asm::decode::{Opcode,decode_opcodes};
use crate::elf::decoder::ElfError;
use crate::asm::HalfWord;
use std::path::{Path, PathBuf};
use std::fs::File;
use std::io::Write;

fn load_instruction_opcodes(file: &Path)->Result<Vec<u8>,ElfError>{
   use crate::elf::decoder::{
      SectionHeader,
      get_header,
      get_all_section_headers,
      is_text_section_hdr,
      read_text_section
   };
   let (elf_header,mut reader) = get_header(file).unwrap();

   let section_headers = get_all_section_headers(&mut reader, &elf_header)?;
   println!("sect_hdrs {:?}",section_headers);
   assert!(!section_headers.is_empty());

   let text_sect_hdr: Vec<SectionHeader> = section_headers.into_iter()
      .filter(|hdr| is_text_section_hdr(&elf_header, hdr))
      .collect();

   println!("header {:?}",text_sect_hdr);
   assert_eq!(text_sect_hdr.len(),1);
   let sect_hdr = &text_sect_hdr[0];

   let text_section = read_text_section(&mut reader, &elf_header, sect_hdr)?;
   assert!(!text_section.is_empty());
   Ok(text_section)
}

fn write_asm(path: &Path, data: &[u8])->Result<File,std::io::Error>{
   println!("writing  asm to {:?}",path);
   let mut file = File::create(path)?;
   file.write_all(data)?;
   println!("written asm to {:?}",file);
   Ok(file)
}

fn asm_file_to_elf(path: &Path)->Result<PathBuf,std::io::Error>{
   use std::process::Command;
   let mut fname = String::new();
   fname.push_str(path.to_str().unwrap());
   fname = fname.replace(".s", "");
   fname.push_str(".elf");
   println!("writing to {:?}",fname);
   let ret = PathBuf::from(fname.clone());
   let cmd = Command::new("arm-none-eabi-as")
      .arg(path.to_str().unwrap())
      .arg("-o")
      .arg(fname)
      .output()
      .expect("could not link");

   println!("=======\n{:?}\n=======",std::str::from_utf8(&cmd.stderr[..]).unwrap());
   Ok(ret)
}

fn asm_file_to_elf_with_t2_arm_encoding(path: &Path)->Result<PathBuf,std::io::Error>{
   println!("forcing T2 encoding");
   use std::process::Command;
   let mut fname = String::new();
   fname.push_str(path.to_str().unwrap());
   fname = fname.replace(".s", "");
   fname.push_str(".elf");
   println!("writing to {:?}",fname);
   let ret = PathBuf::from(fname.clone());
   Command::new("arm-none-eabi-as")
      .arg(path.to_str().unwrap())
      .arg("-march=armv6t2")
      .arg("-o")
      .arg(fname)
      .status()
      .expect("could not link");

   Ok(ret)

}

fn decode_single_16b_instruction(path: &Path, asm: &[u8])->Result<Opcode,std::io::Error>{
   write_asm(path,asm)?;
   let elf = asm_file_to_elf(path)?;
   let opcodes = load_instruction_opcodes(&elf).unwrap();
   let first_instr: [u8;2] = [opcodes[0],opcodes[1]];

   let first: Opcode = (&first_instr).into();

   std::fs::remove_file(path)?;
   std::fs::remove_file(elf)?;
   println!("successfully assembled");
   Ok(first)
}

use std::io::Error as IOError;
fn assemble_and_decode<F: Fn(&Path)->Result<PathBuf,IOError>> (path: &Path, asm: &[u8],assembler: F)->Result<Opcode,IOError>{
   write_asm(path,asm)?;
   let elf = assembler(path)?;
   let opcodes = load_instruction_opcodes(&elf).unwrap();
   let first_instr: [u8;2] = [opcodes[0],opcodes[1]];

   let first: Opcode = (&first_instr).into();

   std::fs::remove_file(path)?;
   std::fs::remove_file(elf)?;
   Ok(first)
}

fn assemble_and_decode_32b<F: Fn(&Path)->Result<PathBuf,IOError>> (path: &Path, asm: &[u8],assembler: F)->Result<Opcode,IOError>{
   write_asm(path,asm)?;
   let elf = assembler(path)?;
   let opcodes = load_instruction_opcodes(&elf).unwrap();
   let first_instr: [u8;4] = [opcodes[0],opcodes[1],opcodes[2],opcodes[3]];

   let first: Opcode = (&first_instr).into();

   std::fs::remove_file(path)?;
   std::fs::remove_file(elf)?;
   Ok(first)
}
#[test]
pub fn should_recognise_instructions()-> Result<(),std::io::Error>{
   let path = Path::new("assembly_tests/adc.s");
   let instruction = decode_single_16b_instruction(
      path, 
      b".text\n\t.thumb\nADC r0, r1\n"
   ).unwrap();
   assert_eq!(Opcode::ADCS,instruction);

   Ok(())
}

#[test]
pub fn should_recognise_add_with_immediates()-> Result<(),std::io::Error>{
   let path = Path::new("assembly_tests/addi.s");
   write_asm(path, b".text\n\t.thumb\nADD r0,r1,#7\nADD r0,r1,#1\nADD r0, #255\nADD r0, #44")?;
   let elf = asm_file_to_elf(path)?;
   let opcodes = load_instruction_opcodes(&elf).unwrap();
   let mut first_inst: [u8;2] = [0;2];
   first_inst[0] = opcodes[0];
   first_inst[1] = opcodes[1];
   let mut second_instr: [u8;2] = [0;2];
   second_instr[0] = opcodes[2];
   second_instr[1] = opcodes[3];
   let mut third_instr: [u8;2] = [0;2];
   third_instr[0] = opcodes[4];
   third_instr[1] = opcodes[5];
   let mut fourth_instr: [u8;2] = [0;2];
   fourth_instr[0] = opcodes[6];
   fourth_instr[1] = opcodes[7];


   let instr: Opcode = (&first_inst).into();
   let secnd: Opcode = (&second_instr).into();
   let third: Opcode = (&third_instr).into();
   let fourth: Opcode = (&fourth_instr).into();

   println!("opcode raw bin{:?}",opcodes);
   std::fs::remove_file(path)?;
   std::fs::remove_file(elf)?;
   assert_eq!(Opcode::ADD_Imm3,instr);
   assert_eq!(Opcode::ADD_Imm3,secnd);
   assert_eq!(Opcode::ADD_Imm8,third);
   assert_eq!(Opcode::ADD_Imm8,fourth);

   Ok(())
}

#[test]
pub fn should_recognise_add_with_registers()-> Result<(),std::io::Error>{
   let path = Path::new("assembly_tests/addr.s");
   write_asm(path,b".text\n.thumb\nADD r3,r2,r1\nADD r0,r1\n")?;
   let elf = asm_file_to_elf_with_t2_arm_encoding(path)?;
   let opcodes = load_instruction_opcodes(&elf).unwrap();
   let mut first_instr: [u8;2] = [0;2];
   first_instr[0] = opcodes[0];
   first_instr[1] = opcodes[1];

   let first: Opcode = (&first_instr).into();

   println!("opcode raw bin{:?}",opcodes);
   std::fs::remove_file(path)?;
   std::fs::remove_file(elf)?;
   assert_eq!(Opcode::ADDS_REG,first);
   Ok(())
}

#[test]
pub fn should_recognise_add_sp_and_immediate() -> Result<(),std::io::Error>{
   let path = Path::new("assembly_tests/add_sp_imm.s");
   write_asm(path,b".text\n.thumb\nADD r7,SP,#64\nADD SP,SP,#128\nADD r0,SP,r0\nADD SP,r6\n")?;
   let elf = asm_file_to_elf(path)?;
   let opcodes = load_instruction_opcodes(&elf).unwrap();
   let mut first_instr: [u8;2] = [0;2];
   first_instr[0] = opcodes[0];
   first_instr[1] = opcodes[1];
   let mut sec_instr: [u8;2] = [0;2];
   sec_instr[0] = opcodes[2];
   sec_instr[1] = opcodes[3];
   let mut third_instr: [u8;2] = [0;2];
   third_instr[0] = opcodes[4];
   third_instr[1] = opcodes[5];
   let mut fourth_instr: [u8;2] = [0;2];
   fourth_instr[0] = opcodes[6];
   fourth_instr[1] = opcodes[7];

   let first: Opcode = (&first_instr).into();
   let second: Opcode = (&sec_instr).into();
   let third: Opcode = (&third_instr).into();
   let fourth: Opcode = (&fourth_instr).into();

   std::fs::remove_file(path)?;
   std::fs::remove_file(elf)?;
   assert_eq!(Opcode::ADD_REG_SP_IMM8,first);
   assert_eq!(Opcode::INCR_SP_BY_IMM7,second);
   assert_eq!(Opcode::INCR_REG_BY_SP,third);
   assert_eq!(Opcode::INCR_SP_BY_REG,fourth);

   Ok(())
}

#[test]
pub fn should_recognise_adr()->Result<(),std::io::Error>{
   let path = Path::new("assembly_tests/adr.s");

   let instruction = decode_single_16b_instruction(
      path,
      b".text\n.thumb\nADR r0,_some_lbl\nNOP\n_some_lbl:"
   ).unwrap();

   assert_eq!(Opcode::ADR,instruction);
   Ok(())
}

#[test]
pub fn should_recognise_adr_with_alternate_syntax()->Result<(),std::io::Error>{
   let path = Path::new("assembly_tests/adr_alt.s");
   
   let instruction = decode_single_16b_instruction(
      path,
      b".text\n.thumb\nADD r0,PC,#8\n"
   ).unwrap();

   assert_eq!(Opcode::ADR,instruction);
   Ok(())
}

#[test]
pub fn should_recognise_and_instruction()->Result<(),std::io::Error>{
   let path = Path::new("assembly_tests/and.s");
   write_asm(path,b".text\n.thumb\nAND r0,r1\nAND r2,r3,r2\n")?;
   let elf = asm_file_to_elf(path)?;
   let opcodes = load_instruction_opcodes(&elf).unwrap();
   let mut first_instr: [u8;2] = [0;2];
   first_instr[0] = opcodes[0];
   first_instr[1] = opcodes[1];

   let first: Opcode = (&first_instr).into();

   std::fs::remove_file(path)?;
   std::fs::remove_file(elf)?;
   assert_eq!(Opcode::ANDS,first);
   Ok(())
}

#[test]
pub fn should_recognise_asr()->Result<(),std::io::Error>{
   let path = Path::new("assembly_tests/asr.s");
   write_asm(path,b".text\n.thumb\nASR r0,r1,#24\nASR r0,r1\n")?;
   let elf = asm_file_to_elf(path)?;
   let opcodes = load_instruction_opcodes(&elf).unwrap();
   let mut first_instr: [u8;2] = [0;2];
   first_instr[0] = opcodes[0];
   first_instr[1] = opcodes[1];
   let mut second_instr: [u8;2] = [0;2];
   second_instr[0] = opcodes[2];
   second_instr[1] = opcodes[3];

   let first: Opcode = (&first_instr).into();
   let second: Opcode = (&second_instr).into();

   std::fs::remove_file(path)?;
   std::fs::remove_file(elf)?;
   assert_eq!(Opcode::ASRS_Imm5,first);
   assert_eq!(Opcode::ASRS_REG,second);
   Ok(())
}

#[test]
pub fn should_recognise_16bit_branch_instructions()->Result<(),std::io::Error>{
   let path = Path::new("assembly_tests/branch.s");
   let code = concat!(
      ".text\n",
      ".thumb\n",
      "b _l0\n",
      "_l0:\n",
      "beq _l1\n",
      "_l1:\n",
      "bne _l2\n",
      "_l2:\n",
      "bcs _l3\n",
      "_l3:\n",
      "bcc _l4\n",
      "_l4:\n",
      "bmi _l5\n",
      "_l5:\n",
      "bpl _l6\n",
      "_l6:\n",
      "bvs _l7\n",
      "_l7:\n",
      "bvc _l8\n",
      "_l8:\n",
      "bhi _l9\n",
      "_l9:\n",
      "bls _l10\n",
      "_l10:\n",
      "bge _l11\n",
      "_l11:\n",
      "blt _l12\n",
      "_l12:\n",
      "bgt _l13\n",
      "_l13:\n",
      "ble _l14\n",
      "_l14:\n"
   );

   write_asm(path, code.as_bytes())?;
   let elf = asm_file_to_elf(path)?;
   let opcodes = load_instruction_opcodes(&elf).unwrap();
   let decoded_opcodes = decode_16b_opcodes(&opcodes);

   std::fs::remove_file(path)?;
   std::fs::remove_file(elf)?;

   assert_eq!(
      decoded_opcodes,
      vec![
         Opcode::B_ALWAYS,
         Opcode::BEQ,
         Opcode::BNEQ,
         Opcode::B_CARRY_IS_SET,
         Opcode::B_CARRY_IS_CLEAR,
         Opcode::B_IF_NEGATIVE,
         Opcode::B_IF_POSITIVE,
         Opcode::B_IF_OVERFLOW,
         Opcode::B_IF_NO_OVERFLOW,
         Opcode::B_UNSIGNED_HIGHER,
         Opcode::B_UNSIGNED_LOWER_OR_SAME,
         Opcode::B_GTE,
         Opcode::B_LT,
         Opcode::B_GT,
         Opcode::B_LTE
      ]
   );
   Ok(())
}

fn decode_16b_opcodes(bytes: &[u8])->Vec<Opcode>{
   let mut opcodes = Vec::new();
   for i in bytes.chunks_exact(2){
      let halfword: HalfWord = i.try_into().expect("should be 16 bit aligned");
      opcodes.push(Opcode::from(&halfword));
   }
   opcodes
}

#[test]
fn should_recognise_bic()->Result<(),std::io::Error>{
   let path = Path::new("assembly_tests/bic.s");

   let instruction = decode_single_16b_instruction(
      path,
      b".text\n.thumb\nBIC r0,r1\n"
   ).unwrap();

   assert_eq!(Opcode::BIT_CLEAR_REGISTER,instruction);
   Ok(())
}

#[test]
fn should_recognise_breakpoint()->Result<(),std::io::Error>{
   let path = Path::new("assembly_tests/bkpt.s");
   
   let instruction = decode_single_16b_instruction(
      path,
      b".text\n.thumb\nBKPT #255\n"
   ).unwrap();

   assert_eq!(Opcode::BREAKPOINT,instruction);
   Ok(())
}

#[test]
fn should_recognise_bl()->Result<(),std::io::Error>{
   let path = Path::new("assembly_tests/bl.s");
   write_asm(path,b".text\n.thumb\nBL _some_where\n_some_where:\n")?;
   let elf = asm_file_to_elf(path)?;
   let opcodes = load_instruction_opcodes(&elf).unwrap();
   let first_instr: [u8;4] = [opcodes[0],opcodes[1],opcodes[2],opcodes[3]];

   let first: Opcode = (&first_instr).into();

   std::fs::remove_file(path)?;
   std::fs::remove_file(elf)?;
   assert_eq!(Opcode::BR_AND_LNK,first);
   Ok(())
}

#[test]
fn should_recognise_blx()->Result<(),std::io::Error>{
   let path = Path::new("assembly_tests/blx.s");

   let instruction = decode_single_16b_instruction(
      path,
      b".text\n.thumb\nBLX r0\n"
   ).unwrap();

   assert_eq!(Opcode::BR_LNK_EXCHANGE,instruction);
   Ok(())
}

#[test]
fn should_recognise_bx()->Result<(),std::io::Error>{
   let path = Path::new("assembly_tests/bx.s");

   let instruction = decode_single_16b_instruction(
      path,
      b".text\n.thumb\nBX r0\n"
   ).unwrap();

   assert_eq!(Opcode::BR_EXCHANGE,instruction);
   Ok(())
}

#[test]
fn should_recognise_cmn()->Result<(),std::io::Error>{
   let path = Path::new("assembly_tests/cmn.s");

   let instruction = decode_single_16b_instruction(
      path,
      b".text\n.thumb\nCMN r0,r1\n"
   ).unwrap();

   assert_eq!(Opcode::CMP_NEG_REG,instruction);
   Ok(())
}

#[test]
fn should_recognise_cmp()->Result<(),std::io::Error>{
   let path_imm8 = Path::new("assembly_tests/cmp_imm8.s");
   let path_reg_t1 = Path::new("assembly_tests/cmp_reg_t1.s");
   let path_reg_t2 = Path::new("assembly_tests/cmp_reg_t2.s");

   let inst_imm8 = decode_single_16b_instruction(
      path_imm8,
      b".text\n.thumb\nCMP r0,#255\n"
   ).unwrap();

   let inst_cmp_t1 = decode_single_16b_instruction(
      path_reg_t1, 
      b".text\n.thumb\nCMP r0,r1\n"
   ).unwrap();

   let inst_cmp_t2 = decode_single_16b_instruction(
      path_reg_t2,
      b".text\n.thumb\nCMP r8,r9\n"
   ).unwrap();

   assert_eq!(Opcode::CMP_Imm8,inst_imm8);
   assert_eq!(Opcode::CMP_REG_T1,inst_cmp_t1);
   assert_eq!(Opcode::CMP_REG_T2,inst_cmp_t2);
   Ok(())
}

#[test]
fn should_recognise_xor()->Result<(),std::io::Error>{
   let path_xor = Path::new("assembly_tests/xor.s");

   let instr = decode_single_16b_instruction(
      path_xor,
      b".text\n.thumb\nEOR r0,r1\n"
   ).unwrap();

   assert_eq!(Opcode::XOR_REG,instr);
   Ok(())
}

#[test]
fn should_recognise_ldm()->Result<(),std::io::Error>{
   let path_ldm = Path::new("assembly_tests/ldm.s");
   let path_ldmwb = Path::new("assembly_tests/ldmwb.s");

   let instr_exm_1 = decode_single_16b_instruction(
      path_ldm, 
      b".text\n.thumb\nLDM r3, {r1,r2,r3}\n"
   ).unwrap();

   let instr_exm_2 = decode_single_16b_instruction(
      path_ldmwb,
      b".text\n.thumb\nLDM r0!,{r1,r2,r3}\n"
   ).unwrap();

   assert_eq!(Opcode::LDM,instr_exm_1);
   assert_eq!(Opcode::LDM,instr_exm_2);
   Ok(())
}

#[test]
fn should_recognise_ldr()->Result<(),std::io::Error>{
   let path_imm5 = Path::new("assembly_tests/ldr_imm5.s");
   let path_imm8 = Path::new("assembly_tests/ldr_imm8.s");
   let path_pc_imm8 = Path::new("assembly_tests/ldr_pc_imm8.s");
   let path_reg = Path::new("assembly_tests/ldr_regs.s");

   let code_imm5 = decode_single_16b_instruction(
      path_imm5,
      b".text\n.thumb\nLDR r0,[r1,#20]\n"
   ).unwrap();

   let code_imm8 = decode_single_16b_instruction(
      path_imm8,
      b".text\n.thumb\nLDR r1,[SP,#124]\n"
   ).unwrap();

   let code_pc_imm8_alt = decode_single_16b_instruction(
      path_pc_imm8,
      b".text\n.thumb\nLDR r1,[PC,#16]\n"
   ).unwrap();
   
   let code_reg = decode_single_16b_instruction(
      path_reg,
      b".text\n.thumb\nLDR r1,[r2,r4]\n"
   ).unwrap();

   assert_eq!(Opcode::LDR_Imm5,code_imm5);
   assert_eq!(Opcode::LDR_SP_Imm8,code_imm8);
   assert_eq!(Opcode::LDR_PC_Imm8,code_pc_imm8_alt);
   assert_eq!(Opcode::LDR_REGS,code_reg);
   Ok(())
}

#[test]
fn should_recognise_ldrb()->Result<(),std::io::Error>{
   let path = Path::new("assembly_tests/ldrb.s");

   let imm5 = decode_single_16b_instruction(
      path,
      b".text\n.thumb\nLDRB r0,[r4,#24]\n"
   ).unwrap();

   let regs = decode_single_16b_instruction(
      path,
      b".text\n.thumb\n LDRB r0,[r3,r5]\n"
   ).unwrap();

   assert_eq!(Opcode::LDRB_Imm5,imm5);
   assert_eq!(Opcode::LDRB_REGS,regs);
   Ok(())
}

#[test]
fn should_recognise_ldrh()->Result<(),std::io::Error>{
   let path = Path::new("assembly_tests/ldrh.s");

   let imm5 = decode_single_16b_instruction(
      path,
      b".text\n.thumb\nLDRH r0,[r1,#58]\n"
   ).unwrap();

   let regs = decode_single_16b_instruction(
      path,
      b".text\n.thumb\nLDRH r2,[r1,r7]\n"
   ).unwrap();

   assert_eq!(Opcode::LDRH_Imm5,imm5);
   assert_eq!(Opcode::LDRH_REGS,regs);
   Ok(())
}

#[test]
fn should_recognise_ldrsb()->Result<(),std::io::Error>{
   let path = Path::new("assembly_tests/ldrsb.s");

   let regs = decode_single_16b_instruction(
      path,
      b".text\n.thumb\nLDRSB r0,[r1,r7]\n"
   ).unwrap();

   assert_eq!(Opcode::LDRSB_REGS,regs);
   Ok(())
}

#[test]
fn should_recognise_ldrsh()->Result<(),std::io::Error>{
   let path = Path::new("assembly_tests/ldrsh.s");

   let regs = decode_single_16b_instruction(
      path,
      b".text\n.thumb\nLDRSH r0,[r1,r7]\n"
   ).unwrap();

   assert_eq!(Opcode::LDRSH_REGS,regs);
   Ok(())
}

#[test]
fn should_recognise_lsl()->Result<(),std::io::Error>{
   let path = Path::new("assembly_tests/lsl.s");
   
   let imm5 = decode_single_16b_instruction(
      path,
      b".text\n.thumb\nLSL r0,r1,#31\n"
   ).unwrap();
   let regs = decode_single_16b_instruction(
      path,
      b".text\n.thumb\nLSL r0,r7\n"
   ).unwrap();

   assert_eq!(Opcode::LSL_Imm5,imm5);
   assert_eq!(Opcode::LSL_REGS,regs);
   Ok(())
}

#[test]
fn should_recognise_lsr()->Result<(),std::io::Error>{
   let path = Path::new("assembly_tests/lsr.s");
   
   let imm5 = decode_single_16b_instruction(
      path,
      b".text\n.thumb\nlsr r0,r1,#31\n"
   ).unwrap();
   let regs = decode_single_16b_instruction(
      path,
      b".text\n.thumb\nlsr r0,r7\n"
   ).unwrap();

   assert_eq!(Opcode::LSR_Imm5,imm5);
   assert_eq!(Opcode::LSR_REGS,regs);
   Ok(())
}

#[test]
fn should_recognise_mov()->Result<(),std::io::Error>{
   let path = Path::new("assembly_tests/mov.s");

   let imm8 = decode_single_16b_instruction(
      path,
      b".text\n.syntax unified\n.thumb\nMOVS.N r7,#255\n"
   ).unwrap();

   let regs_t1 = decode_single_16b_instruction(
      path,
      b".text\n.thumb\nMOV r7,r8\n"
   ).unwrap();

   let regs_t2 = assemble_and_decode(
      path,
      b".text\n.syntax unified\n.thumb\nMOVS.N r5,r7\n",
      asm_file_to_elf_with_t2_arm_encoding
   ).unwrap();

   assert_eq!(Opcode::MOV_Imm8,imm8);
   assert_eq!(Opcode::MOV_REGS_T1,regs_t1);
   assert_eq!(Opcode::MOV_REGS_T2,regs_t2);
   Ok(())
}

#[test]
fn should_recognise_mul()->Result<(),std::io::Error>{
   let path = Path::new("assembly_tests/mul.s");

   let regs = decode_single_16b_instruction(path, b".text\n.thumb\nMUL r0,r1\n").unwrap();
   
   assert_eq!(Opcode::MUL,regs);
   Ok(())
}

#[test]
fn should_recognise_mvn()->Result<(),std::io::Error>{
   let path = Path::new("assembly_tests/mvn.s");

   let regs = decode_single_16b_instruction(path, b".text\n.thumb\nMVN r0,r7\n").unwrap();

   assert_eq!(Opcode::MVN,regs);
   Ok(())
}

#[test]
fn should_recognise_nop()->Result<(),std::io::Error>{
   let path = Path::new("assembly_tests/nop.s");
   
   let encoding_t2 = assemble_and_decode(
      path,
      b".text\n.syntax unified\n.thumb\nNOP\n",
      asm_file_to_elf_with_t2_arm_encoding
   ).unwrap();

   assert_eq!(Opcode::NOP,encoding_t2);
   Ok(())
}

#[test]
fn should_recognise_orr()->Result<(),std::io::Error>{
   let path = Path::new("assembly_tests/orr.s");

   let t1 = decode_single_16b_instruction(path, b".text\n.thumb\nORR r0,r1\n").unwrap();

   assert_eq!(Opcode::ORR,t1);
   Ok(())
}

#[test]
fn should_recognise_pop()->Result<(),std::io::Error>{
   let path = Path::new("assembly_tests/pop.s");

   let t1 = decode_single_16b_instruction(path, b".text\n.thumb\nPOP {r0,r1}\n").unwrap();

   assert_eq!(Opcode::POP,t1);
   Ok(())
}

#[test]
fn should_recognise_push()->Result<(),std::io::Error>{
   let path = Path::new("assembly_tests/push.s");

   let t1 = decode_single_16b_instruction(path, b".text\n.thumb\nPUSH {r0,r1}\n").unwrap();

   assert_eq!(Opcode::PUSH,t1);
   Ok(())
}

#[test]
fn should_recognise_rev()->Result<(),std::io::Error>{
   let path = Path::new("assembly_tests/rev.s");

   let t1 = decode_single_16b_instruction(path, b".text\n.thumb\nREV r0,r1\n").unwrap();
   let c16 = decode_single_16b_instruction(path, b".text\n.thumb\nREV16 r0,r1\n").unwrap();
   let sh = decode_single_16b_instruction(path, b".text\n.thumb\nREVSH r0,r1\n").unwrap();

   assert_eq!(Opcode::REV,t1);
   assert_eq!(Opcode::REV_16,c16);
   assert_eq!(Opcode::REVSH,sh);
   Ok(())
}

#[test]
fn should_recognise_ror()->Result<(),std::io::Error>{
   let path = Path::new("assembly_tests/ror.s");

   let t1 = decode_single_16b_instruction(path, b".text\n.thumb\nROR r0,r7\n").unwrap();

   assert_eq!(Opcode::ROR,t1);
   Ok(())
}

#[test]
fn should_recognise_rsb()->Result<(),std::io::Error>{
   let path = Path::new("assembly_tests/rsb.s");

   let t1 = assemble_and_decode(
      path,
      b".text\n.thumb\nNEG r0,r1\n",// idk why gnu as doesn't assemble RSB D: NEG is the pre-UAL version of RSB tho:D
      asm_file_to_elf_with_t2_arm_encoding
   ).unwrap();

   assert_eq!(Opcode::RSB,t1);
   Ok(())
}


#[test]
fn should_recognise_sbc()->Result<(),std::io::Error>{
   let path = Path::new("assembly_tests/sbc.s");

   let t1 = decode_single_16b_instruction(
      path,
      b".text\n.thumb\nSBC r0,r1\n"
   ).unwrap();

   assert_eq!(Opcode::SBC,t1);
   Ok(())
}

#[test]
fn should_recognise_sev()->Result<(),std::io::Error>{
   let path = Path::new("assembly_tests/sev.s");

   let t1 = assemble_and_decode(
      path,
      b".text\n.thumb\nSEV\n",
      asm_file_to_elf_with_t2_arm_encoding
   ).unwrap();

   assert_eq!(Opcode::SEV,t1);
   Ok(())
}

#[test]
fn should_recognise_stm()->Result<(),std::io::Error>{
   let path = Path::new("assembly_tests/stm.s");

   let t1 = decode_single_16b_instruction(
      path,
      b".text\n.thumb\nSTM r0!,{r1,r3}\n"
   ).unwrap();

   assert_eq!(Opcode::STM,t1);
   Ok(())
}

#[test]
fn should_recognise_str()->Result<(),std::io::Error>{
   let path = Path::new("assembly_tests/str.s");

   let t1 = decode_single_16b_instruction( path, b".text\n.thumb\nSTR r0, [r1,#12]\n").unwrap();
   let t2 = decode_single_16b_instruction( path, b".text\n.thumb\nSTR r0, [SP, #224]\n").unwrap();
   let reg = decode_single_16b_instruction( path, b".text\n.thumb\nSTR r0, [r1, r2]\n").unwrap();

   assert_eq!(Opcode::STR_Imm5,t1);
   assert_eq!(Opcode::STR_Imm8,t2);
   assert_eq!(Opcode::STR_REG,reg);
   Ok(())
}

#[test]
fn should_recognise_strb()->Result<(),std::io::Error>{
   let path = Path::new("assembly_tests/strb.s");
   
   let im = decode_single_16b_instruction(path, b".text\n.thumb\nSTRB r0, [r1,#17]\n").unwrap();
   let reg = decode_single_16b_instruction(path, b".text\n.thumb\nSTRB r0, [r1,r2]\n").unwrap();

   assert_eq!(Opcode::STRB_Imm5,im);
   assert_eq!(Opcode::STRB_REG,reg);
   Ok(())
}

#[test]
fn should_recognise_strh()->Result<(),std::io::Error>{
   let path = Path::new("assembly_tests/strh.s");
   
   let im = decode_single_16b_instruction(path, b".text\n.thumb\nstrh r0, [r1,#62]\n").unwrap();
   let reg = decode_single_16b_instruction(path, b".text\n.thumb\nstrh r0, [r1,r2]\n").unwrap();

   assert_eq!(Opcode::STRH_Imm5,im);
   assert_eq!(Opcode::STRH_REG,reg);
   Ok(())
}

#[test]
fn should_recognise_sub()->Result<(),std::io::Error>{
   let path = Path::new("assembly_tests/sub.s");

   let im3 = decode_single_16b_instruction(path, b".text\n.thumb\nSUB r0,r1,#2\n").unwrap();
   let im8 = decode_single_16b_instruction(path, b".text\n.thumb\nSUB r0,#240\n").unwrap();
   let reg = decode_single_16b_instruction(path, b".text\n.thumb\nSUB r0,r1,r3\n").unwrap();
   let sp  = decode_single_16b_instruction(path, b".text\n.thumb\nSUB sp,sp,#16\n").unwrap();

   assert_eq!(Opcode::SUB_Imm3,im3);
   assert_eq!(Opcode::SUB_Imm8,im8);
   assert_eq!(Opcode::SUB_REG,reg);
   assert_eq!(Opcode::SUB_SP_Imm7,sp);
   Ok(())
}

#[test]
fn should_recognise_svc()->Result<(),std::io::Error>{
   let path = Path::new("assembly_tests/svc.s");

   let svc = decode_single_16b_instruction(path, b".text\n.thumb\nSVC #15\n").unwrap();

   assert_eq!(Opcode::SVC,svc);
   Ok(())
}

#[test]
fn should_recognise_sxt()->Result<(),std::io::Error>{
   let path = Path::new("assembly_tests/sxt_.s");

   let sxtb = decode_single_16b_instruction(path, b".text\n.thumb\n SXTB r0,r1\n").unwrap();
   let sxth = decode_single_16b_instruction(path, b".text\n.thumb\n SXTH r0,r1\n").unwrap();

   assert_eq!(Opcode::SXTB,sxtb);
   assert_eq!(Opcode::SXTH,sxth);
   Ok(())
}

#[test]
fn should_recognise_tst()->Result<(),std::io::Error>{
   let path = Path::new("assembly_tests/tst.s");

   let t1 = decode_single_16b_instruction(path, b".text\n.thumb\n TST r0,r1\n").unwrap();

   assert_eq!(Opcode::TST,t1);
   Ok(())
}

#[test]
fn should_recognise_udfw()->Result<(),std::io::Error>{
   let path = Path::new("assembly_tests/udfw.s");

   let t1 = assemble_and_decode_32b(
      path,
      b".text\n.thumb\n.syntax unified\n UDF.W #101\n",
      asm_file_to_elf_with_t2_arm_encoding
   ).unwrap();

   assert_eq!(Opcode::UNDEFINED,t1);
   Ok(())
}

#[test] 
fn should_recognise_uxt()->Result<(),std::io::Error>{
   let path = Path::new("assembly_tests/uxt.s");

   let uxtb = assemble_and_decode(
      path,
      b".text\n.thumb\nUXTB r0,r1\n", 
      asm_file_to_elf_with_t2_arm_encoding
   ).unwrap();

   let uxth = assemble_and_decode(
      path,
      b".text\n.thumb\nUXTH r0,r1\n", 
      asm_file_to_elf_with_t2_arm_encoding
   ).unwrap();

   assert_eq!(Opcode::UXTB,uxtb);
   assert_eq!(Opcode::UXTH,uxth);
   Ok(())
}

#[test]
fn should_recognise_wf()->Result<(),std::io::Error>{
   let path = Path::new("assembly_tests/wf.s");

   let wfe = assemble_and_decode(
      path,
      b".thumb\n.text\nWFE\n",
      asm_file_to_elf_with_t2_arm_encoding
   ).unwrap();

   let wfi = assemble_and_decode(
      path,
      b".thumb\n.text\nWFI\n",
      asm_file_to_elf_with_t2_arm_encoding
   ).unwrap();

   assert_eq!(Opcode::WFE,wfe);
   assert_eq!(Opcode::WFI,wfi);
   Ok(())
}

#[test]
fn should_recognise_yield()->Result<(),std::io::Error>{
   let path = Path::new("assembly_tests/yield.s");

   let yield_opc = assemble_and_decode(
      path,
      b".thumb\n.text\nYIELD\n",
      asm_file_to_elf_with_t2_arm_encoding
   ).unwrap();

   assert_eq!(Opcode::YIELD,yield_opc);
   Ok(())
}
