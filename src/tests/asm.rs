use crate::asm::decode::{Opcode,B32,B16};
use crate::asm::decode_operands::{
   get_operands, Operands
};

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

   println!("bin: {:#x},{:#x}",first_instr[0],first_instr[1]);
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

fn assemble_16b<F: Fn(&Path)->Result<PathBuf,IOError>> (path: &Path, asm: &[u8],assembler: F)->Result<HalfWord,IOError>{
   write_asm(path,asm)?;
   let elf = assembler(path)?;
   let opcodes = load_instruction_opcodes(&elf).unwrap();
   let first_instr: [u8;2] = [opcodes[0],opcodes[1]];
   Ok(first_instr)
}

fn assemble(path: &Path, asm: &[u8])->Result<Vec<u8>,ElfError>{
   write_asm(path,asm)?;
   let elf = asm_file_to_elf(path)?;
   load_instruction_opcodes(&elf)
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
pub fn should_recognise_adc()-> Result<(),std::io::Error>{
   let path = Path::new("assembly_tests/adc.s");
   let src_code = b".text\n\t.thumb\nADC r5, r1\n";

   let adc = decode_single_16b_instruction(
      path, 
      src_code
   ).unwrap();

   let bin = assemble_16b(
      path,
      src_code,
      asm_file_to_elf
   ).unwrap();

   if let Some(Operands::RegisterPair(dest,src)) =  get_operands(&Opcode::_16Bit(B16::ADCS),&bin){
      assert_eq!(5u8,dest.0);
      assert_eq!(1u8,src.0);
   }else{
      panic!("could not parse adc operands");
   }

   assert_eq!(Opcode::_16Bit(B16::ADCS),adc);
   Ok(())
}

#[test]
pub fn should_recognise_add_with_immediates()-> Result<(),std::io::Error>{
   let path = Path::new("assembly_tests/addi.s");
   write_asm(
      path,
      b".text\n\t.thumb\nADD r0,r1,#7\nADD r7,r1,#5\nADD r0, #255\nADD r2, #255\n"
   )?;
   let elf = asm_file_to_elf(path)?;
   let opcodes = load_instruction_opcodes(&elf).unwrap();
   let first_inst: [u8;2] = [opcodes[0],opcodes[1]];
   let second_instr: [u8;2] = [opcodes[2],opcodes[3]];
   let third_instr: [u8;2] = [opcodes[4],opcodes[5]];
   let fourth_instr: [u8;2] = [opcodes[6],opcodes[7]];

   if let Some(Operands::ADD_Imm3(dest_0,r0,imm3)) = get_operands(&Opcode::_16Bit(B16::ADD_Imm3),&second_instr){
      assert_eq!(dest_0.0,7u8);
      assert_eq!(r0.0,1);
      assert_eq!(imm3.0,5);
   }else {
      panic!("could not parse add::Imm3 operands");
   }

   if let Some(Operands::ADD_Imm8(dest_1,imm8)) = get_operands(&Opcode::_16Bit(B16::ADD_Imm8),&fourth_instr){
      assert_eq!(dest_1.0,2u8);
      assert_eq!(imm8.0,255);
   }else {
      panic!("could not parse add::Imm8 operands");
   }

   let instr: Opcode = (&first_inst).into();
   let secnd: Opcode = (&second_instr).into();
   let third: Opcode = (&third_instr).into();
   let fourth: Opcode = (&fourth_instr).into();

   println!("opcode raw bin{:?}",opcodes);
   std::fs::remove_file(path)?;
   std::fs::remove_file(elf)?;
   assert_eq!(Opcode::_16Bit(B16::ADD_Imm3),instr);
   assert_eq!(Opcode::_16Bit(B16::ADD_Imm3),secnd);
   assert_eq!(Opcode::_16Bit(B16::ADD_Imm8),third);
   assert_eq!(Opcode::_16Bit(B16::ADD_Imm8),fourth);

   Ok(())
}

#[test]
pub fn should_recognise_add_with_registers()-> Result<(),std::io::Error>{
   let path = Path::new("assembly_tests/addr.s");
   write_asm(path,b".text\n.thumb\nADD r3,r2,r7\nADD r7,r12\n")?;
   let elf = asm_file_to_elf_with_t2_arm_encoding(path)?;
   let opcodes = load_instruction_opcodes(&elf).unwrap();
   let first_instr: [u8;2] = [opcodes[0], opcodes[1]];
   let sec_instr: [u8;2] = [opcodes[2],opcodes[3]];

   let first: Opcode = (&first_instr).into();

   if let Some(Operands::RegisterTriplet(rd0,ra0,ra1)) = get_operands(&Opcode::_16Bit(B16::ADDS_REG),&first_instr){
      assert_eq!((rd0.0,ra0.0,ra1.0), (3,2,7));
   }else {
      panic!("could not decode ADD_reg operands");
   }

   let second: Opcode = (&sec_instr).into();
   if let Some(Operands::RegisterPair(rd1,rb0)) = get_operands(&Opcode::_16Bit(B16::ADDS_REG_T2),&sec_instr){
      assert_eq!((rd1.0,rb0.0), (7,12));
   }else{
      panic!("could not decode ADD_reg T2 operands");
   }

   std::fs::remove_file(path)?;
   std::fs::remove_file(elf)?;
   assert_eq!(Opcode::_16Bit(B16::ADDS_REG),first);
   assert_eq!(Opcode::_16Bit(B16::ADDS_REG_T2),second);
   Ok(())
}

#[test]
pub fn should_recognise_add_sp_and_immediate() -> Result<(),std::io::Error>{
   let path = Path::new("assembly_tests/add_sp_imm.s");
   write_asm(
      path,
      b".text\n.thumb\nADD r7,SP,#64\nADD SP,SP,#128\nADD r0,SP,r0\nADD SP,r6\n"
   )?;
   let elf = asm_file_to_elf(path)?;
   let opcodes = load_instruction_opcodes(&elf).unwrap();
   let first_instr: [u8;2] = [opcodes[0], opcodes[1]];
   let sec_instr: [u8;2] = [opcodes[2], opcodes[3]];
   let third_instr: [u8;2] = [opcodes[4],opcodes[5]];
   let fourth_instr: [u8;2] = [opcodes[6],opcodes[7]];

   let first: Opcode = (&first_instr).into();

   if let Some(Operands::ADD_REG_SP_IMM8(dest_0,imm8)) = get_operands(&Opcode::_16Bit(B16::ADD_REG_SP_IMM8), &first_instr){
      assert_eq!((dest_0.0,imm8.0),(7,64/4));
   }else{
      panic!("could not decode add_rep_sp_imm8 operands");
   }

   let second: Opcode = (&sec_instr).into();

   if let Some(Operands::INCR_SP_BY_IMM7(imm7)) = get_operands(&Opcode::_16Bit(B16::INCR_SP_BY_IMM7),&sec_instr){
      assert_eq!(128/4,imm7.0);
   }else{
      panic!("could not decode add_rep_sp_imm7 operands");
   }

   let third: Opcode = (&third_instr).into();
   if let Some(Operands::INCR_REG_BY_SP(reg)) = get_operands(&Opcode::_16Bit(B16::INCR_REG_BY_SP),&third_instr){
      assert_eq!(reg.0,0);
   }else{
      panic!("could not decode incr reg by sp operands");
   }

   let fourth: Opcode = (&fourth_instr).into();

   if let Some(Operands::INCR_SP_BY_REG(reg_1)) = get_operands(&Opcode::_16Bit(B16::INCR_SP_BY_REG), &fourth_instr){
      assert_eq!(reg_1.0,6);
   }else{
      panic!("could notdecode incr sp by reg operands");
   }

   std::fs::remove_file(path)?;
   std::fs::remove_file(elf)?;
   assert_eq!(Opcode::_16Bit(B16::ADD_REG_SP_IMM8),first);
   assert_eq!(Opcode::_16Bit(B16::INCR_SP_BY_IMM7),second);
   assert_eq!(Opcode::_16Bit(B16::INCR_REG_BY_SP),third);
   assert_eq!(Opcode::_16Bit(B16::INCR_SP_BY_REG),fourth);
   Ok(())
}

#[test]
pub fn should_recognise_adr()->Result<(),std::io::Error>{
   let path = Path::new("assembly_tests/adr.s");

   let bytes = assemble(
      path,
      b".text\n.thumb\nADR r7,_some_lbl\nNOP\n_some_lbl:"
   ).unwrap();
   let instruction: [u8;2] = [bytes[0],bytes[1]];
   let adr: Opcode = (&instruction).into();

   if let Some(Operands::ADR(dest,_literal)) =  get_operands(&Opcode::_16Bit(B16::ADR),&instruction){
      assert_eq!(dest.0,7);
   }else{
      panic!("could not decode adr operands");
   }

   assert_eq!(Opcode::_16Bit(B16::ADR),adr);
   Ok(())
}

#[test]
pub fn should_recognise_adr_with_alternate_syntax()->Result<(),std::io::Error>{
   let path = Path::new("assembly_tests/adr_alt.s");
   
   let instruction = decode_single_16b_instruction(
      path,
      b".text\n.thumb\nADD r0,PC,#8\n"
   ).unwrap();

   assert_eq!(Opcode::_16Bit(B16::ADR),instruction);
   Ok(())
}

#[test]
pub fn should_recognise_and_instruction()->Result<(),std::io::Error>{
   let path = Path::new("assembly_tests/and.s");
   write_asm(path,b".text\n.thumb\nAND r7,r1\nAND r2,r3,r2\n")?;
   let elf = asm_file_to_elf(path)?;
   let opcodes = load_instruction_opcodes(&elf).unwrap();
   let first_instr: [u8;2] = [opcodes[0],opcodes[1]];

   let first: Opcode = (&first_instr).into();

   if let Some(Operands::RegisterPair(dest,reg)) = get_operands(&Opcode::_16Bit(B16::ANDS),&first_instr){
      assert_eq!((dest.0,reg.0),(7,1));
   }else{
      panic!("could not decode 'and' instruction operands");
   }

   std::fs::remove_file(path)?;
   std::fs::remove_file(elf)?;
   assert_eq!(Opcode::_16Bit(B16::ANDS),first);
   Ok(())
}

#[test]
pub fn should_recognise_asr()->Result<(),std::io::Error>{
   let path = Path::new("assembly_tests/asr.s");
   write_asm(path,b".text\n.thumb\nASR r7,r5,#24\nASR r3,r5\n")?;
   let elf = asm_file_to_elf(path)?;
   let opcodes = load_instruction_opcodes(&elf).unwrap();
   let first_instr: [u8;2] = [opcodes[0],opcodes[1]];
   let second_instr: [u8;2] = [opcodes[2],opcodes[3]];

   let first: Opcode = (&first_instr).into();
   if let Some(Operands::ASRS_Imm5(dest,src,imm5)) = get_operands(&Opcode::_16Bit(B16::ASRS_Imm5),&first_instr){
      assert_eq!((dest.0,src.0,imm5.0),(7,5,24));
   }else{
      panic!("did not parse ASR imm5 operands");
   }

   let second: Opcode = (&second_instr).into();

   if let Some(Operands::RegisterPair(dest_1,other)) = get_operands(&Opcode::_16Bit(B16::ASRS_REG),&second_instr){
      assert_eq!((dest_1.0,other.0),(3,5));
   }

   std::fs::remove_file(path)?;
   std::fs::remove_file(elf)?;
   assert_eq!(Opcode::_16Bit(B16::ASRS_Imm5),first);
   assert_eq!(Opcode::_16Bit(B16::ASRS_REG),second);
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
         Opcode::_16Bit(B16::B_ALWAYS),
         Opcode::_16Bit(B16::BEQ),
         Opcode::_16Bit(B16::BNEQ),
         Opcode::_16Bit(B16::B_CARRY_IS_SET),
         Opcode::_16Bit(B16::B_CARRY_IS_CLEAR),
         Opcode::_16Bit(B16::B_IF_NEGATIVE),
         Opcode::_16Bit(B16::B_IF_POSITIVE),
         Opcode::_16Bit(B16::B_IF_OVERFLOW),
         Opcode::_16Bit(B16::B_IF_NO_OVERFLOW),
         Opcode::_16Bit(B16::B_UNSIGNED_HIGHER),
         Opcode::_16Bit(B16::B_UNSIGNED_LOWER_OR_SAME),
         Opcode::_16Bit(B16::B_GTE),
         Opcode::_16Bit(B16::B_LT),
         Opcode::_16Bit(B16::B_GT),
         Opcode::_16Bit(B16::B_LTE)
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

   let bytes = assemble(
      path,
      b".text\n.thumb\nBIC r5,r2\n"
   ).unwrap();
   let hw: [u8;2] = [bytes[0],bytes[1]];

   if let Some(Operands::RegisterPair(dest,reg)) = get_operands(&Opcode::_16Bit(B16::BIT_CLEAR_REGISTER), &hw){
      println!("{:?},{}",dest,reg);
      assert_eq!((dest.0,reg.0),(5,2));
   }else{
      panic!("could not decode bic operands");
   }

   let bic: Opcode = Opcode::from(&hw);
   assert_eq!(Opcode::_16Bit(B16::BIT_CLEAR_REGISTER),bic);
   Ok(())
}

#[test]
fn should_recognise_breakpoint()->Result<(),std::io::Error>{
   let path = Path::new("assembly_tests/bkpt.s");
   
   let bytes = assemble(
      path,
      b".text\n.thumb\nBKPT #255\n"
   ).unwrap();
   let hw: [u8;2] = [bytes[0],bytes[1]];

   if let Some(Operands::BREAKPOINT(imm8)) = get_operands(&Opcode::_16Bit(B16::BREAKPOINT), &hw){
      assert_eq!(255,imm8.0);
   }else{
      panic!("could not detect breakpoint arguements");
   }

   let instruction = Opcode::from(&hw);
   assert_eq!(Opcode::_16Bit(B16::BREAKPOINT),instruction);
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
   assert_eq!(Opcode::_32Bit(B32::BR_AND_LNK),first);
   Ok(())
}

#[test]
fn should_recognise_blx()->Result<(),std::io::Error>{
   let path = Path::new("assembly_tests/blx.s");

   let instruction = decode_single_16b_instruction(
      path,
      b".text\n.thumb\nBLX r0\n"
   ).unwrap();

   assert_eq!(Opcode::_16Bit(B16::BR_LNK_EXCHANGE),instruction);
   Ok(())
}

#[test]
fn should_recognise_bx()->Result<(),std::io::Error>{
   let path = Path::new("assembly_tests/bx.s");

   let instruction = decode_single_16b_instruction(
      path,
      b".text\n.thumb\nBX r0\n"
   ).unwrap();

   assert_eq!(Opcode::_16Bit(B16::BR_EXCHANGE),instruction);
   Ok(())
}

#[test]
fn should_recognise_cmn()->Result<(),std::io::Error>{
   let path = Path::new("assembly_tests/cmn.s");

   let instruction = decode_single_16b_instruction(
      path,
      b".text\n.thumb\nCMN r0,r1\n"
   ).unwrap();

   assert_eq!(Opcode::_16Bit(B16::CMP_NEG_REG),instruction);
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

   assert_eq!(Opcode::_16Bit(B16::CMP_Imm8),inst_imm8);
   assert_eq!(Opcode::_16Bit(B16::CMP_REG_T1),inst_cmp_t1);
   assert_eq!(Opcode::_16Bit(B16::CMP_REG_T2),inst_cmp_t2);
   Ok(())
}

#[test]
fn should_recognise_xor()->Result<(),std::io::Error>{
   let path_xor = Path::new("assembly_tests/xor.s");

   let instr = decode_single_16b_instruction(
      path_xor,
      b".text\n.thumb\nEOR r0,r1\n"
   ).unwrap();

   assert_eq!(Opcode::_16Bit(B16::XOR_REG),instr);
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

   assert_eq!(Opcode::_16Bit(B16::LDM),instr_exm_1);
   assert_eq!(Opcode::_16Bit(B16::LDM),instr_exm_2);
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

   assert_eq!(Opcode::_16Bit(B16::LDR_Imm5),code_imm5);
   assert_eq!(Opcode::_16Bit(B16::LDR_SP_Imm8),code_imm8);
   assert_eq!(Opcode::_16Bit(B16::LDR_PC_Imm8),code_pc_imm8_alt);
   assert_eq!(Opcode::_16Bit(B16::LDR_REGS),code_reg);
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

   assert_eq!(Opcode::_16Bit(B16::LDRB_Imm5),imm5);
   assert_eq!(Opcode::_16Bit(B16::LDRB_REGS),regs);
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

   assert_eq!(Opcode::_16Bit(B16::LDRH_Imm5),imm5);
   assert_eq!(Opcode::_16Bit(B16::LDRH_REGS),regs);
   Ok(())
}

#[test]
fn should_recognise_ldrsb()->Result<(),std::io::Error>{
   let path = Path::new("assembly_tests/ldrsb.s");

   let regs = decode_single_16b_instruction(
      path,
      b".text\n.thumb\nLDRSB r0,[r1,r7]\n"
   ).unwrap();

   assert_eq!(Opcode::_16Bit(B16::LDRSB_REGS),regs);
   Ok(())
}

#[test]
fn should_recognise_ldrsh()->Result<(),std::io::Error>{
   let path = Path::new("assembly_tests/ldrsh.s");

   let regs = decode_single_16b_instruction(
      path,
      b".text\n.thumb\nLDRSH r0,[r1,r7]\n"
   ).unwrap();

   assert_eq!(Opcode::_16Bit(B16::LDRSH_REGS),regs);
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

   assert_eq!(Opcode::_16Bit(B16::LSL_Imm5),imm5);
   assert_eq!(Opcode::_16Bit(B16::LSL_REGS),regs);
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

   assert_eq!(Opcode::_16Bit(B16::LSR_Imm5),imm5);
   assert_eq!(Opcode::_16Bit(B16::LSR_REGS),regs);
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

   assert_eq!(Opcode::_16Bit(B16::MOV_Imm8),imm8);
   assert_eq!(Opcode::_16Bit(B16::MOV_REGS_T1),regs_t1);
   assert_eq!(Opcode::_16Bit(B16::MOV_REGS_T2),regs_t2);
   Ok(())
}

#[test]
fn should_recognise_mul()->Result<(),std::io::Error>{
   let path = Path::new("assembly_tests/mul.s");

   let regs = decode_single_16b_instruction(path, b".text\n.thumb\nMUL r0,r1\n").unwrap();
   
   assert_eq!(Opcode::_16Bit(B16::MUL),regs);
   Ok(())
}

#[test]
fn should_recognise_mvn()->Result<(),std::io::Error>{
   let path = Path::new("assembly_tests/mvn.s");

   let regs = decode_single_16b_instruction(path, b".text\n.thumb\nMVN r0,r7\n").unwrap();

   assert_eq!(Opcode::_16Bit(B16::MVN),regs);
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

   assert_eq!(Opcode::_16Bit(B16::NOP),encoding_t2);
   Ok(())
}

#[test]
fn should_recognise_orr()->Result<(),std::io::Error>{
   let path = Path::new("assembly_tests/orr.s");

   let t1 = decode_single_16b_instruction(path, b".text\n.thumb\nORR r0,r1\n").unwrap();

   assert_eq!(Opcode::_16Bit(B16::ORR),t1);
   Ok(())
}

#[test]
fn should_recognise_pop()->Result<(),std::io::Error>{
   let path = Path::new("assembly_tests/pop.s");

   let t1 = decode_single_16b_instruction(path, b".text\n.thumb\nPOP {r0,r1}\n").unwrap();
   let sc = decode_single_16b_instruction(path, b".text\n.thumb\nPOP {r0,PC}\n").unwrap();

   assert_eq!(Opcode::_16Bit(B16::POP),t1);
   assert_eq!(Opcode::_16Bit(B16::POP),sc);
   Ok(())
}

#[test]
fn should_recognise_push()->Result<(),std::io::Error>{
   let path = Path::new("assembly_tests/push.s");

   let t1 = decode_single_16b_instruction(path, b".text\n.thumb\nPUSH {r0,r1}\n").unwrap();

   assert_eq!(Opcode::_16Bit(B16::PUSH),t1);
   Ok(())
}

#[test]
fn should_recognise_rev()->Result<(),std::io::Error>{
   let path = Path::new("assembly_tests/rev.s");

   let t1 = decode_single_16b_instruction(path, b".text\n.thumb\nREV r0,r1\n").unwrap();
   let c16 = decode_single_16b_instruction(path, b".text\n.thumb\nREV16 r0,r1\n").unwrap();
   let sh = decode_single_16b_instruction(path, b".text\n.thumb\nREVSH r0,r1\n").unwrap();

   assert_eq!(Opcode::_16Bit(B16::REV),t1);
   assert_eq!(Opcode::_16Bit(B16::REV_16),c16);
   assert_eq!(Opcode::_16Bit(B16::REVSH),sh);
   Ok(())
}

#[test]
fn should_recognise_ror()->Result<(),std::io::Error>{
   let path = Path::new("assembly_tests/ror.s");

   let t1 = decode_single_16b_instruction(path, b".text\n.thumb\nROR r0,r7\n").unwrap();

   assert_eq!(Opcode::_16Bit(B16::ROR),t1);
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

   assert_eq!(Opcode::_16Bit(B16::RSB),t1);
   Ok(())
}


#[test]
fn should_recognise_sbc()->Result<(),std::io::Error>{
   let path = Path::new("assembly_tests/sbc.s");

   let t1 = decode_single_16b_instruction(
      path,
      b".text\n.thumb\nSBC r0,r1\n"
   ).unwrap();

   assert_eq!(Opcode::_16Bit(B16::SBC),t1);
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

   assert_eq!(Opcode::_16Bit(B16::SEV),t1);
   Ok(())
}

#[test]
fn should_recognise_stm()->Result<(),std::io::Error>{
   let path = Path::new("assembly_tests/stm.s");

   let t1 = decode_single_16b_instruction(
      path,
      b".text\n.thumb\nSTM r0!,{r1,r3}\n"
   ).unwrap();

   assert_eq!(Opcode::_16Bit(B16::STM),t1);
   Ok(())
}

#[test]
fn should_recognise_str()->Result<(),std::io::Error>{
   let path = Path::new("assembly_tests/str.s");

   let t1 = decode_single_16b_instruction( path, b".text\n.thumb\nSTR r0, [r1,#12]\n").unwrap();
   let t2 = decode_single_16b_instruction( path, b".text\n.thumb\nSTR r0, [SP, #224]\n").unwrap();
   let reg = decode_single_16b_instruction( path, b".text\n.thumb\nSTR r0, [r1, r2]\n").unwrap();

   assert_eq!(Opcode::_16Bit(B16::STR_Imm5),t1);
   assert_eq!(Opcode::_16Bit(B16::STR_Imm8),t2);
   assert_eq!(Opcode::_16Bit(B16::STR_REG),reg);
   Ok(())
}

#[test]
fn should_recognise_strb()->Result<(),std::io::Error>{
   let path = Path::new("assembly_tests/strb.s");
   
   let im = decode_single_16b_instruction(path, b".text\n.thumb\nSTRB r0, [r1,#17]\n").unwrap();
   let reg = decode_single_16b_instruction(path, b".text\n.thumb\nSTRB r0, [r1,r2]\n").unwrap();

   assert_eq!(Opcode::_16Bit(B16::STRB_Imm5),im);
   assert_eq!(Opcode::_16Bit(B16::STRB_REG),reg);
   Ok(())
}

#[test]
fn should_recognise_strh()->Result<(),std::io::Error>{
   let path = Path::new("assembly_tests/strh.s");
   
   let im = decode_single_16b_instruction(path, b".text\n.thumb\nstrh r0, [r1,#62]\n").unwrap();
   let reg = decode_single_16b_instruction(path, b".text\n.thumb\nstrh r0, [r1,r2]\n").unwrap();

   assert_eq!(Opcode::_16Bit(B16::STRH_Imm5),im);
   assert_eq!(Opcode::_16Bit(B16::STRH_REG),reg);
   Ok(())
}

#[test]
fn should_recognise_sub()->Result<(),std::io::Error>{
   let path = Path::new("assembly_tests/sub.s");

   let im3 = decode_single_16b_instruction(path, b".text\n.thumb\nSUB r0,r1,#2\n").unwrap();
   let im8 = decode_single_16b_instruction(path, b".text\n.thumb\nSUB r0,#240\n").unwrap();
   let reg = decode_single_16b_instruction(path, b".text\n.thumb\nSUB r0,r1,r3\n").unwrap();
   let sp  = decode_single_16b_instruction(path, b".text\n.thumb\nSUB sp,sp,#16\n").unwrap();

   assert_eq!(Opcode::_16Bit(B16::SUB_Imm3),im3);
   assert_eq!(Opcode::_16Bit(B16::SUB_Imm8),im8);
   assert_eq!(Opcode::_16Bit(B16::SUB_REG),reg);
   assert_eq!(Opcode::_16Bit(B16::SUB_SP_Imm7),sp);
   Ok(())
}

#[test]
fn should_recognise_svc()->Result<(),std::io::Error>{
   let path = Path::new("assembly_tests/svc.s");

   let svc = decode_single_16b_instruction(path, b".text\n.thumb\nSVC #15\n").unwrap();

   assert_eq!(Opcode::_16Bit(B16::SVC),svc);
   Ok(())
}

#[test]
fn should_recognise_sxt()->Result<(),std::io::Error>{
   let path = Path::new("assembly_tests/sxt_.s");

   let sxtb = decode_single_16b_instruction(path, b".text\n.thumb\n SXTB r0,r1\n").unwrap();
   let sxth = decode_single_16b_instruction(path, b".text\n.thumb\n SXTH r0,r1\n").unwrap();

   assert_eq!(Opcode::_16Bit(B16::SXTB),sxtb);
   assert_eq!(Opcode::_16Bit(B16::SXTH),sxth);
   Ok(())
}

#[test]
fn should_recognise_tst()->Result<(),std::io::Error>{
   let path = Path::new("assembly_tests/tst.s");

   let t1 = decode_single_16b_instruction(path, b".text\n.thumb\n TST r0,r1\n").unwrap();

   assert_eq!(Opcode::_16Bit(B16::TST),t1);
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

   assert_eq!(Opcode::_32Bit(B32::UNDEFINED),t1);
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

   assert_eq!(Opcode::_16Bit(B16::UXTB),uxtb);
   assert_eq!(Opcode::_16Bit(B16::UXTH),uxth);
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

   assert_eq!(Opcode::_16Bit(B16::WFE),wfe);
   assert_eq!(Opcode::_16Bit(B16::WFI),wfi);
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

   assert_eq!(Opcode::_16Bit(B16::YIELD),yield_opc);
   Ok(())
}
