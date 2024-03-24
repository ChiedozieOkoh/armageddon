use std::collections::HashMap;
use std::fmt::Display;
use std::io::{BufReader, Read,Seek};
use std::path::Path; 
use std::fs::File;

use crate::dbg_ln;

type Addr = [u8;4];
type ElfHalf = [u8;2];
type ElfOffset = [u8;4];
type ElfSword = [u8;4];
type ElfWord = [u8;4];

pub const ELF_FORMAT_HDR: [u8; 4] = [0x7f,'E' as u8,'L' as u8,'F' as u8];
pub const ELF_HDR_LEN: usize = 16;

#[repr(u8)]
#[derive(PartialEq)]
pub enum EIClass{
   _None = 0_u8,
   _32 = 1_u8,
   _64 = 2_u8
}

impl From<u8> for EIClass{
   fn from(byte: u8)-> Self{
      match byte{
         0 => EIClass::_None,
         1 => EIClass::_32,
         2 => EIClass::_64,
         _ => EIClass::_None
      }
   }
}

pub enum EIType{
   _None,
   Rel,
   Exec,
   Dyn,
   Core,
   Loproc,
   Hiproc
}

#[repr(u8)]
#[derive(PartialEq)]
pub enum EIData{
   _None,
   Lsb,
   Msb
}

impl From<u8> for EIData{
   fn from(byte: u8)-> Self{
      match byte{
         0 => EIData::_None,
         1 => EIData::Lsb,
         2 => EIData::Msb,
         _ => EIData::_None
      }
   }
}

pub enum EIVersion{
   _None,
   Current
}

impl From<u8> for EIVersion{
   fn from(byte: u8)-> Self{
      match byte{
         0 => EIVersion::_None,
         1 => EIVersion::Current,
         _ => EIVersion::_None
      }
   }
}

#[allow(non_camel_case_types)]
pub enum EIMachine{
   _None,
   Intelx86x64 = 3,
   Arm = 40
}

#[repr(C,packed)]
#[derive(Debug)]
pub struct ElfHeader{
   pub identity: [u8; ELF_HDR_LEN],
   pub e_type: ElfHalf,
   pub machine: ElfHalf,
   pub version: ElfWord,
   pub _entry_point: Addr,// turns out this is very useful :D
   pub program_header_offset: ElfOffset,
   pub section_header_offset: ElfOffset,
   pub cpu_flags: ElfWord,
   pub elf_header_size_in_bytes: ElfHalf,
   pub program_header_entry_size_in_bytes: ElfHalf,
   pub num_program_header_entries: ElfHalf,
   pub section_header_size_in_bytes: ElfHalf,
   pub num_section_header_entries: ElfHalf,
   pub section_header_table_index: ElfHalf
}

impl ElfHeader{
   pub fn get_elf_endianess(&self)-> EIData{
      EIData::from(self.identity[5])
   }
}

fn to_native_endianness_16b(header: &ElfHeader, bytes: &[u8;2])->u16{
   match header.get_elf_endianess(){
      EIData::_None => panic!("cannot read with unknown endianness"),
      EIData::Lsb => u16::from_le_bytes(*bytes),
      EIData::Msb => u16::from_be_bytes(*bytes)
   }
}

pub fn to_native_endianness_32b(header: &ElfHeader, bytes: &[u8;4])->u32{
   match header.get_elf_endianess(){
      EIData::_None => panic!("cannot read with unknown endianness"),
      EIData::Lsb => u32::from_le_bytes(*bytes),
      EIData::Msb => u32::from_be_bytes(*bytes)
   }
}

pub enum SectionHeaderType{
   NULL = 0x0,
   PROGBITS = 0x1,
   NOBITS = 0x8,
   SymbolTable = 0x2,
   StringTable = 0x3,
   ArmAttributes = 0x70000003,
}

pub enum SectionHeaderFlag{
   Null = 0x0,
   Write = 0x1,
   Allocatable  = 0x2,
   Executable = 0x4
}

