use crate::binutils::get_set_bits;
use crate::asm::decode::{Opcode,B32,B16, instruction_size, InstructionSize};
use crate::asm::decode_operands::{
   get_operands, Operands, get_operands_32b
};

use crate::elf::decoder::ElfError;
use crate::asm::{HalfWord, STACK_POINTER, PROGRAM_COUNTER};
use crate::system::registers::SpecialRegister;
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

pub fn write_asm(path: &Path, data: &[u8])->Result<File,std::io::Error>{
   println!("writing  asm to {:?}",path);
   let mut file = File::create(path)?;
   file.write_all(data)?;
   println!("written asm to {:?}",file);
   Ok(file)
}

pub fn asm_file_to_elf(path: &Path)->Result<PathBuf,std::io::Error>{
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

fn asm_file_to_elf_armv6(path: &Path)->Result<PathBuf,std::io::Error>{
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
      .arg("-march=armv6-m")
      .arg("-o")
      .arg(fname)
      .status()
      .expect("could not assemble");

   Ok(ret)
}

pub fn asm_file_to_elf_armv6m(path: &Path)->Result<PathBuf,std::io::Error>{
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
      .arg("-march=armv6-m")
      .arg("-o")
      .arg(fname)
      .status()
      .expect("could not assemble");

   Ok(ret)
}

pub fn asm_file_to_elf_armv6t2(path: &Path)->Result<PathBuf,std::io::Error>{
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
      .expect("could not assemble");

   Ok(ret)
}

fn decode_single_16b_instruction(path: &Path, asm: &[u8])->Result<Opcode,std::io::Error>{
   write_asm(path,asm)?;
   let elf = asm_file_to_elf(path)?;
   let opcodes = load_instruction_opcodes(&elf).unwrap();
   let first_instr: [u8;2] = [opcodes[0],opcodes[1]];

   println!("bin: {:#x},{:#x}",first_instr[0],first_instr[1]);
   let first: Opcode = (first_instr).into();

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

   let first: Opcode = (first_instr).into();

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

fn assemble_by<F: Fn(&Path)->Result<PathBuf,IOError>>(path: &Path, asm: &[u8], assembler: F)->Result<Vec<u8>,ElfError>{
   write_asm(path,asm)?;
   let elf = assembler(path)?;
   load_instruction_opcodes(&elf)
}


fn assemble_by_32b<F: Fn(&Path)->Result<PathBuf,IOError>> (path: &Path, asm: &[u8],assembler: F)->Result<Vec<u8>,IOError>{
   write_asm(path,asm)?;
   println!("running assembler");
   let elf = assembler(path)?;
   let opcodes = load_instruction_opcodes(&elf).unwrap();

   std::fs::remove_file(path)?;
   std::fs::remove_file(elf)?;

   Ok(opcodes)
}

fn assemble_and_decode_32b<F: Fn(&Path)->Result<PathBuf,IOError>> (path: &Path, asm: &[u8],assembler: F)->Result<Opcode,IOError>{
   write_asm(path,asm)?;
   println!("running assembler");
   let elf = assembler(path)?;
   let opcodes = load_instruction_opcodes(&elf).unwrap();
   let first_instr: [u8;4] = [opcodes[0],opcodes[1],opcodes[2],opcodes[3]];

   for i in first_instr{
      print!("{:x}",i);
   }
   println!();
   let first: Opcode = (first_instr).into();

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

   if let Some(Operands::RegisterPair(dest,src)) =  get_operands(&Opcode::_16Bit(B16::ADCS),bin){
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

   if let Some(Operands::RegPairImm3(dest_0,r0,imm3)) = get_operands(&Opcode::_16Bit(B16::ADD_Imm3),second_instr){
      assert_eq!(dest_0.0,7u8);
      assert_eq!(r0.0,1);
      assert_eq!(imm3.0,5);
   }else {
      panic!("could not parse add::Imm3 operands");
   }

   if let Some(Operands::DestImm8(dest_1,imm8)) = get_operands(&Opcode::_16Bit(B16::ADD_Imm8),fourth_instr){
      assert_eq!(dest_1.0,2u8);
      assert_eq!(imm8.0,255);
   }else {
      panic!("could not parse add::Imm8 operands");
   }

   let instr: Opcode = (first_inst).into();
   let secnd: Opcode = (second_instr).into();
   let third: Opcode = (third_instr).into();
   let fourth: Opcode = (fourth_instr).into();

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
   write_asm(path,b".text\n.thumb\nADD r3,r2,r7\nADD r8,r12\n")?;
   let elf = asm_file_to_elf_armv6t2(path)?;
   let opcodes = load_instruction_opcodes(&elf).unwrap();
   let first_instr: [u8;2] = [opcodes[0], opcodes[1]];
   let sec_instr: [u8;2] = [opcodes[2],opcodes[3]];

   let first: Opcode = (first_instr).into();

   if let Some(Operands::RegisterTriplet(rd0,ra0,ra1)) = get_operands(&Opcode::_16Bit(B16::ADDS_REG),first_instr){
      assert_eq!((rd0.0,ra0.0,ra1.0), (3,2,7));
   }else {
      panic!("could not decode ADD_reg operands");
   }

   let second: Opcode = (sec_instr).into();
   if let Some(Operands::RegisterPair(rd1,rb0)) = get_operands(&Opcode::_16Bit(B16::ADDS_REG_T2),sec_instr){
      assert_eq!((rd1.0,rb0.0), (8,12));
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
      b".text\n.thumb\nADD r7,SP,#64\nADD SP,SP,#128\nADD r3,SP,r3\nADD SP,r6\n"
   )?;
   let elf = asm_file_to_elf(path)?;
   let opcodes = load_instruction_opcodes(&elf).unwrap();
   let first_instr: [u8;2] = [opcodes[0], opcodes[1]];
   let sec_instr: [u8;2] = [opcodes[2], opcodes[3]];
   let third_instr: [u8;2] = [opcodes[4],opcodes[5]];
   let fourth_instr: [u8;2] = [opcodes[6],opcodes[7]];

   let first: Opcode = (first_instr).into();

   if let Some(Operands::ADD_REG_SP_IMM8(dest_0,imm8)) = get_operands(&Opcode::_16Bit(B16::ADD_REG_SP_IMM8), first_instr){
      assert_eq!((dest_0.0,imm8.0),(7,64));
   }else{
      panic!("could not decode add_rep_sp_imm8 operands");
   }

   let second: Opcode = (sec_instr).into();

   if let Some(Operands::INCR_SP_BY_IMM7(imm7)) = get_operands(&Opcode::_16Bit(B16::INCR_SP_BY_IMM7),sec_instr){
      assert_eq!(128,imm7.0);
   }else{
      panic!("could not decode add_rep_sp_imm7 operands");
   }

   let third: Opcode = (third_instr).into();
   if let Some(Operands::RegisterPair(a,b)) = get_operands(&Opcode::_16Bit(B16::ADDS_REG_T2),third_instr){
      assert_eq!((a.0,b.0),(3,13));
   }else{
      panic!("could not decode add ");
   }

   let fourth: Opcode = (fourth_instr).into();

   if let Some(Operands::INCR_SP_BY_REG(reg)) = get_operands(&Opcode::_16Bit(B16::INCR_SP_BY_REG), fourth_instr){
      assert_eq!(reg.0,6);
   }else{
      panic!("could not decode add");
   }

   std::fs::remove_file(path)?;
   std::fs::remove_file(elf)?;
   assert_eq!(Opcode::_16Bit(B16::ADD_REG_SP_IMM8),first);
   assert_eq!(Opcode::_16Bit(B16::INCR_SP_BY_IMM7),second);
   assert_eq!(Opcode::_16Bit(B16::ADDS_REG_T2),third);
   assert_eq!(Opcode::_16Bit(B16::ADDS_REG_T2),fourth);
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
   let adr: Opcode = (instruction).into();

   if let Some(Operands::ADR(dest,_literal)) =  get_operands(&Opcode::_16Bit(B16::ADR),instruction){
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

   let first: Opcode = (first_instr).into();

   if let Some(Operands::RegisterPair(dest,reg)) = get_operands(&Opcode::_16Bit(B16::ANDS),first_instr){
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

   let first: Opcode = (first_instr).into();
   if let Some(Operands::ASRS_Imm5(dest,src,imm5)) = get_operands(&Opcode::_16Bit(B16::ASRS_Imm5),first_instr){
      assert_eq!((dest.0,src.0,imm5.0),(7,5,24));
   }else{
      panic!("did not parse ASR imm5 operands");
   }

   let second: Opcode = (second_instr).into();

   if let Some(Operands::RegisterPair(dest_1,other)) = get_operands(&Opcode::_16Bit(B16::ASRS_REG),second_instr){
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
      opcodes.push(Opcode::from(halfword));
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

   if let Some(Operands::RegisterPair(dest,reg)) = get_operands(&Opcode::_16Bit(B16::BIT_CLEAR_REGISTER), hw){
      println!("{:?},{}",dest,reg);
      assert_eq!((dest.0,reg.0),(5,2));
   }else{
      panic!("could not decode bic operands");
   }

   let bic: Opcode = Opcode::from(hw);
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

   if let Some(Operands::BREAKPOINT(imm8)) = get_operands(&Opcode::_16Bit(B16::BREAKPOINT), hw){
      assert_eq!(255,imm8.0);
   }else{
      panic!("could not detect breakpoint arguements");
   }

   let instruction = Opcode::from(hw);
   assert_eq!(Opcode::_16Bit(B16::BREAKPOINT),instruction);
   Ok(())
}

#[test]
fn should_calculate_labels_correctly()->Result<(),std::io::Error>{
   let path = Path::new("assembly_tests/labels.s");
   let src_code = concat!(
      ".text\n",
      ".thumb\n",
      ".syntax unified\n",
      "BEQ.N .\n",
   );

   let bytes = assemble_by(path,src_code.as_bytes(),asm_file_to_elf_armv6t2).unwrap();

   let opcode: Opcode = ([bytes[0],bytes[1]]).into();
   if let Some(Operands::COND_BRANCH(literal)) = get_operands(&Opcode::_16Bit(B16::BEQ), [bytes[0],bytes[1]]){
      assert_eq!(literal,0);
   }else{
      panic!("could not decode branch");
   }

   let src_code = concat!(
      ".text\n",
      ".thumb\n",
      ".syntax unified\n",
      "BEQ.N .+2\n",
   );

   let bytes_1 = assemble_by(path,src_code.as_bytes(),asm_file_to_elf_armv6t2).unwrap();

   if let Some(Operands::COND_BRANCH(literal)) = get_operands(&Opcode::_16Bit(B16::BEQ), [bytes_1[0],bytes_1[1]]){
      assert_eq!(literal,2);
   }else{
      panic!("could not decode branch");
   }

   let src_code = concat!(
      ".text\n",
      ".thumb\n",
      ".syntax unified\n",
      "BEQ.N .-20\n",
   );

   let bytes = assemble(path,src_code.as_bytes()).unwrap();

   if let Some(Operands::COND_BRANCH(literal)) = get_operands(&Opcode::_16Bit(B16::BEQ), [bytes[0],bytes[1]]){
      assert_eq!(literal,-20);
   }else{
      panic!("could not decode branch");
   }
   assert_eq!(opcode,Opcode::_16Bit(B16::BEQ));

   let src_code = concat!(
      ".text\n",
      ".thumb\n",
      ".syntax unified\n",
      "BAL.N .-200\n",
   );
   let bytes = assemble(path,src_code.as_bytes()).unwrap();
   let opcode: Opcode = ([bytes[0],bytes[1]]).into();
   assert_eq!(opcode,Opcode::_16Bit(B16::B_ALWAYS));
   if let Some(Operands::B_ALWAYS(literal)) = get_operands(&Opcode::_16Bit(B16::B_ALWAYS), [bytes[0],bytes[1]]){
      assert_eq!(literal,-200);
   }else{
      panic!("could not decode branch");
   }
   Ok(())
}

#[test]
fn should_recognise_bl()->Result<(),std::io::Error>{
   let path = Path::new("assembly_tests/bl.s");
   write_asm(path,b".text\n.thumb\n.syntax unified\nBL.W .-252\n\n")?;
   let elf = asm_file_to_elf(path)?;
   let opcodes = load_instruction_opcodes(&elf).unwrap();
   let first_instr: [u8;4] = [opcodes[0],opcodes[1],opcodes[2],opcodes[3]];

   let first: Opcode = (first_instr).into();

   if let Some(Operands::BR_LNK(literal)) = get_operands_32b(&Opcode::_32Bit(B32::BR_AND_LNK), first_instr){
      assert_eq!(-252,literal);
   }else{
      panic!("could not detect literal");
   }


   std::fs::remove_file(path)?;
   std::fs::remove_file(elf)?;
   assert_eq!(Opcode::_32Bit(B32::BR_AND_LNK),first);
   Ok(())
}

#[test]
fn should_recognise_blx()->Result<(),std::io::Error>{
   let path = Path::new("assembly_tests/blx.s");

   let bytes = assemble(
      path,
      b".text\n.thumb\nBLX r11\n"
   ).unwrap();

   let first: [u8;2] = [bytes[0],bytes[1]];
   let instruction: Opcode = (first).into();
   if let Some(Operands::BR_LNK_EXCHANGE(reg)) = get_operands(&Opcode::_16Bit(B16::BR_LNK_EXCHANGE), first){
      assert_eq!(11,reg.0);
   }else{
      panic!("could not detect blx operands");
   }
   assert_eq!(Opcode::_16Bit(B16::BR_LNK_EXCHANGE),instruction);
   Ok(())
}

#[test]
fn should_recognise_bx()->Result<(),std::io::Error>{
   let path = Path::new("assembly_tests/bx.s");

   let bytes = assemble(
      path,
      b".text\n.thumb\nBX r7\n"
   ).unwrap();

   let first: [u8;2] = [bytes[0],bytes[1]];
   let instruction: Opcode = (first).into();

   if let Some(Operands::BR_EXCHANGE(reg)) = get_operands(&Opcode::_16Bit(B16::BR_EXCHANGE), first){
      assert_eq!(7,reg.0);
   }else{
      panic!("could not detect bx operands");
   }
   assert_eq!(Opcode::_16Bit(B16::BR_EXCHANGE),instruction);
   Ok(())
}

#[test]
fn should_recognise_cmn()->Result<(),std::io::Error>{
   let path = Path::new("assembly_tests/cmn.s");

   let bytes = assemble(
      path,
      b".text\n.thumb\nCMN r5,r7\n"
   ).unwrap();

   let first: [u8;2] = [bytes[0],bytes[1]];
   let instruction: Opcode = (first).into();

   if let Some(Operands::PureRegisterPair(first,sec)) = get_operands(&Opcode::_16Bit(B16::CMP_NEG_REG),first){
      assert_eq!((first.0,sec.0),(5,7));
   }else{
      panic!("could not detect cmn  operands");
   }

   assert_eq!(Opcode::_16Bit(B16::CMP_NEG_REG),instruction);
   Ok(())
}

#[test]
fn should_recognise_cps()->Result<(),std::io::Error>{
   let path = Path::new("assembly_tests/cps.s");
   let bytes = assemble(path,b".text\n.thumb\nCPSIE i\nCPSID i").unwrap();
   let enable: Opcode = ([bytes[0],bytes[1]]).into();
   let disable: Opcode = ([bytes[2],bytes[3]]).into();

   if let Some(Operands::EnableInterupt(flag)) = get_operands(&Opcode::_16Bit(B16::CPS), [bytes[0],bytes[1]]){
      assert_eq!(flag,false);
   }else{
      panic!("could not parse CPS");
   }

   if let Some(Operands::EnableInterupt(flag)) = get_operands(&Opcode::_16Bit(B16::CPS), [bytes[2],bytes[3]]){
      assert_eq!(flag,true);
   }else{
      panic!("could not parse CPS");
   }
   assert_eq!(Opcode::_16Bit(B16::CPS),enable);
   assert_eq!(Opcode::_16Bit(B16::CPS),disable);
   Ok(())
}

#[test]
fn should_recognise_cmp()->Result<(),std::io::Error>{
   let cmp_path = Path::new("assembly_tests/cmp.s");
   let src_code = concat!(
      ".text\n.thumb\nCMP r4,#255\n",
      "CMP r3,r6\n",
      "CMP r2,r9\n"
   );

   let bytes = assemble(
      cmp_path,
      src_code.as_bytes()
   ).unwrap();

   let imm8_bytes: [u8;2] = [bytes[0],bytes[1]];
   let inst_imm8: Opcode = (imm8_bytes).into();
   let cmp_t1_bytes: [u8;2] = [bytes[2],bytes[3]];
   let inst_cmp_t1: Opcode = (cmp_t1_bytes).into();
   let cmp_t2_bytes: [u8;2] = [bytes[4],bytes[5]];
   let inst_cmp_t2: Opcode = (cmp_t2_bytes).into();

   if let Some(Operands::CMP_Imm8(reg,imm8)) = get_operands(&Opcode::_16Bit(B16::CMP_Imm8), imm8_bytes){
      assert_eq!((reg.0,imm8.0),(4,255));
   }else{
      panic!("cannot detect cmp imm8 operands");
   }

   if let Some(Operands::PureRegisterPair(left,right)) = get_operands(&Opcode::_16Bit(B16::CMP_REG_T1), cmp_t1_bytes){
      assert_eq!((left.0,right.0),(3,6));
   }else{
      panic!("cannot detect cmp operands t1");
   }

   if let Some(Operands::PureRegisterPair(left,right)) = get_operands(&Opcode::_16Bit(B16::CMP_REG_T2), cmp_t2_bytes){
      assert_eq!((left.0,right.0),(2,9));
   }else{
      panic!("cannot detect cmp operands t2");
   }

   assert_eq!(Opcode::_16Bit(B16::CMP_Imm8),inst_imm8);
   assert_eq!(Opcode::_16Bit(B16::CMP_REG_T1),inst_cmp_t1);
   assert_eq!(Opcode::_16Bit(B16::CMP_REG_T2),inst_cmp_t2);
   Ok(())
}

#[test]
fn should_recognise_xor()->Result<(),std::io::Error>{
   let path_xor = Path::new("assembly_tests/xor.s");

   let bytes = assemble(
      path_xor,
      b".text\n.thumb\nEOR r7,r3\n"
   ).unwrap();

   let instr: [u8;2] = [bytes[0],bytes[1]];
   let xor: Opcode = (instr).into();
   if let Some(Operands::RegisterPair(left, right)) = get_operands(&Opcode::_16Bit(B16::XOR_REG), instr){
      assert_eq!((left.0,right.0),(7,3));
   }else{
      panic!("cannot detect EOR operands");
   }
   assert_eq!(Opcode::_16Bit(B16::XOR_REG),xor);
   Ok(())
}

#[test]
fn should_recognise_ldm()->Result<(),std::io::Error>{
   use crate::binutils::get_set_bits;
   let path_ldm = Path::new("assembly_tests/ldm.s");
   let src_code = concat!(
      ".text\n.thumb\nLDM r3, {r1,r2,r3}\n",
      "LDM r5!,{r1,r2,r3}\n"
   );

   let bytes = assemble(
      path_ldm, 
      src_code.as_bytes()
   ).unwrap();
   let xmpl_0: [u8;2] = [bytes[0],bytes[1]];
   let xmpl_1: [u8;2] = [bytes[2],bytes[3]];
   let opc_0: Opcode = (xmpl_0).into();
   let opc_1: Opcode = (xmpl_1).into();


   if let Some(Operands::LoadableList(base, list)) = get_operands(&Opcode::_16Bit(B16::LDM), xmpl_0){
      assert_eq!(base.0,3);
      assert_eq!(get_set_bits(list),vec![1,2,3]);
   }else{
      panic!("could not parse ldm instruction");
   }

   if let Some(Operands::LoadableList(base, list)) = get_operands(&Opcode::_16Bit(B16::LDM), xmpl_1){
      assert_eq!(base.0,5);
      assert_eq!(get_set_bits(list),vec![1,2,3]);
   }else{
      panic!("could not parse ldm instruction");
   }

   assert_eq!(Opcode::_16Bit(B16::LDM),opc_0);
   assert_eq!(Opcode::_16Bit(B16::LDM),opc_1);
   Ok(())
}

#[test]
fn should_recognise_ldr()->Result<(),std::io::Error>{
   let path_imm5 = Path::new("assembly_tests/ldr_imm5.s");

   let src_code = concat!(
      ".text\n.thumb\nLDR r0,[r1,#20]\n",
      ".text\n.thumb\nLDR r4,[SP,#124]\n",
      ".text\n.thumb\nLDR r6,[PC,#16]\n",
      ".text\n.thumb\nLDR r1,[r2,r4]\n"
   );

   let bytes = assemble(
      path_imm5,
      src_code.as_bytes()
   ).unwrap();

   let imm5_b = [bytes[0],bytes[1]];
   let code_imm5: Opcode = (imm5_b).into();
   let imm8_b = [bytes[2],bytes[3]];
   let code_imm8: Opcode = (imm8_b).into();
   let pc_imm8_alt_b = [bytes[4],bytes[5]];
   let code_pc_imm8_alt = (pc_imm8_alt_b).into();
   let reg_b = [bytes[6],bytes[7]];
   let code_reg = (reg_b).into();

   if let Some(Operands::LDR_Imm5(dest,base, offset)) = get_operands(&Opcode::_16Bit(B16::LDR_Imm5), imm5_b){
      assert_eq!((dest.0,base.0,offset.0),(0,1,20));
   }else{
      panic!("could not parse ldr operands");
   }

   if let Some(Operands::LDR_Imm8(dest,src ,offset)) = get_operands(&Opcode::_16Bit(B16::LDR_SP_Imm8), imm8_b){
      assert_eq!((dest.0,src.0,offset.0),(4,STACK_POINTER,124));
   }else{
      panic!("could not parse ldr operands");
   }

   if let Some(Operands::LDR_Imm8(dest,src,offset)) = get_operands(&Opcode::_16Bit(B16::LDR_PC_Imm8), pc_imm8_alt_b){
      assert_eq!((dest.0,src.0,offset.0),(6,PROGRAM_COUNTER,16));
   }else{
      panic!("could not parse ldr operands");
   }

   if let Some(Operands::LDR_REG(dest,base,offset)) = get_operands(&Opcode::_16Bit(B16::LDR_REGS), reg_b){
      assert_eq!((dest.0,base.0,offset.0),(1,2,4));
   }else{
      panic!("could not parse ldr operands");
   }

   assert_eq!(Opcode::_16Bit(B16::LDR_Imm5),code_imm5);
   assert_eq!(Opcode::_16Bit(B16::LDR_SP_Imm8),code_imm8);
   assert_eq!(Opcode::_16Bit(B16::LDR_PC_Imm8),code_pc_imm8_alt);
   assert_eq!(Opcode::_16Bit(B16::LDR_REGS),code_reg);
   Ok(())
}

#[test]
fn should_recognise_ldrb()->Result<(),std::io::Error>{
   let path = Path::new("assembly_tests/ldrb.s");
   let src_code = concat!(
      ".text\n.thumb\nLDRB r0,[r4,#24]\n",
      "LDRB r0,[r3,r5]\n"
   );

   let bytes = assemble(
      path,
      src_code.as_bytes()
   ).unwrap();

   let imm5_b = [bytes[0],bytes[1]];
   let reg_b = [bytes[2],bytes[3]];
   let imm5: Opcode = (imm5_b).into();
   let regs: Opcode = (reg_b).into();

   if let Some(Operands::LDR_Imm5(dest,base, offset)) = get_operands(&Opcode::_16Bit(B16::LDRB_Imm5), imm5_b){
      assert_eq!((dest.0,base.0,offset.0),(0,4,24));
   }else{
      panic!("could not parse ldrb operands");
   }

   if let Some(Operands::LDR_REG(dest,base,offset)) = get_operands(&Opcode::_16Bit(B16::LDRB_REGS), reg_b){
      assert_eq!((dest.0,base.0,offset.0),(0,3,5));
   }else{
      panic!("could not parse ldrb operands");
   }

   assert_eq!(Opcode::_16Bit(B16::LDRB_Imm5),imm5);
   assert_eq!(Opcode::_16Bit(B16::LDRB_REGS),regs);
   Ok(())
}

#[test]
fn should_recognise_ldrh()->Result<(),std::io::Error>{
   let path = Path::new("assembly_tests/ldrh.s");
   let src_code = concat!(
      ".text\n.thumb\nLDRH r0,[r1,#58]\n",
      "LDRH r2,[r1,r7]\n"
   );

   let bytes = assemble(
      path,
      src_code.as_bytes()
   ).unwrap();

   let imm5_b = [bytes[0],bytes[1]];
   let reg_b = [bytes[2],bytes[3]];
   let imm5: Opcode = (imm5_b).into();
   let regs: Opcode = (reg_b).into();

   if let Some(Operands::LDR_Imm5(dest,base, offset)) = get_operands(&Opcode::_16Bit(B16::LDRH_Imm5), imm5_b){
      assert_eq!((dest.0,base.0,offset.0),(0,1,58));
   }else{
      panic!("could not parse ldrh operands");
   }

   if let Some(Operands::LDR_REG(dest,base,offset)) = get_operands(&Opcode::_16Bit(B16::LDRH_REGS), reg_b){
      assert_eq!((dest.0,base.0,offset.0),(2,1,7));
   }else{
      panic!("could not parse ldrh operands");
   }

   assert_eq!(Opcode::_16Bit(B16::LDRH_Imm5),imm5);
   assert_eq!(Opcode::_16Bit(B16::LDRH_REGS),regs);
   Ok(())
}

#[test]
fn should_recognise_ldrsb()->Result<(),std::io::Error>{
   let path = Path::new("assembly_tests/ldrsb.s");

   let bytes = assemble(
      path,
      b".text\n.thumb\nLDRSB r0,[r1,r7]\n"
   ).unwrap();

   let reg_b = [bytes[0],bytes[1]];
   let regs: Opcode = (reg_b).into();

   if let Some(Operands::LDR_REG(dest,base,offset)) = get_operands(&Opcode::_16Bit(B16::LDRSB_REGS), reg_b){
      assert_eq!((dest.0,base.0,offset.0),(0,1,7));
   }else{
      panic!("could not parse ldrsb operands");
   }
   assert_eq!(Opcode::_16Bit(B16::LDRSB_REGS),regs);
   Ok(())
}

#[test]
fn should_recognise_ldrsh()->Result<(),std::io::Error>{
   let path = Path::new("assembly_tests/ldrsh.s");

   let bytes = assemble(
      path,
      b".text\n.thumb\nLDRSH r0,[r3,r7]\n"
   ).unwrap();

   let reg_b = [bytes[0],bytes[1]];
   let regs: Opcode = (reg_b).into();

   if let Some(Operands::LDR_REG(dest,base,offset)) = get_operands(&Opcode::_16Bit(B16::LDRSH_REGS), reg_b){
      assert_eq!((dest.0,base.0,offset.0),(0,3,7));
   }else{
      panic!("could not parse ldrsh operands");
   }
   assert_eq!(Opcode::_16Bit(B16::LDRSH_REGS),regs);
   Ok(())
}

#[test]
fn should_recognise_lsl()->Result<(),std::io::Error>{
   let path = Path::new("assembly_tests/lsl.s");
   let src_code = ".text\n.thumb\nLSL r4,r1,#31\nLSL r0,r7";
   let bytes = assemble(
      path,
      src_code.as_bytes()
   ).unwrap();

   let imm5_b = [bytes[0],bytes[1]];
   let imm5: Opcode = (imm5_b).into();
   let reg_b = [bytes[2],bytes[3]];
   let regs: Opcode = (reg_b).into();

   if let Some(Operands::LS_Imm5(dest, src, imm)) = get_operands(&Opcode::_16Bit(B16::LSL_Imm5), imm5_b){
      assert_eq!((dest.0,src.0,imm.0),(4,1,31));
   }else{
      panic!("could not parse lsl");
   }

   if let Some(Operands::RegisterPair(dest, other)) = get_operands(&Opcode::_16Bit(B16::LSL_REGS), reg_b){
      assert_eq!((dest.0,other.0),(0,7));
   }else{
      panic!("could not parse lsl");
   }
   assert_eq!(Opcode::_16Bit(B16::LSL_Imm5),imm5);
   assert_eq!(Opcode::_16Bit(B16::LSL_REGS),regs);
   Ok(())
}

#[test]
fn should_recognise_lsr()->Result<(),std::io::Error>{
   let path = Path::new("assembly_tests/lsr.s");
   let src_code = ".text\n.thumb\nLSR r2,r5,#31\nLSR r0,r7";
   
   let bytes = assemble(
      path,
      src_code.as_bytes()
   ).unwrap();

   let imm5_b = [bytes[0],bytes[1]];
   let reg_b = [bytes[2],bytes[3]];
   let imm5: Opcode = (imm5_b).into();
   let regs: Opcode = (reg_b).into();

   if let Some(Operands::LS_Imm5(dest, src, imm)) = get_operands(&Opcode::_16Bit(B16::LSR_Imm5), imm5_b){
      assert_eq!((dest.0,src.0,imm.0),(2,5,31));
   }else{
      panic!("could not parse lsr");
   }

   if let Some(Operands::RegisterPair(dest, other)) = get_operands(&Opcode::_16Bit(B16::LSR_REGS), reg_b){
      assert_eq!((dest.0,other.0),(0,7));
   }else{
      panic!("could not parse lsr");
   }

   assert_eq!(Opcode::_16Bit(B16::LSR_Imm5),imm5);
   assert_eq!(Opcode::_16Bit(B16::LSR_REGS),regs);
   Ok(())
}

#[test]
fn should_recognise_mov()->Result<(),std::io::Error>{
   let path = Path::new("assembly_tests/mov.s");

   let bytes = assemble_by(
      path,
      b".text\n.thumb\nMOV r0,#7\n",
      asm_file_to_elf_armv6
   ).unwrap();

   let sanity: Opcode = ([bytes[0],bytes[1]]).into();
   assert_eq!(Opcode::_16Bit(B16::MOV_Imm8),sanity);

   let bytes = assemble(
      path,
      b".text\n.syntax unified\n.thumb\nMOVS.N r7,#255\n"
   ).unwrap();
   let imm8: Opcode = ([bytes[0],bytes[1]]).into();
   if let Some(Operands::DestImm8(reg,literal)) = get_operands(&Opcode::_16Bit(B16::MOV_Imm8), [bytes[0],bytes[1]]){
      assert_eq!((reg.0,literal.0),(7,255));
   }else{
      panic!("could not parse mov");
   }

   let bytes = assemble(
      path,
      b".text\n.thumb\nMOV r2,r8\n"
   ).unwrap();
   let regs_t1: Opcode = ([bytes[0],bytes[1]]).into();
   if let Some(Operands::MOV_REG(dest,src)) = get_operands(&Opcode::_16Bit(B16::MOV_REGS_T1), [bytes[0],bytes[1]]){
      assert_eq!((dest.0,src.0),(2,8));
   }else{
      panic!("could not parse mov");
   }

   let bytes = assemble_by(
      path,
      b".text\n.syntax unified\n.thumb\nMOVS.N r4,r7\n",
      asm_file_to_elf_armv6t2
   ).unwrap();
   let regs_t2: Opcode = ([bytes[0],bytes[1]]).into();
   if let Some(Operands::MOV_REG(dest,src)) = get_operands(&Opcode::_16Bit(B16::MOV_REGS_T2), [bytes[0],bytes[1]]){
      assert_eq!((dest.0,src.0),(4,7));
   }else{
      panic!("could not parse mov");
   }


   assert_eq!(Opcode::_16Bit(B16::MOV_Imm8),imm8);
   assert_eq!(Opcode::_16Bit(B16::MOV_REGS_T1),regs_t1);
   assert_eq!(Opcode::_16Bit(B16::MOV_REGS_T2),regs_t2);
   Ok(())
}

#[test]
fn should_recognise_mul()->Result<(),std::io::Error>{
   let path = Path::new("assembly_tests/mul.s");

   let bytes = assemble(path, b".text\n.thumb\nMUL r4,r1\n").unwrap();
   let regs: Opcode = ([bytes[0],bytes[1]]).into();
   if let Some(Operands::RegisterPair(dest,other)) = get_operands(&Opcode::_16Bit(B16::MUL),[bytes[0],bytes[1]]){
      assert_eq!((dest.0,other.0),(4,1));
   }else{
      panic!("could not parse MUL");
   }
   
   assert_eq!(Opcode::_16Bit(B16::MUL),regs);
   Ok(())
}

#[test]
fn should_recognise_mvn()->Result<(),std::io::Error>{
   let path = Path::new("assembly_tests/mvn.s");

   let bytes = assemble(path, b".text\n.thumb\nMVN r1,r3\n").unwrap();
   let regs: Opcode = ([bytes[0],bytes[1]]).into();
   if let Some(Operands::MOV_REG(dest,src)) = get_operands(&Opcode::_16Bit(B16::MVN),[bytes[0],bytes[1]]){
      assert_eq!((dest.0,src.0),(1,3));
   }else{
      panic!("could not parse MVN");
   }

   assert_eq!(Opcode::_16Bit(B16::MVN),regs);
   Ok(())
}

#[test]
fn should_recognise_nop()->Result<(),std::io::Error>{
   let path = Path::new("assembly_tests/nop.s");
   
   let bytes = assemble_by(
      path,
      b".text\n.syntax unified\n.thumb\nNOP\n",
      asm_file_to_elf_armv6t2
   ).unwrap();
   let encoding_t2: Opcode = ([bytes[0],bytes[1]]).into();

   assert_eq!(Opcode::_16Bit(B16::NOP),encoding_t2);
   assert!(get_operands(&Opcode::_16Bit(B16::NOP),[bytes[0],bytes[1]]).is_none());
   Ok(())
}

#[test]
fn should_recognise_orr()->Result<(),std::io::Error>{
   let path = Path::new("assembly_tests/orr.s");

   let bytes = assemble(path, b".text\n.thumb\nORR r5,r1\n").unwrap();
   let t1: Opcode = ([bytes[0],bytes[1]]).into();
   if let Some(Operands::RegisterPair(dest,other)) = get_operands(&Opcode::_16Bit(B16::ORR), [bytes[0],bytes[1]]){
      assert_eq!((dest.0,other.0),(5,1));
   }else{
      panic!("could not parse orr");
   }

   assert_eq!(Opcode::_16Bit(B16::ORR),t1);
   Ok(())
}

#[test]
fn should_recognise_pop()->Result<(),std::io::Error>{
   let path = Path::new("assembly_tests/pop.s");

   let bytes = assemble(path, b".text\n.thumb\nPOP {r0,r1,r4,r7}\n").unwrap();
   let t1: Opcode = ([bytes[0],bytes[1]]).into();
   if let Some(Operands::RegisterList(list)) = get_operands(&Opcode::_16Bit(B16::POP), [bytes[0],bytes[1]]){
      assert_eq!(get_set_bits(list),vec![0,1,4,7]);
   }else{
      panic!("could not parse POP");
   }

   let bytes = assemble(path, b".text\n.thumb\nPOP {r1,PC}\n").unwrap();
   let sc: Opcode = ([bytes[0],bytes[1]]).into();
   if let Some(Operands::RegisterList(list)) = get_operands(&Opcode::_16Bit(B16::POP), [bytes[0],bytes[1]]){
      assert_eq!(get_set_bits(list),vec![1,15]);
   }else{
      panic!("could not parse POP");
   }

   assert_eq!(Opcode::_16Bit(B16::POP),t1);
   assert_eq!(Opcode::_16Bit(B16::POP),sc);
   Ok(())
}

#[test]
fn should_recognise_push()->Result<(),std::io::Error>{
   let path = Path::new("assembly_tests/push.s");

   let bytes = assemble(path, b".text\n.thumb\nPUSH {r2,r1,LR}\n").unwrap();
   let t1: Opcode = ([bytes[0],bytes[1]]).into();
   if let Some(Operands::RegisterList(list)) = get_operands(&Opcode::_16Bit(B16::PUSH),[bytes[0],bytes[1]]){
      assert_eq!(get_set_bits(list),vec![1,2,14])
   }

   assert_eq!(Opcode::_16Bit(B16::PUSH),t1);
   Ok(())
}

#[test]
fn should_recognise_rev()->Result<(),std::io::Error>{
   let path = Path::new("assembly_tests/rev.s");

   let bytes = assemble(path, b".text\n.thumb\nREV r0,r1\n").unwrap();
   let t1: Opcode = ([bytes[0],bytes[1]]).into();
   if let Some(Operands::RegisterPair(dest,other)) = get_operands(&Opcode::_16Bit(B16::REV), [bytes[0],bytes[1]]){
      assert_eq!((dest.0,other.0),(0,1));
   }else{
      panic!("could not parse REV");
   }

   let bytes = assemble(path, b".text\n.thumb\nREV16 r5,r6\n").unwrap();
   let c16: Opcode = ([bytes[0],bytes[1]]).into();
   if let Some(Operands::RegisterPair(dest,other)) = get_operands(&Opcode::_16Bit(B16::REV_16), [bytes[0],bytes[1]]){
      assert_eq!((dest.0,other.0),(5,6));
   }else{
      panic!("could not parse REV16");
   }


   let bytes = assemble(path, b".text\n.thumb\nREVSH r3,r2\n").unwrap();
   let sh: Opcode = ([bytes[0],bytes[1]]).into();
   if let Some(Operands::RegisterPair(dest,other)) = get_operands(&Opcode::_16Bit(B16::REVSH), [bytes[0],bytes[1]]){
      assert_eq!((dest.0,other.0),(3,2));
   }else{
      panic!("could not parse REVSH");
   }

   assert_eq!(Opcode::_16Bit(B16::REV),t1);
   assert_eq!(Opcode::_16Bit(B16::REV_16),c16);
   assert_eq!(Opcode::_16Bit(B16::REVSH),sh);
   Ok(())
}

#[test]
fn should_recognise_ror()->Result<(),std::io::Error>{
   let path = Path::new("assembly_tests/ror.s");

   let bytes = assemble(path, b".text\n.thumb\nROR r0,r7\n").unwrap();
   let t1: Opcode = ([bytes[0],bytes[1]]).into();
   if let Some(Operands::RegisterPair(dest,other)) = get_operands(&Opcode::_16Bit(B16::ROR), [bytes[0],bytes[1]]){
      assert_eq!((dest.0,other.0),(0,7));
   }else{
      panic!("could not parse ROR");
   }

   assert_eq!(Opcode::_16Bit(B16::ROR),t1);
   Ok(())
}

#[test]
fn should_recognise_rsb()->Result<(),std::io::Error>{
   let path = Path::new("assembly_tests/rsb.s");

   let bytes = assemble_by(
      path,
      b".text\n.thumb\nNEG r3,r1\n",// idk why gnu as doesn't assemble RSB D: NEG is the pre-UAL version of RSB tho:D
      // gnu as .. more like gnu ass
      asm_file_to_elf_armv6t2
   ).unwrap();
   let t1: Opcode = ([bytes[0],bytes[1]]).into();
   if let Some(Operands::RegisterPair(dest, other)) = get_operands(&Opcode::_16Bit(B16::RSB), [bytes[0],bytes[1]]){
      assert_eq!((dest.0,other.0),(3,1));
   }else{
      panic!("could not parse RSB");
   }

   assert_eq!(Opcode::_16Bit(B16::RSB),t1);
   Ok(())
}

#[test]
fn should_recognise_sbc()->Result<(),std::io::Error>{
   let path = Path::new("assembly_tests/sbc.s");

   let bytes = assemble(
      path,
      b".text\n.thumb\nSBC r2,r5\n"
   ).unwrap();
   let t1: Opcode = ([bytes[0],bytes[1]]).into();
   if let Some(Operands::RegisterPair(dest, other)) = get_operands(&Opcode::_16Bit(B16::SBC), [bytes[0],bytes[1]]){
      assert_eq!((dest.0,other.0),(2,5));
   }else{
      panic!("could not parse SBC");
   }

   assert_eq!(Opcode::_16Bit(B16::SBC),t1);
   Ok(())
}

#[test]
fn should_recognise_sev()->Result<(),std::io::Error>{
   let path = Path::new("assembly_tests/sev.s");

   let bytes = assemble_by(
      path,
      b".text\n.thumb\nSEV\n",
      asm_file_to_elf_armv6t2
   ).unwrap();
   let t1: Opcode = ([bytes[0],bytes[1]]).into();

   assert_eq!(Opcode::_16Bit(B16::SEV),t1);
   assert!(get_operands(&Opcode::_16Bit(B16::SEV), [bytes[0],bytes[1]]).is_none());
   Ok(())
}

#[test]
fn should_recognise_stm()->Result<(),std::io::Error>{
   let path = Path::new("assembly_tests/stm.s");

   let bytes = assemble(
      path,
      b".text\n.thumb\nSTM r5!,{r1,r3,r7}\n"
   ).unwrap();
   let t1: Opcode = ([bytes[0],bytes[1]]).into();
   if let Some(Operands::LoadableList(base, list)) = get_operands(&Opcode::_16Bit(B16::STM), [bytes[0],bytes[1]]){
      assert_eq!((base.0,get_set_bits(list)),(5,vec![1,3,7]));
   }else{
      panic!("could not parse STM");
   }

   assert_eq!(Opcode::_16Bit(B16::STM),t1);
   Ok(())
}

#[test]
fn should_recognise_str()->Result<(),std::io::Error>{
   let path = Path::new("assembly_tests/str.s");
   let src_code = concat!(
      ".text\n.thumb\n",
      "STR r0, [r1,#12]\n",
      "STR r0, [SP,#224]\n",
      "STR r5, [r1,r2]\n"
   );

   let bytes = assemble( path, src_code.as_bytes()).unwrap();
   let t1: Opcode = ([bytes[0],bytes[1]]).into();
   let t2: Opcode = ([bytes[2],bytes[3]]).into();
   let reg: Opcode = ([bytes[4],bytes[5]]).into();
   let t1_b = [bytes[0],bytes[1]];
   let t2_b = [bytes[2],bytes[3]];
   let reg_b = [bytes[4],bytes[5]];
   if let Some(Operands::STR_Imm5(src, base,literal)) = get_operands(&Opcode::_16Bit(B16::STR_Imm5), t1_b){
      assert_eq!((src.0,base.0,literal.0),(0,1,12));
   }else{
      panic!("could not parse STR");
   }

   if let Some(Operands::STR_Imm8(src, literal)) = get_operands(&Opcode::_16Bit(B16::STR_Imm8), t2_b){
      assert_eq!((src.0,literal.0),(0,224));
   }else{
      panic!("could not parse STR");
   }

   if let Some(Operands::STR_REG(src, base,offset)) = get_operands(&Opcode::_16Bit(B16::STR_REG), reg_b){
      assert_eq!((src.0,base.0,offset.0),(5,1,2));
   }else{
      panic!("could not parse STR");
   }

   assert_eq!(Opcode::_16Bit(B16::STR_Imm5),t1);
   assert_eq!(Opcode::_16Bit(B16::STR_Imm8),t2);
   assert_eq!(Opcode::_16Bit(B16::STR_REG),reg);
   Ok(())
}

#[test]
fn should_recognise_strb()->Result<(),std::io::Error>{
   let path = Path::new("assembly_tests/strb.s");
   
   let bytes = assemble(path, b".text\n.thumb\nSTRB r2, [r1,#17]\n").unwrap();
   let im: Opcode = ([bytes[0],bytes[1]]).into();
   if let Some(Operands::STR_Imm5(src,base,literal)) = get_operands(&Opcode::_16Bit(B16::STRB_Imm5), [bytes[0],bytes[1]]){
      assert_eq!((src.0,base.0,literal.0),(2,1,17));
   }else{
      panic!("could not parse STRB");
   }

   let bytes = assemble(path, b".text\n.thumb\nSTRB r3, [r1,r6]\n").unwrap();
   let reg: Opcode = ([bytes[0],bytes[1]]).into();
   if let Some(Operands::STR_REG(src,base,offset)) = get_operands(&Opcode::_16Bit(B16::STRB_REG), [bytes[0],bytes[1]]){
      assert_eq!((src.0,base.0,offset.0),(3,1,6));
   }else{
      panic!("could not parse STRB");
   }

   assert_eq!(Opcode::_16Bit(B16::STRB_Imm5),im);
   assert_eq!(Opcode::_16Bit(B16::STRB_REG),reg);
   Ok(())
}

#[test]
fn should_recognise_strh()->Result<(),std::io::Error>{
   let path = Path::new("assembly_tests/strh.s");
   
   let bytes = assemble(path, b".text\n.thumb\nstrh r4, [r3,#62]\n").unwrap();
   let im: Opcode = ([bytes[0],bytes[1]]).into();
   println!("is {:?}",im);
   if let Some(Operands::STR_Imm5(src,base,literal)) = get_operands(&Opcode::_16Bit(B16::STRH_Imm5), [bytes[0],bytes[1]]){
      assert_eq!((src.0,base.0,literal.0),(4,3,62));
   }else{
      panic!("could not parse STRH");
   }

   let bytes = assemble(path, b".text\n.thumb\nstrh r5, [r1,r2]\n").unwrap();
   let reg: Opcode = ([bytes[0],bytes[1]]).into();
   if let Some(Operands::STR_REG(src,base,offset)) = get_operands(&Opcode::_16Bit(B16::STRH_REG), [bytes[0],bytes[1]]){
      assert_eq!((src.0,base.0,offset.0),(5,1,2));
   }else{
      panic!("could not parse STRH");
   }

   assert_eq!(Opcode::_16Bit(B16::STRH_Imm5),im);
   assert_eq!(Opcode::_16Bit(B16::STRH_REG),reg);
   Ok(())
}

#[test]
fn should_recognise_sub()->Result<(),std::io::Error>{
   let path = Path::new("assembly_tests/sub.s");

   let src_code = concat!(
      ".text\n.thumb\nSUB r3,r4,#5\n",
      "SUB r5,#240\n",
      "SUB r0,r1,r3\n",
      "SUB sp,sp,#16\n"
   );
   let bytes = assemble(path, src_code.as_bytes()).unwrap();
   let im3: Opcode = ([bytes[0],bytes[1]]).into();
   if let Some(Operands::RegPairImm3(dest,src,literal)) = get_operands(&Opcode::_16Bit(B16::SUB_Imm3),[bytes[0],bytes[1]]){
      assert_eq!((dest.0,src.0,literal.0),(3,4,5));
   }else{
      panic!("could not parse SUB");
   }

   let im8: Opcode = ([bytes[2],bytes[3]]).into();
   if let Some(Operands::DestImm8(dest,literal)) = get_operands(&Opcode::_16Bit(B16::SUB_Imm8), [bytes[2],bytes[3]]){
      assert_eq!((dest.0,literal.0),(5,240));
   }else{
      panic!("could not parse SUB");
   }

   let reg: Opcode = ([bytes[4],bytes[5]]).into();
   if let Some(Operands::RegisterTriplet(dest,src,other)) = get_operands(&Opcode::_16Bit(B16::SUB_REG), [bytes[4],bytes[5]]){
      assert_eq!((dest.0,src.0,other.0),(0,1,3));
   }else{
      panic!("could not parse SUB");
   }

   let sp: Opcode = ([bytes[6],bytes[7]]).into();
   if let Some(Operands::SP_SUB(literal)) = get_operands(&Opcode::_16Bit(B16::SUB_SP_Imm7), [bytes[6],bytes[7]]){
      assert_eq!((literal.0),(16));
   }else{
      panic!("could not parse SUB");
   }

   assert_eq!(Opcode::_16Bit(B16::SUB_Imm3),im3);
   assert_eq!(Opcode::_16Bit(B16::SUB_Imm8),im8);
   assert_eq!(Opcode::_16Bit(B16::SUB_REG),reg);
   assert_eq!(Opcode::_16Bit(B16::SUB_SP_Imm7),sp);
   Ok(())
}

#[test]
fn should_recognise_svc()->Result<(),std::io::Error>{
   let path = Path::new("assembly_tests/svc.s");

   let bytes = assemble(path, b".text\n.thumb\nSVC #15\n").unwrap();
   let svc: Opcode = ([bytes[0],bytes[1]]).into();
   if let Some(Operands::Byte(literal)) = get_operands(&Opcode::_16Bit(B16::SVC), [bytes[0],bytes[1]]){
      assert_eq!(literal.0,15);
   }else{
      panic!("could not parse SVC");
   }

   assert_eq!(Opcode::_16Bit(B16::SVC),svc);
   Ok(())
}

#[test]
fn should_recognise_sxt()->Result<(),std::io::Error>{
   let path = Path::new("assembly_tests/sxt_.s");

   let bytes = assemble(path, b".text\n.thumb\n SXTB r0,r1\n").unwrap();
   let sxtb: Opcode = ([bytes[0],bytes[1]]).into();
   if let Some(Operands::RegisterPair(dest,other)) = get_operands(&Opcode::_16Bit(B16::SXTB), [bytes[0],bytes[1]]){
      assert_eq!((dest.0,other.0),(0,1));
   }else{
      panic!("could not parse SXTB");
   }

   let bytes = assemble(path, b".text\n.thumb\n SXTH r6,r1\n").unwrap();
   let sxth: Opcode = ([bytes[0],bytes[1]]).into();
   if let Some(Operands::RegisterPair(dest,other)) = get_operands(&Opcode::_16Bit(B16::SXTH), [bytes[0],bytes[1]]){
      assert_eq!((dest.0,other.0),(6,1));
   }else{
      panic!("could not parse SXTH");
   }

   assert_eq!(Opcode::_16Bit(B16::SXTB),sxtb);
   assert_eq!(Opcode::_16Bit(B16::SXTH),sxth);
   Ok(())
}

#[test]
fn should_recognise_tst()->Result<(),std::io::Error>{
   let path = Path::new("assembly_tests/tst.s");

   let bytes = assemble(path, b".text\n.thumb\n TST r4,r5\n").unwrap();
   let t1: Opcode = ([bytes[0],bytes[1]]).into();
   if let Some(Operands::PureRegisterPair(a,b)) = get_operands(&Opcode::_16Bit(B16::TST), [bytes[0],bytes[1]]){
      assert_eq!((a.0,b.0),(4,5));
   }else{
      panic!("could not parse TST");
   }

   assert_eq!(Opcode::_16Bit(B16::TST),t1);
   Ok(())
}

#[test]
fn should_recognise_udfw()->Result<(),std::io::Error>{
   let path = Path::new("assembly_tests/udfw.s");

   let t1 = assemble_and_decode_32b(
      path,
      b".text\n.thumb\n.syntax unified\n UDF.W #101\n",
      asm_file_to_elf_armv6t2
   ).unwrap();

   println!("raw {}",t1);
   assert_eq!(Opcode::_32Bit(B32::UNDEFINED),t1);
   Ok(())
}

#[test] 
fn should_recognise_uxt()->Result<(),std::io::Error>{
   let path = Path::new("assembly_tests/uxt.s");

   let bytes = assemble_by(
      path,
      b".text\n.thumb\nUXTB r4,r1\n", 
      asm_file_to_elf_armv6t2
   ).unwrap();
   let uxtb: Opcode = ([bytes[0],bytes[1]]).into();
   if let Some(Operands::RegisterPair(a,b)) = get_operands(&Opcode::_16Bit(B16::UXTB), [bytes[0],bytes[1]]){
      assert_eq!((a.0,b.0),(4,1));
   }else{
      panic!("could not parse UXTB");
   }

   let bytes = assemble_by(
      path,
      b".text\n.thumb\nUXTH r5,r7\n", 
      asm_file_to_elf_armv6t2
   ).unwrap();
   let uxth: Opcode = ([bytes[0],bytes[1]]).into();
   if let Some(Operands::RegisterPair(a,b)) = get_operands(&Opcode::_16Bit(B16::UXTH), [bytes[0],bytes[1]]){
      assert_eq!((a.0,b.0),(5,7));
   }else{
      panic!("could not parse UXTH");
   }

   assert_eq!(Opcode::_16Bit(B16::UXTB),uxtb);
   assert_eq!(Opcode::_16Bit(B16::UXTH),uxth);
   Ok(())
}

#[test]
fn should_recognise_wf()->Result<(),std::io::Error>{
   let path = Path::new("assembly_tests/wf.s");

   let bytes = assemble_by(
      path,
      b".thumb\n.text\nWFE\nWFI\n",
      asm_file_to_elf_armv6t2
   ).unwrap();
   let wfe: Opcode = ([bytes[0],bytes[1]]).into();
   let wfi: Opcode = ([bytes[2],bytes[3]]).into();


   assert_eq!(Opcode::_16Bit(B16::WFE),wfe);
   assert!(get_operands(&Opcode::_16Bit(B16::WFE), [bytes[0],bytes[1]]).is_none());
   assert_eq!(Opcode::_16Bit(B16::WFI),wfi);
   assert!(get_operands(&Opcode::_16Bit(B16::WFI), [bytes[2],bytes[3]]).is_none());
   Ok(())
}

#[test]
fn should_recognise_yield()->Result<(),std::io::Error>{
   let path = Path::new("assembly_tests/yield.s");

   let yield_opc = assemble_and_decode(
      path,
      b".thumb\n.text\nYIELD\n",
      asm_file_to_elf_armv6t2
   ).unwrap();

   assert_eq!(Opcode::_16Bit(B16::YIELD),yield_opc);
   Ok(())
}

#[test]
fn should_recognise_dmb()->Result<(),std::io::Error>{
   let path = Path::new("assembly_tests/dmb.s");

   let bytes = assemble_by_32b(
      path,
      b".text\nDMB #12\n",
      asm_file_to_elf_armv6
   ).unwrap();

   let instr = [bytes[0],bytes[1],bytes[2],bytes[3]];
   assert_eq!(Opcode::_32Bit(B32::DMB),instr.into());
   if let Some(Operands::Nibble(opt)) = get_operands_32b(&Opcode::_32Bit(B32::DMB), instr){
      assert_eq!(opt.0,12);
   }else{
      panic!("could not decode dmb operands");
   }

   Ok(())
}


#[test]
fn should_recognise_dsb()->Result<(),std::io::Error>{
   let path = Path::new("assembly_tests/dsb.s");

   let bytes = assemble_by_32b(
      path,
      b".text\nDSB #12\n",
      asm_file_to_elf_armv6
   ).unwrap();

   let instr = [bytes[0],bytes[1],bytes[2],bytes[3]];
   assert_eq!(Opcode::_32Bit(B32::DSB),instr.into());
   if let Some(Operands::Nibble(opt)) = get_operands_32b(&Opcode::_32Bit(B32::DSB), instr){
      assert_eq!(opt.0,12);
   }else{
      panic!("could not decode dsb operands");
   }

   Ok(())
}

#[test]
fn should_recognise_isb()->Result<(),std::io::Error>{
   let path = Path::new("assembly_tests/isb.s");

   let bytes = assemble_by_32b(
      path,
      b".text\nISB #13\n",
      asm_file_to_elf_armv6
   ).unwrap();

   let instr = [bytes[0],bytes[1],bytes[2],bytes[3]];
   assert_eq!(Opcode::_32Bit(B32::ISB),instr.into());
   if let Some(Operands::Nibble(opt)) = get_operands_32b(&Opcode::_32Bit(B32::ISB), instr){
      assert_eq!(opt.0,13);
   }else{
      panic!("could not decode isb operands");
   }

   Ok(())
}

#[test]
fn should_recognise_mrs()->Result<(),std::io::Error>{
   let path = Path::new("assembly_tests/mrs.s");

   let bytes = assemble_by_32b(
      path,
      b".text\n.thumb\nMRS R1, APSR\n",
      asm_file_to_elf_armv6t2
   ).unwrap();

   let instr = [bytes[0],bytes[1],bytes[2],bytes[3]];
   assert_eq!(Opcode::_32Bit(B32::MRS),instr.into());
   if let Some(Operands::MRS(reg,spc)) = get_operands_32b(&Opcode::_32Bit(B32::MRS), instr){
      assert_eq!((reg.0, spc),(1,SpecialRegister::APSR));
   }else{
      panic!("could not decode isb operands");
   }

   Ok(())
}

#[test]
fn should_recognise_msr()->Result<(),std::io::Error>{
   let path = Path::new("assembly_tests/msr.s");

   let bytes = assemble_by_32b(
      path,
      b".text\n.thumb\nMSR APSR, R7\n",
      asm_file_to_elf_armv6t2
   ).unwrap();

   let instr = [bytes[0],bytes[1],bytes[2],bytes[3]];
   assert_eq!(Opcode::_32Bit(B32::MSR),instr.into());
   if let Some(Operands::MSR(spc,reg)) = get_operands_32b(&Opcode::_32Bit(B32::MSR), instr){
      assert_eq!((spc,reg.0),(SpecialRegister::APSR,7));
   }else{
      panic!("could not decode isb operands");
   }

   Ok(())
}

#[test]
fn should_recognise_instruction_size()->Result<(),std::io::Error>{
   let path = Path::new("assembly_tests/instruction_size.s");

   let bytes = assemble_by_32b(
      path,
      concat!(
         ".text\n",
         ".thumb\n",
         "MSR APSR, r7\n",
         "ADD r0, #7\n"
         ).as_bytes(),
      asm_file_to_elf_armv6t2
   ).unwrap();

   let instr_32 = [bytes[0], bytes[1]];
   let instr_16 = [bytes[4],bytes[5]];

   assert_eq!(instruction_size(instr_32), InstructionSize::B32);
   assert_eq!(instruction_size(instr_16), InstructionSize::B16);
   Ok(())
}
