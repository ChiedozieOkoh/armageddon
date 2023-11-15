mod asm;
mod elf;
mod dwarf;
mod system;
mod binutils;
mod log;

#[cfg(test)]
mod tests;

use std::collections::HashMap;
use std::path::{Path,PathBuf};
use std::fs::File;
use std::io::Write;
use elf::decoder::ElfError;

use crate::asm::interpreter::print_assembly;
use crate::elf::decoder::{get_string_table_section_hdr, is_symbol_table_section_hdr, get_section_symbols, get_text_section_symbols, get_matching_symbol_names,  build_symbol_byte_offset_map, get_entry_point_offset};
fn main() {
   let args: Vec<String> = std::env::args().collect();
   if args.len() != 2{
      dbg_ln!("you must provide one elf file");
      std::process::exit(-1);
   }

   dbg_ln!("DEBUG ENABLED");
   let maybe_file = Path::new(&args[1]);
   let maybe_instructions  = load_instruction_opcodes(maybe_file);
   exit_on_err(&maybe_instructions);

   let (instructions, entry_point, symbol_map) = maybe_instructions.unwrap();
   print_assembly(&instructions[..],entry_point, &symbol_map);
}

/*fn assemble(path: &Path, asm: &[u8])->Result<Vec<u8>,ElfError>{
   write_asm(path,asm)?;
   let elf = asm_file_to_elf(path)?;
   load_instruction_opcodes(&elf)
}*/
/*
fn write_asm(path: &Path, data: &[u8])->Result<File,std::io::Error>{
   dbg_ln!("writing  asm to {:?}",path);
   let mut file = File::create(path)?;
   file.write_all(data)?;
   dbg_ln!("written asm to {:?}",file);
   Ok(file)
}

fn asm_file_to_elf(path: &Path)->Result<PathBuf,std::io::Error>{
   use std::process::Command;
   let mut fname = String::new();
   fname.push_str(path.to_str().unwrap());
   fname = fname.replace(".s", "");
   fname.push_str(".elf");
   dbg_ln!("writing to {:?}",fname);
   let ret = PathBuf::from(fname.clone());
   let cmd = Command::new("arm-none-eabi-as")
      .arg(path.to_str().unwrap())
      .arg("-o")
      .arg(fname)
      .output()
      .expect("could not link");

   dbg_ln!("=======\n{:?}\n=======",std::str::from_utf8(&cmd.stderr[..]).unwrap());
   Ok(ret)
}
*/

fn load_instruction_opcodes(file: &Path)->Result<(Vec<u8>, usize, HashMap<usize, String>),ElfError>{
   use crate::elf::decoder::{
      SectionHeader,
      get_header,
      get_all_section_headers,
      is_text_section_hdr,
      read_text_section
   };
   let (elf_header,mut reader) = get_header(file)?;

   let section_headers = get_all_section_headers(&mut reader, &elf_header)?;
   dbg_ln!("sect_hdrs {:?}",section_headers);
   assert!(!section_headers.is_empty());

   let text_sect_hdr: Vec<&SectionHeader> = section_headers.iter()
      .filter(|hdr| is_text_section_hdr(&elf_header, hdr))
      .collect();

   dbg_ln!("header {:?}",text_sect_hdr);
   assert_eq!(text_sect_hdr.len(),1);
   let sect_hdr = &text_sect_hdr[0];

   let text_section = read_text_section(&mut reader, &elf_header, sect_hdr)?;

   let strtab_idx = get_string_table_section_hdr(&elf_header, &section_headers).unwrap();
   let str_table_hdr = &section_headers[strtab_idx];

   let maybe_symtab: Vec<&SectionHeader> = section_headers.iter()
      .filter(|hdr| is_symbol_table_section_hdr(&elf_header, hdr))
      .collect();

   let sym_entries = get_section_symbols(&mut reader, &elf_header, &maybe_symtab[0]).unwrap();
   let text_section_symbols = get_text_section_symbols(&elf_header, &section_headers, &sym_entries).unwrap();
   let names = get_matching_symbol_names(&mut reader, &elf_header, &text_section_symbols, &str_table_hdr).unwrap();
   let text_sect_offset_map = build_symbol_byte_offset_map(&elf_header, names, &sym_entries);
   let entry_point = get_entry_point_offset(&elf_header);

   Ok((text_section, entry_point, text_sect_offset_map))
}

fn exit_on_err<T>(maybe_err: &Result<T,ElfError>){
   match maybe_err{
      Err(e) => {println!("{}",e); std::process::exit(-1);},
      Ok(_) => {}
   }
}