#[repr(C,packed)]
#[derive(Debug)]
pub struct SectionHeader{
   name: ElfWord,
   _type: ElfWord,
   flags: ElfWord,
   _addr_in_memory_img: Addr,
   offset_of_entries_in_bytes: ElfOffset,
   section_size_in_bytes: ElfWord,  // not header size
   link: ElfWord,
   info: ElfWord,
   alignment: ElfWord,
   entry_size: ElfWord,
}


pub const SHN_ABS: u16 = 0xfff1;

#[repr(C,packed)]
#[derive(Debug)]
pub struct SymbolTableEntry{
   name_index: ElfWord,
   value: Addr,
   size: ElfWord,
   info: u8,
   other: u8,
   header_index: ElfHalf
}

impl From<&SymbolTableEntry> for Option<SymbolType>{
   fn from(symbol: &SymbolTableEntry) -> Self {
      match symbol.info & 0xf {
         0 => Some(SymbolType::Notype),
         1 => Some(SymbolType::Object),
         2 => Some(SymbolType::Func),
         3 => Some(SymbolType::Section),
         4 => Some(SymbolType::File),
         13 => Some(SymbolType::Loproc),
         15 => Some(SymbolType::Hiproc),
         _ => None
      }
   }
}

fn has_no_type(symbol: &SymbolTableEntry) ->bool{
   let sym_type: Option<SymbolType> = symbol.into();
   if let Some(SymbolType::Notype) = sym_type{
      true
   }else{
      false
   }
}
fn has_a_visible_type(symbol: &SymbolTableEntry)->bool{
   let sym_type: Option<SymbolType> = symbol.into();
   match sym_type{
      Some(s_type) =>{
         match s_type{
            SymbolType::Notype | SymbolType::Func => true,
            _ => false
         }
      },
      None => {
         println!("WARNING: Could not determine type of symbol from info {}",symbol.info);
         false
      }
   }
}

impl From<&SymbolTableEntry> for Option<SymbolBinding>{
   fn from(symbol: &SymbolTableEntry) -> Self {
      match symbol.info >> 4 {
         0 => Some(SymbolBinding::Local),
         1 => Some(SymbolBinding::Global),
         2 => Some(SymbolBinding::Weak),
         13 => Some(SymbolBinding::Loproc),
         15 => Some(SymbolBinding::Hiproc),
         _ => None
      } 
   }
}

fn has_local_binding(symbol: &SymbolTableEntry) -> bool{
   let binding: Option<SymbolBinding> = symbol.into();
   if let Some(SymbolBinding::Local) = binding{
      true
   }else{
      false
   }
}

fn has_any_binding(symbol: &SymbolTableEntry) -> bool{
   let binding: Option<SymbolBinding> = symbol.into();
   return binding.is_some();
}

#[derive(Debug)]
pub enum SymbolType{
   Notype = 0x0,
   Object = 0x1,
   Func = 0x2,
   Section = 0x3,
   File = 0x4,
   Loproc = 0xd,
   Hiproc = 0xf
}

pub enum SymbolBinding{
   Local = 0x0,
   Global = 0x1,
   Weak = 0x2,
   Loproc = 0xd,
   Hiproc = 0xf
}

pub fn is_text_section_hdr(header: &ElfHeader, sect_header: &SectionHeader)->bool{
   let flags = to_native_endianness_32b(header,&sect_header.flags);
   let _type = to_native_endianness_32b(header, &sect_header._type);
   const TEXT_MASK: u32 = SectionHeaderFlag::Allocatable as u32 | SectionHeaderFlag::Executable as u32;
   //dbg_ln!("f: {} t: {} == f:{} t:{}",flags,_type,TEXT_MASK,SectionHeaderType::PROGBITS as u32);
   (_type == SectionHeaderType::PROGBITS as u32) && (flags == TEXT_MASK)
}

pub fn is_data_section_hdr(header: &ElfHeader, sect_header: &SectionHeader)->bool{
   let flags = to_native_endianness_32b(header,&sect_header.flags);
   let _type = to_native_endianness_32b(header, &sect_header._type);
   const DATA_MASK: u32 = SectionHeaderFlag::Allocatable as u32 | SectionHeaderFlag::Write as u32;
   (_type == SectionHeaderType::PROGBITS as u32) && (flags == DATA_MASK)
}

