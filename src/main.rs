mod asm;
mod elf;
mod dwarf;
mod system;
mod binutils;
mod log;
mod ui;

#[cfg(test)]
mod tests;

use std::io::Read;
use std::path::{Path, PathBuf};
use elf::decoder::{ElfError, SymbolDefinition};
use iced::Application;
use ui::parse_hex;

use crate::asm::interpreter::{print_assembly, disasm_text};
use crate::elf::decoder::{get_string_table_section_hdr, is_symbol_table_section_hdr, get_section_symbols, get_entry_point_offset, get_all_symbol_names};
use crate::system::System;
use crate::ui::App;

struct Args{
   pub elf: PathBuf,
   pub sp_reset_val: Option<u32>,
   pub vtor_override: Option<u32>,
   pub entry_point_override: Option<u32>,
   pub opt_load_section: Option<(String,u32)>,
   pub opt_jump: Option<u32>,
   pub manual_boot: bool
}

#[derive(Debug)]
struct ParseErr(String);

//TODO diassemble entire binary not just text section, load other segments into system
fn main() {
   gui_diasm();
   //cli_disasm();
}

const HELP_MSG: &'static str =  concat!(
   "Usage: armageddon <FILE> <OPTIONS>\n",
   "-h,--help               show this message\n",
   "\n",
   "--manual-boot           when this flag is passed the simulator will not execute the reset handler during startup\n",
   "\n",
   "--vtor=<HEX>            override the default value used for the vtor register.\n",
   "                        this will also change the SP value use after a reset. the SP value will be a u32 loaded from the address\n",
   "                        pointed to by the VTOR\n",
   "\n",
   "--sp-reset-val=<HEX>    specify the stack pointer value asigned during a reset.\n",
   "                        when this value is set via the CLI the entry_point of the ELF\n",
   "                        will be assumed to point to the reset routine handler\n",
   "\n",
   "--entrypoint=<HEX>      explicitly set the entry point \n",
   "\n",
   "--load-mem=<FILE:HEX>   specify a raw binary file to be mapped into memory at a specific hex address\n",
   "\n",
   "--jump-to=<HEX>  jump to an address after boot\n"
);

fn gui_diasm(){
   let args: Vec<String> = std::env::args().collect();

   dbg_ln!("DEBUG ENABLED");
   //let maybe_file = Path::new(&args[1]);
   if args.contains(&String::from("-h")) | args.contains(&String::from("--help")){
      println!("{}",HELP_MSG);
      std::process::exit(0);
   }

   let cli_arg = parse_args(args).unwrap(); 
   let maybe_instructions  = load_instruction_opcodes(&cli_arg.elf,cli_arg.opt_load_section.clone());
   exit_on_err(&maybe_instructions);

   let (disasm, mut entry_point, symbol_map, mut sys) = maybe_instructions.unwrap();
   println!("sys memory image: 0 -> {} pages ",sys.alloc.pages());
   println!("{} symbols defined",symbol_map.len());

   if cli_arg.entry_point_override.is_some(){
      println!("overriding entry point");
      entry_point = cli_arg.entry_point_override.unwrap() as usize;
      sys.set_pc(entry_point & !1).unwrap();
   }

   if cli_arg.sp_reset_val.is_some(){
      println!("overriding reset handler ptr and sp_reset_val");
      sys.reset_cfg = Some(system::ResetCfg {
         sp_reset_val: cli_arg.sp_reset_val.unwrap(), 
         reset_hander_ptr: (entry_point & (!1)) as u32
      });
   }

   if cli_arg.vtor_override.is_some(){
      println!("overriding default VTOR value");
      sys.set_vtor(cli_arg.vtor_override.unwrap());
   }

   sys.scs.wfi_wake_up = false;

   if !cli_arg.manual_boot{
      println!("system boot type: RESET");
      sys.reset();
   }else{
      println!("system boot type: MANUAL");
   }

   if let Some(addr) = cli_arg.opt_jump{
      sys.set_pc((addr & !1) as usize).unwrap();
   }

   
   /*
   println!("spoofing PI RESETS DONE register");
   sys.alloc.put(0x4000c008, [0xFF,0xFF,0xFF,1]);
   println!("spoofing PI ADC.CW register");
   sys.alloc.put(0x4004c000, [1,1,0,0]);
   */
   

   sys.trace_enabled = true;
   //let disasm = disasm_text(&instructions, entry_point, &symbol_map);
   let mut msg = String::new(); 
   for i in disasm.into_iter(){
      msg.push_str(&i);
      msg.push('\n');
   }
   let flags = (sys,entry_point,symbol_map, msg);
   App::run(iced::Settings::with_flags(flags)).unwrap();
}


fn get_optional_hex(args: &Vec<String>,name: &str)->Result<Option<u32>,ParseErr>{
   let maybe_val = args.iter().position(|a| a.starts_with(name));
   let mut val = None;
   match maybe_val{
      Some(i) => {
         match &args[i].strip_prefix(name){
            Some(input) => match parse_hex(*input){
                Some(v) => {val = Some(v);},
                None => {return Err(ParseErr(format!("invalid input for {}",name)));},
            },
            None => {return Err(ParseErr(format!("missing value for {}",name)));},
        }
      },
      None => {},
   }
   Ok(val)
}

fn get_first_parameter_arg<'a>(args: &'a Vec<String>,name: &str)->Result<Option<&'a str>,ParseErr>{
   let maybe_val = args.iter().position(|a| a.starts_with(name));
   match maybe_val{
      Some(i) => {
         match args[i].strip_prefix(name){
            Some(input) => {
               return Ok(Some(input));
            },
            None => {return Err(ParseErr(String::from(format!("missing value for {}",name))))}
        }
      },
      None => {Ok(None)},
   }
}

