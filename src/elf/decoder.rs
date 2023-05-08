use std::collections::HashMap;
use std::fmt::Display;
use std::io::{BufReader, Read,Seek};
use std::path::Path; 
use std::fs::File;

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
   pub _entry_point: Addr,// this is the address of main() we dont care about this
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

fn to_native_endianness_32b(header: &ElfHeader, bytes: &[u8;4])->u32{
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

pub fn is_arm_attribute_section_hdr(header: &ElfHeader, sect_header: &SectionHeader)->bool{
   todo!()
}

pub fn is_text_section_hdr(header: &ElfHeader, sect_header: &SectionHeader)->bool{
   let flags = to_native_endianness_32b(header,&sect_header.flags);
   let _type = to_native_endianness_32b(header, &sect_header._type);
   const text_mask: u32 = SectionHeaderFlag::Allocatable as u32 | SectionHeaderFlag::Executable as u32;
   println!("f: {} t: {} == f:{} t:{}",flags,_type,text_mask,SectionHeaderType::PROGBITS as u32);
   (_type == SectionHeaderType::PROGBITS as u32) && (flags == text_mask)
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

pub fn get_sh_string_table_header<'a>(header: &'a ElfHeader, section_headers: &'a Vec<SectionHeader>)->&'a SectionHeader{
   let sh_name_table_index = to_native_endianness_16b(header, &header.section_header_table_index);
   let section = &section_headers[sh_name_table_index as usize];
   let _type = to_native_endianness_32b(header, &section._type);
   let flags = to_native_endianness_32b(header, &section.flags);

   assert_eq!(_type, SectionHeaderType::StringTable as u32);
   assert_eq!(flags ,SectionHeaderFlag::Null as u32);
   section
}

//TODO add a function get labels which returns symbols from the symbols table and the offset in the text section.
//for GNU AS the value of a symbol table entry seems the byte offset of a symbol in the text section  
//symbols that are labels usually have type: NOTYPE, binding: local. Ignore the symbol with the name '$s' it seems to be a default generated by GAS 
//I have no idea how to do symbols in the data segment :D

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
   const size: usize = std::mem::size_of::<ElfHeader>();
   let mut source: [u8;size] = [0;size];
   let mut reader = BufReader::new(f);
   reader.read_exact(&mut source)?;
   let header: ElfHeader;
   unsafe {
      //spooky
      header = std::mem::transmute_copy::<[u8;size],ElfHeader>(&source);
   }

   if header.identity[..4] != ELF_FORMAT_HDR {
      return Err(ElfError::FileIO(String::from("unsupported format")));
   }

   if header.get_elf_endianess() == EIData::_None{
      return Err(ElfError::Arch(String::from("could not read endianness")));
   }
   
   Ok((header,reader))
}

pub fn get_sections_header(
   reader: &mut BufReader<File>,
   header: &ElfHeader
   )->Result<SectionHeader,ElfError>{
   let section_offset = to_native_endianness_32b(header, &header.section_header_offset);
   reader.seek(std::io::SeekFrom::Start(section_offset as u64))?;
   const size: usize = std::mem::size_of::<SectionHeader>();
   let mut source: [u8;size] = [0;size];
   reader.read_exact(&mut source)?;
   let header: SectionHeader;
   unsafe {
      //spooky
      header = std::mem::transmute_copy::<[u8;size],SectionHeader>(&source);
   }
   Ok(header)
}

pub fn get_all_section_headers(
   reader: &mut BufReader<File>,
   header: &ElfHeader
)->Result<Vec<SectionHeader>,ElfError>{
   let section_offset = to_native_endianness_32b(header, &header.section_header_offset);
   reader.seek(std::io::SeekFrom::Start(section_offset as u64))?;
   const size: usize = std::mem::size_of::<SectionHeader>();
   let mut header_meta_data_source: [u8;size] = [0;size];
   let num_headers = to_native_endianness_16b(header, &header.num_section_header_entries);
   println!("headers {}",num_headers);
   let mut headers: Vec<SectionHeader> = Vec::with_capacity(num_headers as usize);
   for _ in 0 .. num_headers{
      reader.read_exact(&mut header_meta_data_source)?;
      let header: SectionHeader;
      unsafe {
         //spooky
         header = std::mem::transmute_copy::<[u8;size],SectionHeader>(&header_meta_data_source);
      }
      headers.push(header);
   }
   Ok(headers)
} 

pub fn get_local_symbols(
   reader: &mut BufReader<File>,
   elf_header: &ElfHeader,
   symtable_hdr: &SectionHeader
   )->Result<Vec<SymbolTableEntry>, ElfError>{
   if !is_symbol_table_section_hdr(elf_header, symtable_hdr){
      return Err(ElfError::FileIO(String::from("Cannot read, the symbol table section header given was not a symbol table header")));
   }

   if has_no_bytes(elf_header, &symtable_hdr){
      println!("WARN| symbol table section header has not data... this is pretty strange");
      return Ok(Vec::new());
   }

   let symbol_table_offset = to_native_endianness_32b(elf_header, &symtable_hdr.offset_of_entries_in_bytes);
   let bytes = to_native_endianness_32b(elf_header, &symtable_hdr.section_size_in_bytes);
   let alignment = to_native_endianness_32b(elf_header, &symtable_hdr.alignment);
   let entry_size = to_native_endianness_32b(elf_header, &symtable_hdr.entry_size);
   println!("symbol table section is {} bytes",bytes);
   println!("address alignment {} bytes",alignment);
   println!("section entry size {} bytes",entry_size);

   const SYMBOL_ENTRY_SIZE: usize = std::mem::size_of::<SymbolTableEntry>();

   println!("number of symbol table entries = {}/{} = {}",bytes,entry_size,bytes/entry_size);
   println!("symbol struct entry size is {}",SYMBOL_ENTRY_SIZE);
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
      .filter(|e| has_no_type(e) && has_local_binding(e) && to_native_endianness_32b(elf_header, &e.name_index) != 0)
      .collect();

   assert!(local_symbols.is_empty() == false);
   //TODO link symbol entry to associated name from string table
   Ok(local_symbols)
}

pub fn get_text_section_symbols<'a>(
   elf_header: &ElfHeader,
   sect_hdrs: &Vec<SectionHeader>,
   sym_entries: &'a Vec<SymbolTableEntry>
   )->Option<Vec<&'a SymbolTableEntry>>{

   if let Some(i) = sect_hdrs.iter().position(|hdr| is_text_section_hdr(&elf_header, hdr)){
      Some(sym_entries.iter()
           .filter(|sym| to_native_endianness_16b(&elf_header, &sym.header_index) == (i as u16))
           .collect()
     )
   }else{
      None
   }
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