pub fn is_symbol_table_section_hdr(header: &ElfHeader, sect_header: &SectionHeader)->bool{
   let _type = to_native_endianness_32b(header, &sect_header._type);
   return _type == SectionHeaderType::SymbolTable as u32;
}

pub fn is_str_table_section_hdr(header: &ElfHeader, sect_header: &SectionHeader)->bool{
   let _type = to_native_endianness_32b(header, &sect_header._type);
   let flags = to_native_endianness_32b(header, &sect_header.flags);
   return (flags == 0 || flags == (SectionHeaderFlag::Allocatable as u32)) && _type == SectionHeaderType::StringTable as u32;
}

pub fn get_string_table_section_hdr(header: &ElfHeader, section_headers: &Vec<SectionHeader>)->Option<usize>{
   for (i, sect_hdr) in section_headers.iter().enumerate(){
      let section_name_hdr_index = to_native_endianness_16b(header, &header.section_header_table_index);
      if is_str_table_section_hdr(header, sect_hdr) && i != section_name_hdr_index as usize{
         return Some(i);
      }
   }
   return None;
}


#[derive(Debug,PartialEq)]
pub enum LoadType{
   PROGBITS,
   NOBITS
}

#[derive(Debug,PartialEq)]
pub struct Section{
   pub name: String,
   pub start: u32,
   pub len: u32,
   pub load: LoadType
}

pub fn get_loadable_sections(
      reader: &mut BufReader<File>,
      header: &ElfHeader,
      sect_hdrs: &Vec<SectionHeader>
   )->Result<Vec<Section>,ElfError>{
   let mut loadable_sections = Vec::new();
   let sh_str_table_hdr = get_sh_string_table_header(header, sect_hdrs);
   for (i,hdr) in sect_hdrs.iter().enumerate(){
      let flags = to_native_endianness_32b( header, &hdr.flags);
      if flags & (SectionHeaderFlag::Allocatable as u32)>0{
         let name = section_name(reader,header,hdr,sh_str_table_hdr)?;
         println!("section header {} == {}",i,name);
         let _type = to_native_endianness_32b(header, &hdr._type);
         let addr = to_native_endianness_32b(header, &hdr._addr_in_memory_img);
         let size = to_native_endianness_32b(header, &hdr.section_size_in_bytes);
         if _type == SectionHeaderType::NOBITS as u32 {
            println!("{} section origin: {} len: {} type: NOBITS",name,addr,size);
            loadable_sections.push(Section{
                name,
                start: addr,
                len: size,
                load: LoadType::NOBITS,
            });
         }else if _type == SectionHeaderType::PROGBITS as u32{
            println!("{} section origin: {} len: {} type: PROGBITS",name,addr,size);
            loadable_sections.push(Section{
                name,
                start: addr,
                len: size,
                load: LoadType::PROGBITS,
            });
         }
      }
   }

   Ok(loadable_sections)
}

fn section_name(
      reader: &mut BufReader<File>,
      header: &ElfHeader,
      sect_hdr: &SectionHeader,
      sh_str_table_hdr: &SectionHeader
   )->Result<String, ElfError>{
   let offset = to_native_endianness_32b(
      header,
      &sh_str_table_hdr.offset_of_entries_in_bytes
   );
   let table_size = to_native_endianness_32b(
      header,
      &sh_str_table_hdr.section_size_in_bytes
   );

   let mut str_buffer = vec![0_u8;table_size as usize];
   reader.seek(std::io::SeekFrom::Start(offset as u64))?;
   reader.read_exact(&mut str_buffer)?;
   let sh_str_name = to_native_endianness_32b(header, &sh_str_table_hdr.name);
   //let mut name_map = vec![String::new();sect_hdrs.len()];

   let mut section_name = String::new();
   let name = to_native_endianness_32b(header, &sect_hdr.name);
   if name != sh_str_name{
      let mut c = name as usize;
      while str_buffer[c] as char != '\0' && (c < str_buffer.len()){
         section_name.push(str_buffer[c] as char);
         c += 1;
      }
      println!("section hdr == {}",section_name);
      //name_map.insert(i,section_name.clone());
      //name_map[i].push_str(&section_name);
   }

   Ok(section_name)
}



