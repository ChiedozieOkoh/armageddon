use std::path::{Path,PathBuf};
use std::fs::File;
use std::io::Write;
use crate::elf::decoder::{
   get_header,
   get_all_section_headers,
   is_text_section_hdr,
   SectionHeader,
   read_text_section, is_symbol_table_section_hdr, get_local_symbols, get_string_table_section_hdr, get_matching_symbol_names
};

fn write_asm_make_elf(path: &Path, data: &[u8])->Result<PathBuf, std::io::Error>{
   let mut file = File::create(path)?;
   file.write_all(data)?;
   println!("wrote to {:?}", file);
   println!("forcing T2 encoding");
   use std::process::Command;
   let mut fname = String::new();
   fname.push_str(path.to_str().unwrap());
   fname = fname.replace(".s", ".elf");
   println!("writing to {:?}",fname);
   let ret = PathBuf::from(fname.clone());
   Command::new("arm-none-eabi-as")
      .arg(path.to_str().unwrap())
      .arg("-march=armv6t2")
      .arg("-o")
      .arg(fname)
      .status()
      .expect("could not link");

   assert!(ret.exists());
   Ok(ret)
}

#[test]
fn should_get_header(){
   let file = Path::new("./assembly_tests/add.o");
   let (header,_reader) = get_header(&file).unwrap();
   println!("header {:?}", header);
}

#[test]
fn should_get_all_section_headers(){
   let file = Path::new("./assembly_tests/add.o");
   let (elf_header,mut reader) = get_header(&file).unwrap();

   let section_headers = get_all_section_headers(&mut reader, &elf_header).unwrap();
   assert!(!section_headers.is_empty());
}

#[test]
fn should_get_text_section_hdr(){
   let file = Path::new("./assembly_tests/add.o");
   let (elf_header,mut reader) = get_header(&file).unwrap();

   let section_headers = get_all_section_headers(&mut reader, &elf_header).unwrap();
   println!("sect_hdrs {:?}",section_headers);
   assert!(!section_headers.is_empty());

   let text_sect_hdr: Vec<SectionHeader> = section_headers.into_iter()
      .filter(|hdr| is_text_section_hdr(&elf_header, hdr))
      .collect();

   println!("header {:?}",text_sect_hdr);
   assert_eq!(text_sect_hdr.len(),1);
}


#[test]
fn should_read_text_section(){
   let file = Path::new("./assembly_tests/add.o");
   let (elf_header,mut reader) = get_header(&file).unwrap();

   let section_headers = get_all_section_headers(&mut reader, &elf_header).unwrap();
   println!("sect_hdrs {:?}",section_headers);
   assert!(!section_headers.is_empty());

   let text_sect_hdr: Vec<SectionHeader> = section_headers.into_iter()
      .filter(|hdr| is_text_section_hdr(&elf_header, hdr))
      .collect();

   println!("header {:?}",text_sect_hdr);
   assert_eq!(text_sect_hdr.len(),1);
   let sect_hdr = &text_sect_hdr[0];

   let text_section = read_text_section(&mut reader, &elf_header, sect_hdr).unwrap();
   assert!(!text_section.is_empty());
}

#[test]
fn should_get_local_symbols(){
   let path = Path::new("./assembly_tests/symbols.s");
   let file = write_asm_make_elf(&path,
      concat!(
         ".text\n",
         ".thumb\n\n",
         "_some_label:\n",
         "   ADD r0,#12\n",
         "   ADC r0,r1\n",
         "   LDM r7!, {r1,r2,r3}\n",
         "   WFE\n",
         "_some_other_label:\n",
         "   NOP\n",
         "_la_foo:\n",
         "  WFE\n\n"
     ).as_bytes()
   ).unwrap();
   //let file = Path::new("./assembly_tests/symbols.o");
   println!("made elf {:?}",file);
   let (elf_header,mut reader) = get_header(&file).unwrap();

   let section_headers = get_all_section_headers(&mut reader, &elf_header).unwrap();
   println!("sect_hdrs {:?}",section_headers);
   assert!(!section_headers.is_empty());

   let strtab_idx = get_string_table_section_hdr(&elf_header, &section_headers).unwrap();
   let str_table_hdr = &section_headers[strtab_idx];

   let maybe_symtab: Vec<&SectionHeader> = section_headers.iter()
      .filter(|hdr| is_symbol_table_section_hdr(&elf_header, hdr))
      .collect();

   println!("header {:?}",maybe_symtab[0]);

   
   let sym_entries = get_local_symbols(&mut reader, &elf_header, &maybe_symtab[0]).unwrap();

   let names = get_matching_symbol_names(&mut reader, &elf_header, &sym_entries, &str_table_hdr).unwrap();
   println!("{:?}",names);
   assert!(names.contains(&String::from("_some_label")));
   assert!(names.contains(&String::from("_some_other_label")));
   assert!(names.contains(&String::from("_la_foo")));
}
