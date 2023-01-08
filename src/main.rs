mod asm;
mod elf;
mod dwarf;
mod system;
mod binutils;

#[cfg(test)]
mod tests;

use crate::binutils::{get_bit,clear_bit};
use std::path::{Path,PathBuf};
use std::fs::File;
use std::io::Write;
use elf::decoder::ElfError;

use crate::asm::interpreter::print_assembly;
fn main() {
    assert_eq!(get_bit(1,3),1);
    assert_eq!(get_bit(0,3),1);
    assert_eq!(get_bit(2,3),0);

    println!("{:04b}",10);
    assert_eq!(clear_bit(1, 10),8);

   let path = Path::new("assembly_tests/adc.s");
   let src_code = concat!(
      ".text\n.thumb\n",
      "ADC r0,r1\n",
      "LDM r7!, {r1,r2,r3}\n",
      "LDR r0,[r1,#20]\n",
      "BAL _label\n",
      "_label:\n",
      "SVC #240\n"
   );

   let bytes = assemble(&path,src_code.as_bytes()).unwrap();

   print_assembly(&bytes[..]);
}

fn assemble(path: &Path, asm: &[u8])->Result<Vec<u8>,ElfError>{
   write_asm(path,asm)?;
   let elf = asm_file_to_elf(path)?;
   load_instruction_opcodes(&elf)
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