pub fn get_section_names(
      reader: &mut BufReader<File>,
      header: &ElfHeader,
      sect_hdrs: &Vec<SectionHeader>
   )->Result<Vec<String>, ElfError>{
   let sh_str_table_hdr = get_sh_string_table_header(header, sect_hdrs);
   let offset = to_native_endianness_32b(
      header,
      &sh_str_table_hdr.offset_of_entries_in_bytes
   );
   let table_size = to_native_endianness_32b(
      header,
      &sh_str_table_hdr.section_size_in_bytes
   );

   let mut str_buffer = vec![0_u8;table_size as usize];
   reader.seek(std::io::SeekFrom::Start(offset as u64))?;
   reader.read_exact(&mut str_buffer)?;
   let sh_str_name = to_native_endianness_32b(header, &sh_str_table_hdr.name);
   let mut section_name = String::new();
   //let mut name_map = vec![String::new();sect_hdrs.len()];
   let mut name_list = Vec::new();
   for (i,hdr) in sect_hdrs.iter().enumerate(){
      let name = to_native_endianness_32b(
         header,
         &hdr.name
      );
      if name != sh_str_name{
         let mut c = name as usize;
         while str_buffer[c] as char != '\0' && (c < str_buffer.len()){
            section_name.push(str_buffer[c] as char);
            c += 1;
         }
         println!("section hdr {} == {}",i,section_name);
         //name_map.insert(i,section_name.clone());
         //name_map[i].push_str(&section_name);
         name_list.push(section_name.clone());
         section_name.clear();
      }
   }
   Ok(name_list)
}

pub fn get_sh_string_table_header<'a>(header: &'a ElfHeader, section_headers: &'a Vec<SectionHeader>)->&'a SectionHeader{
   let sh_name_table_index = to_native_endianness_16b(header, &header.section_header_table_index);
   let section = &section_headers[sh_name_table_index as usize];
   let _type = to_native_endianness_32b(header, &section._type);
   let flags = to_native_endianness_32b(header, &section.flags);

   assert_eq!(_type, SectionHeaderType::StringTable as u32);
   assert_eq!(flags ,SectionHeaderFlag::Null as u32);
   section
}

pub fn has_no_bytes(header: &ElfHeader, sect_header: &SectionHeader)->bool{
   let flags = to_native_endianness_32b(header,&sect_header._type);
   (flags & SectionHeaderType::NOBITS as u32) > 0
}

#[derive(Debug)]
pub enum ElfError{
   Arch(String),
   FileIO(String),
}

impl From<std::io::Error> for ElfError{
   fn from(err: std::io::Error) -> Self{
      ElfError::FileIO(err.to_string())
   }
} 

impl Display for ElfError{
   fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
      match self{
         ElfError::FileIO(msg) => write!(f,"{}",msg),
         ElfError::Arch(msg) =>write!(f, "{}",msg)
      }
   }
}

pub fn get_header(file: &Path)-> Result<(ElfHeader, BufReader<File>),ElfError>{
   let f = File::open(file)?;
   const SIZE: usize = std::mem::size_of::<ElfHeader>();
   let mut source: [u8;SIZE] = [0;SIZE];
   let mut reader = BufReader::new(f);
   reader.read_exact(&mut source)?;
   let header: ElfHeader;
   unsafe {
      //spooky
      header = std::mem::transmute_copy::<[u8;SIZE],ElfHeader>(&source);
   }

   if header.identity[..4] != ELF_FORMAT_HDR {
      return Err(ElfError::FileIO(String::from("unsupported format")));
   }

   if header.get_elf_endianess() == EIData::_None{
      return Err(ElfError::Arch(String::from("could not read endianness")));
   }
   
   Ok((header,reader))
}

