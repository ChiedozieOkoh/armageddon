use std::path::Path;
use crate::elf::decoder::{
   get_header,
   get_all_section_headers,
   is_text_section_hdr,
   SectionHeader,
   read_text_section
};

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
