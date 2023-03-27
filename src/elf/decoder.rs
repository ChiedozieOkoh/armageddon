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
   ArmAttributes = 0x70000003,
}

pub enum SectionHeaderFlag{
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


pub fn read_text_section(
   reader: &mut BufReader<File>,
   elf_header: &ElfHeader,
   sect_header: &SectionHeader
   )->Result<Vec<u8>,ElfError>{
   //reader.rewind
   //
   if !is_text_section_hdr(elf_header, sect_header){
      return Err(ElfError::FileIO(String::from("cannot read this is not a text section")));
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