pub fn get_all_section_headers(
   reader: &mut BufReader<File>,
   header: &ElfHeader
)->Result<Vec<SectionHeader>,ElfError>{
   let section_offset = to_native_endianness_32b(header, &header.section_header_offset);
   reader.seek(std::io::SeekFrom::Start(section_offset as u64))?;
   const SIZE: usize = std::mem::size_of::<SectionHeader>();
   let mut header_meta_data_source: [u8;SIZE] = [0;SIZE];
   let num_headers = to_native_endianness_16b(header, &header.num_section_header_entries);
   dbg_ln!("headers {}",num_headers);
   let mut headers: Vec<SectionHeader> = Vec::with_capacity(num_headers as usize);
   for _ in 0 .. num_headers{
      reader.read_exact(&mut header_meta_data_source)?;
      let header: SectionHeader;
      unsafe {
         //spooky
         header = std::mem::transmute_copy::<[u8;SIZE],SectionHeader>(&header_meta_data_source);
      }
      headers.push(header);
   }
   Ok(headers)
} 

pub fn get_entry_point_offset(elf_header: &ElfHeader)->usize{
   let offset = to_native_endianness_32b(elf_header, &elf_header._entry_point);
   offset as usize
}

pub fn get_section_symbols(
   reader: &mut BufReader<File>,
   elf_header: &ElfHeader,
   symtable_hdr: &SectionHeader
   )->Result<Vec<SymbolTableEntry>, ElfError>{
   if !is_symbol_table_section_hdr(elf_header, symtable_hdr){
      return Err(ElfError::FileIO(String::from("Cannot read, the symbol table section header given was not a symbol table header")));
   }

   if has_no_bytes(elf_header, &symtable_hdr){
      dbg_ln!("WARN| symbol table section header has no data... this is pretty strange");
      return Ok(Vec::new());
   }

   let symbol_table_offset = to_native_endianness_32b(elf_header, &symtable_hdr.offset_of_entries_in_bytes);
   let bytes = to_native_endianness_32b(elf_header, &symtable_hdr.section_size_in_bytes);
   let alignment = to_native_endianness_32b(elf_header, &symtable_hdr.alignment);
   let entry_size = to_native_endianness_32b(elf_header, &symtable_hdr.entry_size);
   dbg_ln!("symbol table section is {} bytes",bytes);
   dbg_ln!("address alignment {} bytes",alignment);
   dbg_ln!("section entry size {} bytes",entry_size);

   const SYMBOL_ENTRY_SIZE: usize = std::mem::size_of::<SymbolTableEntry>();

   dbg_ln!("number of symbol table entries = {}/{} = {}",bytes,entry_size,bytes/entry_size);
   dbg_ln!("symbol struct entry size is {}",SYMBOL_ENTRY_SIZE);
   assert_eq!(SYMBOL_ENTRY_SIZE,entry_size as usize);

   reader.seek(std::io::SeekFrom::Start(symbol_table_offset as u64))?;

   let mut entry_buffer: [u8;SYMBOL_ENTRY_SIZE] = [0;SYMBOL_ENTRY_SIZE];
   let num_entries = bytes/entry_size;

   let mut entries: Vec<SymbolTableEntry> = Vec::new();
   for _ in 0 .. num_entries{
      reader.read_exact(&mut entry_buffer)?;
      let entry: SymbolTableEntry;
      unsafe{
         entry = std::mem::transmute_copy::<[u8;SYMBOL_ENTRY_SIZE],SymbolTableEntry>(&entry_buffer);
      }
      entries.push(entry);
   }

   let local_symbols: Vec<SymbolTableEntry> = entries.into_iter()
      .filter(|e| 
         has_a_visible_type(e) 
         && has_any_binding(e) 
         && to_native_endianness_32b(elf_header, &e.name_index) != 0)
      .collect();

   assert!(local_symbols.is_empty() == false);
   Ok(local_symbols)
}

pub fn get_text_section_symbols<'a>(
   elf_header: &ElfHeader,
   sect_hdrs: &Vec<SectionHeader>,
   sym_entries: &'a Vec<SymbolTableEntry>
   )->Option<Vec<&'a SymbolTableEntry>>{

   if let Some(i) = sect_hdrs.iter().position(|hdr| is_text_section_hdr(&elf_header, hdr)){
      dbg_ln!("text section index is {}",i);
      Some(sym_entries.iter()
           .filter(|sym| to_native_endianness_16b(&elf_header, &sym.header_index) == (i as u16))
           .collect()
     )
   }else{
      None
   }
}

#[derive(Debug)]
pub struct SymbolDefinition{
   pub position: usize,
   pub name: String,
   pub section_index: u16,
   pub _type: SymbolType
}