pub fn remove_assembler_artifact_symbols(
   names: &mut Vec<String>,
   sym_entries: &mut Vec<SymbolTableEntry>
){
   if let Some(i) =  names.iter().position(|name| name.eq("$t")){
      names.remove(i);
      sym_entries.remove(i);
   }
}

pub fn build_symbol_byte_offset_map(
   elf_header: &ElfHeader,
   names: Vec<String>,
   sym_entries: &Vec<SymbolTableEntry>
)->HashMap<u32, String>{
   let mut offset_map = HashMap::new();
   for (i, name) in names.into_iter().enumerate(){
      let symbol = &sym_entries[i];
      let offset: u32 = to_native_endianness_32b(&elf_header, &symbol.value);
      offset_map.insert(offset, name);
   }

   return offset_map;
}

pub fn read_text_section(
   reader: &mut BufReader<File>,
   elf_header: &ElfHeader,
   sect_header: &SectionHeader
   )->Result<Vec<u8>,ElfError>{
   //reader.rewind
   //
   if !is_text_section_hdr(elf_header, sect_header){
      return Err(ElfError::FileIO(String::from("cannot read, the section header given was now a text section header")));
   }
   if has_no_bytes(elf_header, &sect_header){
      println!("warn| text section has no data");
      return Ok(Vec::new());
   }
   let text_offset = to_native_endianness_32b(elf_header, &sect_header.offset_of_entries_in_bytes);
   let bytes = to_native_endianness_32b(elf_header, &sect_header.section_size_in_bytes);
   let alignment = to_native_endianness_32b(elf_header, &sect_header.alignment);
   let entry_size = to_native_endianness_32b(elf_header, &sect_header.entry_size);
   println!("text section is {} bytes",bytes);
   println!("address alignment {} bytes",alignment);
   println!("section entry size {} bytes",entry_size);
   let mut section = vec![0u8;bytes as usize];
   reader.seek(std::io::SeekFrom::Start(text_offset as u64))?;
   reader.read_exact(&mut section)?;
   Ok(section)
}

fn read_arm_attributes_section(
   reader: &mut BufReader<File>,
   elf_header: &ElfHeader,
   sect_header: &SectionHeader
   )->Result<Vec<u8>,ElfError>{
   todo!()
}
