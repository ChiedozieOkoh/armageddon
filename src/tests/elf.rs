use std::path::{Path,PathBuf};
use std::fs::File;
use std::io::Write;
use std::process::Command;
use crate::asm::interpreter::SymbolTable;
use crate::elf::decoder::{
   get_header,
   get_all_section_headers,
   is_text_section_hdr,
   SectionHeader,
   read_text_section, is_symbol_table_section_hdr, get_section_symbols, get_string_table_section_hdr, get_matching_symbol_names, build_symbol_byte_offset_map,  get_text_section_symbols, get_entry_point_offset, LiteralPools, get_all_symbol_names, SymbolType, SymbolDefinition, get_section_names, get_loadable_sections, Section, LoadType
};

pub fn write_asm_make_elf(path: &str, data: &[u8])->Result<PathBuf, std::io::Error>{
   println!("writing to [{:?}]", path);
   let mut file = File::create(path)?;
   file.write_all(data)?;
   println!("wrote to {:?}", file);
   //println!("forcing T2 encoding");
   let mut fname = String::new();
   fname.push_str(path);
   fname = fname.replace(".s", ".elf");
   println!("writing to {:?}",fname);
   let ret = PathBuf::from(fname.clone());
   Command::new("arm-none-eabi-as")
      .arg(path)
      .arg("-march=armv6-m")
      .arg("-o")
      .arg(fname)
      .status()
      .expect("could not assemble");

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

pub fn link_elf(new_elf: &Path, elf: &Path, linker_script: &Path){
   let sh = Command::new("arm-none-eabi-ld")
      .arg("-T")
      .arg(linker_script)
      .arg(elf)
      .arg("-o")
      .arg(new_elf)
      .output()
      .expect("failed to link");

   println!("{:?}",std::str::from_utf8(&sh.stdout[..]).unwrap());
   println!("{:?}",std::str::from_utf8(&sh.stderr[..]).unwrap());
   assert!(sh.status.success());
}

#[test]
fn should_map_sections()->Result<(),std::io::Error>{
   let rel_elf = write_asm_make_elf(
      "./assembly_tests/section_map.s", 
      concat!(
         ".thumb\n",
         ".text\n",
         ".global _entry_point\n",
         "_entry_point:\n",
         "   NOP\n",
         "   NOP\n",
         ".data\n",
         "      .4byte 70,32,12,700\n",
         ".bss\n",
         "      .lcomm zero_counter_v, 8\n"
      ).as_bytes()
   )?;

   let mut ld_script = File::create("./assembly_tests/section.ld")?;

   let text_offset: usize = 0;
   let data_offset: usize = 0x1000;
   let bss_offset: usize = 0x2000;
   ld_script.write_all(
      format!(
         "ENTRY(_entry_point);\n\
         SECTIONS{{\n\
             \t. = {:#x};\n\
             \t.text : {{*(.text)}}\n\
             \t. = {:#x};\n\
             \t.data : {{*(.data)}}\n\
             \t. = {:#x};\n\
             \t.bss : {{*(.bss)}}\n\
         }}\n",text_offset,data_offset,bss_offset
      ).as_bytes()
   )?;

   let linked = Path::new("./assembly_tests/ld_section.out");
   link_elf(&linked, &rel_elf, Path::new("./assembly_tests/section.ld"));

   let (elf_header,mut reader) = get_header(&linked).unwrap();
   assert_eq!(text_offset,get_entry_point_offset(&elf_header), "entry point offset should be at the beginning of text segment");

   let section_headers = get_all_section_headers(&mut reader, &elf_header).unwrap();
   assert!(!section_headers.is_empty());

   let sects  = get_loadable_sections(&mut reader, &elf_header, &section_headers).unwrap();

   let t = sects.iter().position(|ld| ld.name.eq(".text")).unwrap();
   let d = sects.iter().position(|ld| ld.name.eq(".data")).unwrap();
   let b = sects.iter().position(|ld| ld.name.eq(".bss")).unwrap();

   assert_eq!(sects[t].name,String::from(".text"));
   assert_eq!(sects[t].start,text_offset as u32);
   assert_eq!(sects[t].len,4); // 2 * 16bit instructions = 4 bytes
   assert_eq!(sects[t].load,LoadType::PROGBITS); // 2 * 16bit instructions = 4 bytes

   assert_eq!(sects[d].name,String::from(".data"));
   assert_eq!(sects[d].start,data_offset as u32);
   assert_eq!(sects[d].len,4 * 4); // 4 .4byte numbers
   assert_eq!(sects[d].load,LoadType::PROGBITS); // 2 * 16bit instructions = 4 bytes

   assert_eq!(sects[b].name,String::from(".bss"));
   assert_eq!(sects[b].start,bss_offset as u32);
   assert_eq!(sects[b].len,8); // 4 .4byte numbers
   assert_eq!(sects[b].load,LoadType::NOBITS); // 2 * 16bit instructions = 4 bytes

   Ok(())
}

#[test]
fn should_get_all_symbols()->Result<(),std::io::Error>{
   let rel_elf = write_asm_make_elf(
      "./assembly_tests/all_symbols.s", 
      concat!(
         ".thumb\n",
         ".text\n",
         ".global _entry_point\n",
         "_entry_point:\n",
         "   LDR r0, =_boot_magic\n",
         "   B pl_end\n",
         "   .pool\n",
         "pl_end:\n",
         "   NOP\n",
         ".data\n",
         "   _boot_magic:\n",
         "      .4byte 70,32,12,700\n"
      ).as_bytes()
   )?;

   let mut ld_script = File::create("./assembly_tests/load.ld")?;
   let text_offset: usize = 0;
   let data_offset: usize = 0x1000;
   ld_script.write_all(
      format!(
         "ENTRY(_entry_point);\n\
         SECTIONS{{\n\
             \t. = {:#x};\n\
             \t.text : {{*(.text)}}\n\
             \t. = {:#x};\n\
             \t.data : {{*(.data)}}\n\
         }}\n",text_offset,data_offset
      ).as_bytes()
   )?;
   
   let linked = Path::new("./assembly_tests/allsym.out");
   link_elf(&linked, &rel_elf, Path::new("./assembly_tests/load.ld"));

   let (elf_header,mut reader) = get_header(&linked).unwrap();
   assert_eq!(text_offset,get_entry_point_offset(&elf_header), "entry point offset should be at the beginning of text segment");

   let section_headers = get_all_section_headers(&mut reader, &elf_header).unwrap();
   assert!(!section_headers.is_empty());

   let strtab_idx = get_string_table_section_hdr(&elf_header, &section_headers).unwrap();
   let str_table_hdr = &section_headers[strtab_idx];

   let maybe_symtab: Vec<&SectionHeader> = section_headers.iter()
      .filter(|hdr| is_symbol_table_section_hdr(&elf_header, hdr))
      .collect();

   let sym_entries = get_section_symbols(&mut reader, &elf_header, &maybe_symtab[0]).unwrap();
   //let text_section_symbols = get_text_section_symbols(&elf_header, &section_headers, &sym_entries).unwrap();
   //let names = get_matching_symbol_names(&mut reader, &elf_header, &text_section_symbols, &str_table_hdr).unwrap();
   let symbols  = get_all_symbol_names(&mut reader, &elf_header, &sym_entries, &str_table_hdr).unwrap();
   println!("symbols : {:?}",symbols);
   let lit_pools = LiteralPools::create_from_list(&symbols);
   println!("pool records: {:?}",lit_pools);
   let pool = lit_pools.get_pool_at(4).unwrap();
   assert_eq!(pool.start,4);
   assert_eq!(pool.end.unwrap(),8);
   let mut sym_table = SymbolTable::create(&symbols);
   
   assert_eq!(sym_table.lookup(0),Some(&String::from("_entry_point")));
   assert_eq!(sym_table.lookup(8),Some(&String::from("pl_end")));
   assert_eq!(sym_table.lookup(data_offset),Some(&String::from("_boot_magic")));
   assert_eq!(sym_table.lookup(data_offset + 2),None);
   assert!(symbols.contains(&SymbolDefinition{position: text_offset,name: String::from("_entry_point"), _type: SymbolType::Notype, section_index: 1}));
   assert!(symbols.contains(&SymbolDefinition{position: data_offset,name: String::from("_boot_magic"), _type: SymbolType::Notype, section_index: 2}));
   assert!(symbols.contains(&SymbolDefinition{position: 8,name: String::from("pl_end"), _type: SymbolType::Notype, section_index: 1}));
   Ok(())
}

#[test]
fn should_get_local_symbols(){
   let file = write_asm_make_elf("./assembly_tests/local_symbols.s",
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
         "   WFE\n\n",
         ".data\n",
         "   _msg:\n",
         "   .string \"Hello word\"\n\n"
     ).as_bytes()
   ).unwrap();

   println!("made elf {:?}",file);
   let (elf_header,mut reader) = get_header(&file).unwrap();

   assert_eq!(0,get_entry_point_offset(&elf_header), "elf hasn't been linked so it should be 0");

   let section_headers = get_all_section_headers(&mut reader, &elf_header).unwrap();
   println!("sect_hdrs {:?}",section_headers);
   assert!(!section_headers.is_empty());

   let strtab_idx = get_string_table_section_hdr(&elf_header, &section_headers).unwrap();
   let str_table_hdr = &section_headers[strtab_idx];

   let maybe_symtab: Vec<&SectionHeader> = section_headers.iter()
      .filter(|hdr| is_symbol_table_section_hdr(&elf_header, hdr))
      .collect();

   println!("header {:#?}",maybe_symtab[0]);

   
   let sym_entries = get_section_symbols(&mut reader, &elf_header, &maybe_symtab[0]).unwrap();
   let text_section_symbols = get_text_section_symbols(&elf_header, &section_headers, &sym_entries).unwrap();
   let names = get_matching_symbol_names(&mut reader, &elf_header, &text_section_symbols, &str_table_hdr).unwrap();
   assert!(names.contains(&String::from("_some_label")));
   assert!(names.contains(&String::from("_some_other_label")));
   assert!(names.contains(&String::from("_la_foo")));

   let text_sect_offset_map = build_symbol_byte_offset_map(&elf_header, names, &sym_entries);
   println!("{:?}",text_sect_offset_map);
   assert_eq!(text_sect_offset_map.get(&0),Some(&String::from("_some_label")));
   assert_eq!(text_sect_offset_map.get(&8),Some(&String::from("_some_other_label")));
   assert_eq!(text_sect_offset_map.get(&10),Some(&String::from("_la_foo")));
   assert_eq!(text_sect_offset_map.values().position(|v| v.eq("_msg")),None);
   assert_eq!(text_sect_offset_map.values().position(|v| v.eq("$t")),None);
   assert_eq!(text_sect_offset_map.values().position(|v| v.eq("$d")),None);

   //TODO test to ensure we can correctly retrive data segment symbols
   //TODO test to ensure we can source see data segment symbols in text segment i.e LDR _SOME_ADDR_LABEL dissassembles propperly
}