impl Ord for SymbolDefinition{
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
       if self.position != other.position{
          self.position.cmp(&other.position)
       }else{
          self.section_index.cmp(&other.section_index)
       }
    }
}

impl PartialOrd for SymbolDefinition{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for SymbolDefinition{
    fn eq(&self, other: &Self) -> bool {
       self.position == other.position 
          && self.section_index == other.section_index
          && self.name.eq(&other.name)
    }
}

impl Eq for SymbolDefinition{}

pub fn get_all_symbol_names(
   reader: &mut BufReader<File>,
   elf_header: &ElfHeader,
   sym_entries: &Vec<SymbolTableEntry>,
   str_table_hdr: &SectionHeader,
   )->Result<Vec<SymbolDefinition>,ElfError>{

   let names = get_matching_sym_in_place(reader, elf_header, sym_entries, str_table_hdr)?;
   println!("matching names: {:?}",names);
   let mut symbol_definitions = Vec::new();
   for (i,name) in names.into_iter().enumerate(){
      let addr = to_native_endianness_32b(elf_header, &sym_entries[i].value) as usize;
      let index = to_native_endianness_16b(elf_header, &sym_entries[i].header_index);
      let t: SymbolType = Into::<Option<SymbolType>>::into(&sym_entries[i]).unwrap();
      println!("symdef: {}@{}(hdr_idx: {})|t|",name,addr, index);
      symbol_definitions.push(
         SymbolDefinition{
            position: addr,
            name: name,
            section_index: index,
            _type: t
         });
   }

   symbol_definitions.sort_unstable();
   return Ok(symbol_definitions);
}

fn get_matching_sym_in_place(
   reader: &mut BufReader<File>,
   elf_header: &ElfHeader,
   sym_entries: &Vec<SymbolTableEntry>,
   str_table_hdr: &SectionHeader,
)-> Result<Vec<String>,ElfError>{
   let str_table_pos = to_native_endianness_32b(elf_header, &str_table_hdr.offset_of_entries_in_bytes);
   let str_table_size = to_native_endianness_32b(elf_header, &str_table_hdr.section_size_in_bytes);
   let mut str_buffer = vec![0u8;str_table_size as usize];

   reader.seek(std::io::SeekFrom::Start(str_table_pos as u64))?;
   reader.read_exact(&mut str_buffer)?;
   
   let mut symbol_names = Vec::new();
   let mut symbol_name = String::new();

   for symbol in sym_entries{
      let mut index = to_native_endianness_32b(elf_header, &symbol.name_index) as usize;
      while str_buffer[index] as char != '\0' && index < str_buffer.len(){
         symbol_name.push(str_buffer[index] as char);
         index += 1;
      }
      symbol_names.push(symbol_name.clone());
      symbol_name.clear();
   }
   return Ok(symbol_names);
}

pub fn get_matching_symbol_names(
   reader: &mut BufReader<File>,
   elf_header: &ElfHeader,
   sym_entries: &Vec<&SymbolTableEntry>,
   str_table_hdr: &SectionHeader,
)-> Result<Vec<String>,ElfError>{
   let str_table_pos = to_native_endianness_32b(elf_header, &str_table_hdr.offset_of_entries_in_bytes);
   let str_table_size = to_native_endianness_32b(elf_header, &str_table_hdr.section_size_in_bytes);
   let mut str_buffer = vec![0u8;str_table_size as usize];

   reader.seek(std::io::SeekFrom::Start(str_table_pos as u64))?;
   reader.read_exact(&mut str_buffer)?;
   
   let mut symbol_names = Vec::new();
   let mut symbol_name = String::new();

   for symbol in sym_entries{
      let mut index = to_native_endianness_32b(elf_header, &symbol.name_index) as usize;
      while str_buffer[index] as char != '\0'{
         symbol_name.push(str_buffer[index] as char);
         index += 1;
      }
      symbol_names.push(symbol_name.clone());
      symbol_name.clear();
   }
   return Ok(symbol_names);
}

pub fn build_symbol_byte_offset_map(
   elf_header: &ElfHeader,
   names: Vec<String>,
   sym_entries: &Vec<SymbolTableEntry>
)->HashMap<usize, String>{
   let mut offset_map = HashMap::new();
   for (i, name) in names.into_iter().enumerate(){
      if name.ne("$d") && name.ne("$t"){
         let symbol = &sym_entries[i];
         let offset: u32 = to_native_endianness_32b(&elf_header, &symbol.value);
         offset_map.insert(offset as usize, name);
      }
   }

   return offset_map;
}

#[derive(Debug)]
pub struct LiteralPools{
   data_marks: Vec<usize>,
   text_marks: Vec<usize>
}
#[derive(Debug)]
pub struct Pool{
   pub start: usize,
   pub end: Option<usize>
}

impl LiteralPools{
   pub fn create_from_list(
      symbols: &Vec<SymbolDefinition>,
   )->Self{
      let mut data_marks = Vec::new();
      let mut text_marks = Vec::new();
      for symbol in symbols.iter(){
         if symbol.name.eq("$d"){
            println!("literal pool {}@{}",symbol.name,symbol.position);
            data_marks.push(symbol.position);
         }
         if symbol.name.eq("$t"){
            text_marks.push(symbol.position);
         }
      }
      data_marks.sort();
      text_marks.sort();
      Self{data_marks, text_marks}
   }

