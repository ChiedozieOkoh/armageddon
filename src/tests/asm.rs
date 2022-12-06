use crate::asm::decode::Opcode;
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

#[test]
pub fn should_recognise_instructions()-> Result<(),std::io::Error>{
   let path = Path::new("assembly_tests/adc.s");
   write_asm(path, b".text\n\t.thumb\nADC r0, r1\n")?;
   let elf = asm_file_to_elf(path)?;
   let opcodes = load_instruction_opcodes(&elf).unwrap();
   let mut first_inst: [u8;2] = [0;2];
   first_inst[0] = opcodes[0];
   first_inst[1] = opcodes[1];

   let instr: Opcode = (&first_inst).into();

   println!("opcode raw bin{:?}",opcodes);
   std::fs::remove_file(path)?;
   std::fs::remove_file(elf)?;
   assert_eq!(Opcode::ADCS,instr);

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
   assert_eq!(Opcode::ADDI,instr);
   assert_eq!(Opcode::ADDI,secnd);
   assert_eq!(Opcode::ADDI8,third);
   assert_eq!(Opcode::ADDI8,fourth);

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

   /*let mut sec_instr: [u8;2] = [0;2];
   sec_instr[0] = opcodes[2];
   sec_instr[1] = opcodes[3];
   */ 

   let first: Opcode = (&first_instr).into();
   //let second: Opcode = (&sec_instr).into();

   println!("opcode raw bin{:?}",opcodes);
   std::fs::remove_file(path)?;
   std::fs::remove_file(elf)?;
   assert_eq!(Opcode::ADDS_REG,first);
   //assert_eq!(Opcode::ADDS_REG_T2,second); idk why gnu as keeps using T1 encoding
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
   write_asm(path,b".text\n.thumb\nADR r0,_some_lbl\nNOP\n_some_lbl:")?;
   let elf = asm_file_to_elf(path)?;
   let opcodes = load_instruction_opcodes(&elf).unwrap();
   let mut first_instr: [u8;2] = [0;2];
   first_instr[0] = opcodes[0];
   first_instr[1] = opcodes[1];

   let first: Opcode = (&first_instr).into();

   std::fs::remove_file(path)?;
   std::fs::remove_file(elf)?;
   assert_eq!(Opcode::ADR,first);
   Ok(())
}

#[test]
pub fn should_recognise_adr_with_alternate_syntax()->Result<(),std::io::Error>{
   let path = Path::new("assembly_tests/adr_alt.s");
   write_asm(path,b".text\n.thumb\nADD r0,PC,#8\n")?;
   let elf = asm_file_to_elf(path)?;
   let opcodes = load_instruction_opcodes(&elf).unwrap();
   let mut first_instr: [u8;2] = [0;2];
   first_instr[0] = opcodes[0];
   first_instr[1] = opcodes[1];

   let first: Opcode = (&first_instr).into();

   std::fs::remove_file(path)?;
   std::fs::remove_file(elf)?;
   assert_eq!(Opcode::ADR,first);
   Ok(())

}
