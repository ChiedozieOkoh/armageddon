use std::{io::{BufReader, Read}, path::Path, fs::File};

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

pub enum SectionHeaderType{
   NULL,
   PROGBITS 
}

pub enum SectionHeaderFlag{
   Write = 0x1,
   Read  = 0x2,
   Executable = 0x4
}

#[repr(C,packed)]
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
pub fn get_header(file: &Path)-> Result<ElfHeader,ElfError>{
   let f = File::open(file)?;
   /* let mut header_buf = vec![0; ELF_HDR_LEN];
   let reader = BufReader::new(f);
   reader.read_exact(&mut header_buf)?;
   if header_buf[..4] != ELF_FORMAT_HDR{
      return Err(ElfError::FileIO(String::from("unsupported format")));
   }
   let (class, data, version): (EIClass, EIData, EIVersion) = match header_buf[4..8]{
      [e_class,e_data,e_version] => {
         (e_class.into(), e_data.into(), e_version.into())
      }
   };
   if class == EIClass::_None{
      return Err(ElfError::FileIO(
            String::from("could not determine whether object size is 32 or 64, EIClass was undefined" )
         ));
   }
   */

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
   
   Ok(header)
}