   pub fn get_pool_at(&self, address: usize)->Option<Pool>{
      match self.data_marks.iter().position(|d| *d == address){
         Some(i) => {
            /*match self.text_marks.iter().position(|t| *t > address){
               Some(j) => {
                  println!("{} > {}",self.text_marks[j],self.data_marks[i]);
                  Some(Pool{start: self.data_marks[i], end: Some(self.text_marks[j])})
               },
               None => Some(Pool{start: self.data_marks[i], end: None})
            }*/
            let next_d_mark = self.data_marks.iter().position(|d| *d > address);
            let next_t_mark = self.text_marks.iter().position(|t| *t > address);
            println!("for {}, nt:{:?}, nd:{:?}",address,next_t_mark,next_d_mark);
            match next_t_mark{
               Some(t) => {
                  match next_d_mark{
                     Some(d) => {
                        println!("current {} min {} | {}",
                                 self.data_marks[i],
                                 self.data_marks[d],
                                 self.text_marks[t]
                                 );
                        Some(Pool{start: self.data_marks[i], end: Some(std::cmp::min(self.data_marks[d],self.text_marks[t]))})
                     },
                     None => Some(Pool{start: self.data_marks[i], end: Some(self.text_marks[t])})
                  }
               },
               None => {
                  match next_d_mark{
                     Some(d) => {
                        Some(Pool{start: self.data_marks[i], end: Some(self.data_marks[d])})
                     },
                     None => Some(Pool{start: self.data_marks[i], end: None})
                  }
               }
            }
         },
         None => None,
      }
   }
}

/*
pub struct SectionMap{
}
pub fn read_section_map(){
}*/

pub fn read_text_section(
   reader: &mut BufReader<File>,
   elf_header: &ElfHeader,
   sect_header: &SectionHeader
   )->Result<Vec<u8>,ElfError>{
   if !is_text_section_hdr(elf_header, sect_header){
      return Err(ElfError::FileIO(String::from("cannot read, the section header given was now a text section header")));
   }
   if has_no_bytes(elf_header, &sect_header){
      dbg_ln!("warn| text section has no data");
      return Ok(Vec::new());
   }
   let text_offset = to_native_endianness_32b(elf_header, &sect_header.offset_of_entries_in_bytes);
   let bytes = to_native_endianness_32b(elf_header, &sect_header.section_size_in_bytes);
   let alignment = to_native_endianness_32b(elf_header, &sect_header.alignment);
   let entry_size = to_native_endianness_32b(elf_header, &sect_header.entry_size);
   dbg_ln!("text section is {} bytes",bytes);
   dbg_ln!("address alignment {} bytes",alignment);
   dbg_ln!("section entry size {} bytes",entry_size);
   let mut section = vec![0u8;bytes as usize];
   reader.seek(std::io::SeekFrom::Start(text_offset as u64))?;
   reader.read_exact(&mut section)?;
   Ok(section)
}