fn parse_args(args: Vec<String>)->Result<Args,ParseErr>{
   if args.len()  < 2{
      dbg_ln!("you must provide one elf file");
      return Err(ParseErr(String::from("you must provide one elf file")));
   }

   let maybe_file = PathBuf::from(&args[1]);
   let maybe_reset_val = args.iter().position(|a| a.starts_with("--sp-reset-val="));
   let mut reset_val = None;
   match maybe_reset_val{
      Some(i) => {
         match &args[i].strip_prefix("--sp-reset-val="){
            Some(input) => match parse_hex(*input){
                Some(v) => {reset_val = Some(v);},
                None => {return Err(ParseErr(String::from("invalid input for --sp-reset-val")));},
            },
            None => {return Err(ParseErr(String::from("missing value for --sp-reset-val")));},
        }
      },
      None => {},
   }


   let manual_boot = args.contains(&String::from("--manual-boot"));
   let maybe_vtor = get_optional_hex(&args, "--vtor=")?;
   let maybe_entry_point = get_optional_hex(&args, "--entrypoint=")?;
   let maybe_jump = get_optional_hex(&args, "--jump-to=")?;
   let maybe_load_opt = get_first_parameter_arg(&args, "--load-mem=")?;
   let load_sec_arg = match maybe_load_opt{
      Some(pair) => {
         match pair.split_once(':'){
            Some((fpath,addr_str)) => {
               match parse_hex(addr_str){
                  Some(c) => {
                     Some((fpath.to_owned(),c))
                  },
                  None => return Err(ParseErr(String::from(format!("{} is an invalid hex string",addr_str)))),
               }
            },
            None => None,
         }
      },
      None => None,
   };

   Ok(Args { 
      elf: maybe_file,
      sp_reset_val: reset_val,
      vtor_override: maybe_vtor,
      entry_point_override: maybe_entry_point,
      opt_load_section: load_sec_arg,
      opt_jump: maybe_jump,
      manual_boot
   })
}

fn cli_disasm(){
   let args: Vec<String> = std::env::args().collect();

   dbg_ln!("DEBUG ENABLED");
   if args.contains(&String::from("-h")) | args.contains(&String::from("--help")){
      println!("{}",HELP_MSG);
      std::process::exit(0);
   }

   let cli_arg = parse_args(args).unwrap(); 
   let maybe_instructions  = load_instruction_opcodes(&cli_arg.elf, None);
   exit_on_err(&maybe_instructions);

   //let (instructions, entry_point, symbol_map) = maybe_instructions.unwrap();
   //print_assembly(&instructions[..],entry_point, &symbol_map);
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

fn load_instruction_opcodes(file: &Path, extra_sec: Option<(String,u32)>)->Result<(Vec<String>, usize, Vec<SymbolDefinition>, System),ElfError>{
   use crate::elf::decoder::{
      SectionHeader,
      get_header,
      get_all_section_headers,
      get_loadable_sections,
      load_sections
   };
   let (elf_header,mut reader) = get_header(file)?;

   let section_headers = get_all_section_headers(&mut reader, &elf_header)?;
   dbg_ln!("sect_hdrs {:?}",section_headers);
   assert!(!section_headers.is_empty());

   let strtab_idx = get_string_table_section_hdr(&elf_header, &section_headers).unwrap();
   let str_table_hdr = &section_headers[strtab_idx];

   let maybe_symtab: Vec<&SectionHeader> = section_headers.iter()
      .filter(|hdr| is_symbol_table_section_hdr(&elf_header, hdr))
      .collect();

   let sym_entries = get_section_symbols(&mut reader, &elf_header, &maybe_symtab[0]).unwrap();


   //let text_section_symbols = get_text_section_symbols(&elf_header, &section_headers, &sym_entries).unwrap();
   //let names = get_matching_symbol_names(&mut reader, &elf_header, &text_section_symbols, &str_table_hdr).unwrap();
   //let text_sect_offset_map = build_symbol_byte_offset_map(&elf_header, names, &sym_entries);
   let symbols = get_all_symbol_names(&mut reader, &elf_header, &sym_entries, str_table_hdr).unwrap();

   let loadable = get_loadable_sections(&mut reader, &elf_header,&section_headers)?;

   let mut section_data = load_sections(&mut reader, &elf_header, &section_headers, loadable)?;

   let t = section_data.iter().position(|(name,_,_)|name.eq(".text")).expect("ELF file did not specify a .text section ???");

   let (_,text_offset,text_data) = &section_data[t];
   let disasm = disasm_text(text_data, *text_offset as usize, &symbols);
   let entry_point = get_entry_point_offset(&elf_header);

   if let Some((fpath,addr)) = extra_sec{
      let mut fhandle = std::fs::File::open(&fpath)?;
      let mut sdata: Vec<u8> = Vec::new();
      fhandle.read_to_end(&mut sdata)?;
      println!("adding user section: {} [{:x} -> {:x}]",&fpath,addr,addr + sdata.len() as u32);
      section_data.push((fpath,addr,sdata));
   }

   let mut sys = System::with_sections(section_data);
   sys.set_pc(entry_point & (!1)).unwrap();
   Ok((disasm, entry_point, symbols, sys))
}

fn exit_on_err<T>(maybe_err: &Result<T,ElfError>){
   match maybe_err{
      Err(e) => {println!("{}",e); std::process::exit(-1);},
      Ok(_) => {}
   }
}
