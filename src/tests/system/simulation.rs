
use std::any::Any;
use std::fs::File;
use std::io::{Write, Seek, SeekFrom};
use std::io::{Error,ErrorKind};
use std::path::Path;

use crate::binutils::{from_arm_bytes, from_arm_bytes_16b, u32_to_arm_bytes, into_arm_bytes};
use crate::system::instructions::{zero_flag, negative_flag, carry_flag, overflow_flag};
use crate::system::registers::{get_overflow_bit, get_carry_bit};
use crate::tests::asm::{write_asm, asm_file_to_elf, asm_file_to_elf_armv6m};
use crate::tests::elf::{write_asm_make_elf, link_elf};
use crate::tests::system::{run_script_on_remote_cpu, parse_gdb_output, print_states};
use crate::system::{ArmException, System, ExceptionStatus, Mode, SystemControlSpace};
use crate::elf::decoder::{to_native_endianness_32b, ElfError, get_string_table_section_hdr, is_symbol_table_section_hdr, get_section_symbols, get_loadable_sections, load_sections, get_all_symbol_names, SymbolDefinition, LiteralPools};
use crate::to_arm_bytes;
use super::{gdb_script, PROC_VARIABLES};

use crate::elf::decoder::{
      SectionHeader,
      get_header,
      get_all_section_headers,
      is_text_section_hdr,
      read_text_section
   };

fn run_assembly<
   T: Any,
   F: Fn(usize,&[u8])->Result<T,ArmException>
>(path: &Path,code: &[u8], interpreter: F)->Result<T,std::io::Error>{
   write_asm(path, code)?;
   let elf = asm_file_to_elf(path)?;

   let (elf_header,mut reader) = get_header(&elf).unwrap();

   let maybe_hdr = get_all_section_headers(&mut reader, &elf_header);
   if maybe_hdr.is_err(){
      std::fs::remove_file(path)?;
      std::fs::remove_file(&elf)?;
      return Err(Error::new(ErrorKind::Other, "could not read hdr"));
   }

   let section_headers = maybe_hdr.unwrap();
   println!("sect_hdrs {:?}",section_headers);
   assert!(!section_headers.is_empty());

   let text_sect_hdr: Vec<SectionHeader> = section_headers.into_iter()
      .filter(|hdr| is_text_section_hdr(&elf_header, hdr))
      .collect();

   println!("header {:?}",text_sect_hdr);
   assert_eq!(text_sect_hdr.len(),1);
   let sect_hdr = &text_sect_hdr[0];

   let maybe_text_section = read_text_section(&mut reader, &elf_header, sect_hdr);
   if maybe_text_section.is_err(){
      std::fs::remove_file(path)?;
      std::fs::remove_file(&elf)?;
      return Err(Error::new(ErrorKind::Other, "could not read text section"));
   }

   let text_section = maybe_text_section.unwrap();
   assert!(!text_section.is_empty());

   let entry_point = to_native_endianness_32b(&elf_header, &elf_header._entry_point);

   let res = interpreter(entry_point as usize, &text_section[..]);

   if res.as_ref().is_err(){
      println!("failed execution exited due to: {:?}",res.as_ref().err());
   }

   std::fs::remove_file(path)?;
   std::fs::remove_file(&elf)?;
   return Ok(res.unwrap());
}


fn run_elf<
   T: Any,
   F: Fn(usize,&[u8])->Result<T,ArmException>
>(elf: &Path, interpreter: F)->Result<T,std::io::Error>{
   let (elf_header,mut reader) = get_header(&elf).unwrap();

   let maybe_hdr = get_all_section_headers(&mut reader, &elf_header);
   if maybe_hdr.is_err(){
      std::fs::remove_file(&elf)?;
      return Err(Error::new(ErrorKind::Other, "could not read hdr"));
   }

   let section_headers = maybe_hdr.unwrap();
   println!("sect_hdrs {:?}",section_headers);
   assert!(!section_headers.is_empty());

   let text_sect_hdr: Vec<SectionHeader> = section_headers.into_iter()
      .filter(|hdr| is_text_section_hdr(&elf_header, hdr))
      .collect();

   println!("header {:?}",text_sect_hdr);
   assert_eq!(text_sect_hdr.len(),1);
   let sect_hdr = &text_sect_hdr[0];

   let maybe_text_section = read_text_section(&mut reader, &elf_header, sect_hdr);
   if maybe_text_section.is_err(){
      std::fs::remove_file(&elf)?;
      return Err(Error::new(ErrorKind::Other, "could not read text section"));
   }

   let text_section = maybe_text_section.unwrap();
   assert!(!text_section.is_empty());

   let entry_point = to_native_endianness_32b(&elf_header, &elf_header._entry_point);

   let res = interpreter(entry_point as usize, &text_section[..]);

   if res.as_ref().is_err(){
      println!("failed execution exited due to: {:?}",res.as_ref().err());
   }

   std::fs::remove_file(&elf)?;
   return Ok(res.unwrap());
}

fn run_assembly_armv6m<
   T: Any,
   F: Fn(usize,&[u8])->Result<T,ArmException>
>(path: &Path,code: &[u8], interpreter: F)->Result<T,std::io::Error>{
   write_asm(path, code)?;
   let elf = asm_file_to_elf_armv6m(path)?;

   let (elf_header,mut reader) = get_header(&elf).unwrap();

   let maybe_hdr = get_all_section_headers(&mut reader, &elf_header);
   if maybe_hdr.is_err(){
      std::fs::remove_file(path)?;
      std::fs::remove_file(&elf)?;
      return Err(Error::new(ErrorKind::Other, "could not read hdr"));
   }

   let section_headers = maybe_hdr.unwrap();
   println!("sect_hdrs {:?}",section_headers);
   assert!(!section_headers.is_empty());

   let text_sect_hdr: Vec<SectionHeader> = section_headers.into_iter()
      .filter(|hdr| is_text_section_hdr(&elf_header, hdr))
      .collect();

   println!("header {:?}",text_sect_hdr);
   assert_eq!(text_sect_hdr.len(),1);
   let sect_hdr = &text_sect_hdr[0];

   let maybe_text_section = read_text_section(&mut reader, &elf_header, sect_hdr);
   if maybe_text_section.is_err(){
      std::fs::remove_file(path)?;
      std::fs::remove_file(&elf)?;
      return Err(Error::new(ErrorKind::Other, "could not read text section"));
   }

   let text_section = maybe_text_section.unwrap();
   assert!(!text_section.is_empty());

   let entry_point = to_native_endianness_32b(&elf_header, &elf_header._entry_point);

   let res = interpreter(entry_point as usize, &text_section[..]);

   if res.as_ref().is_err(){
      println!("failed execution exited due to: {:?}",res.as_ref().err());
   }

   std::fs::remove_file(path)?;
   std::fs::remove_file(&elf)?;
   return Ok(res.unwrap());
}

fn load_code_into_system(entry_point: usize, code: &[u8])->Result<System, ArmException>{
   //let mut sys = System::create(0);
   //for c in code{
   //   sys.memory.push(*c);
   //}
   let mut sys = System::fill_with(code);
   sys.set_pc(entry_point)?;
   return Ok(sys);
}

#[test]
pub fn should_do_add()->Result<(),std::io::Error>{
   let code = concat!(
      ".thumb\n",
      ".text\n",
      "ADD r0,r1\n",
      "ADD r0,r1,#2\n",
      "ADD r0,#200\n",
      "ADD r0,r1\n",
      "ADD r4,SP,#1004\n",
      "ADD SP,#212\n"
   );
   run_assembly(
      &Path::new("sim_add.s"),
      code.as_bytes(),
      |entry_point, code|{
         let mut sys = load_code_into_system(entry_point, code)?;
         //println!("memory: {:?}",sys.memory);
         sys.registers.generic[0] = 20;
         sys.registers.generic[1] = 40;
         sys.step()?;
         assert_eq!(sys.registers.generic[0], 60);

         sys.set_pc(sys.registers.pc + 2)?;
         sys.step()?;
         assert_eq!(sys.registers.generic[0],42);

         sys.set_pc(sys.registers.pc + 2)?;
         sys.step()?;
         assert_eq!(sys.registers.generic[0],242);

         sys.registers.generic[0] = 0xF0000000;
         sys.registers.generic[1] = 1;
         sys.set_pc(sys.registers.pc + 2)?;
         sys.step()?;
         assert_eq!(sys.registers.generic[0], 0xF0000000 + 1);

         let sp_value = 320;
         sys.registers.sp_main = sp_value;
         sys.registers.sp_process = sp_value;
         sys.set_pc(sys.registers.pc + 2)?;
         sys.step()?;
         assert_eq!(sys.registers.generic[4],1004 + sp_value);

         sys.set_pc(sys.registers.pc + 2)?;
         sys.step()?;
         assert_eq!(sys.get_sp(),212 + sp_value);

         return Ok(());
      }
   )?;
   
   return Ok(());
}

#[test]
pub fn should_do_mul()->Result<(),std::io::Error>{
   run_assembly(
      &Path::new("sim_mul.s"),
      b".thumb\n.text\nMUL r0,r1\n",
      |entry_point, code|{
         let mut sys = load_code_into_system(entry_point, code)?;
         sys.registers.generic[0] = 7;
         sys.registers.generic[1] = 3;
         let carry = carry_flag(sys.xpsr);
         let overflow = overflow_flag(sys.xpsr);

         sys.step()?;

         assert_eq!(sys.registers.generic[0], 21);
         assert!(!negative_flag(sys.xpsr));
         assert!(!zero_flag(sys.xpsr));
         assert_eq!(carry_flag(sys.xpsr),carry);
         assert_eq!(overflow_flag(sys.xpsr),overflow);
         return Ok(());
      }
   )?;

   return Ok(());
}

#[test]
pub fn should_do_move()->Result<(),std::io::Error>{
   run_assembly(
      &Path::new("sim_mov.s"), 
      b".thumb\n.text\nMOV r0,#2\nMOV r2,#47\nMOV r1,r2",
      |entry_point, code|{
         let mut sys = load_code_into_system(entry_point, code)?;
         let n = sys.step()?;
         assert_eq!(sys.registers.generic[0],2);
         sys.offset_pc(n)?;

         let n = sys.step()?;
         assert_eq!(sys.registers.generic[2],47);
         sys.offset_pc(n)?;

         let n = sys.step()?;
         assert_eq!(sys.registers.generic[1],47);
         sys.offset_pc(n)?;
         return Ok(());
      }
   )?;

   return Ok(());
}

#[test]
pub fn should_do_rev()->Result<(),std::io::Error>{
   run_assembly(
      &Path::new("sim_ror.s"),
      b".thumb
         .text
         .thumb
         REV r0,r0
         REV16 r0,r0
         REVSH r0,r0
         REVSH r0,r0
      ",
      |entry_point, code|{
         let mut sys = load_code_into_system(entry_point, code)?;
         sys.registers.generic[0] = 0x99AA88CC;
         let n = sys.step()?;
         assert_eq!(sys.registers.generic[0],0xCC88AA99);
         sys.offset_pc(n)?;

         sys.registers.generic[0] = 0xFA82DB14;
         let n = sys.step()?;
         assert_eq!(sys.registers.generic[0],0x82FA14DB);
         sys.offset_pc(n)?;

         sys.registers.generic[0] = 0x09A3;
         let n = sys.step()?;
         assert_eq!(sys.registers.generic[0],0xFFFFFF09);
         sys.offset_pc(n)?;

         sys.registers.generic[0] = 0xFA73;
         let n = sys.step()?;
         assert_eq!(sys.registers.generic[0],0xFA);
         sys.offset_pc(n)?;

         return Ok(());
      }
   )?;
   return Ok(());
}

#[test]
pub fn should_do_sub()->Result<(),std::io::Error>{
   run_assembly(
      &Path::new("sum_sub.s"),
      b".thumb\n.text\nSUB r0,r1,r2\nSUB r0,r1,#5\nSUB r1,#120",
      |entry_point, code|{
         let mut sys = load_code_into_system(entry_point, code)?;
         sys.registers.generic[0] = 0;
         sys.registers.generic[1] = 500;
         sys.registers.generic[2] = 400;
         sys.step()?;
         assert_eq!(sys.registers.generic[0],100);

         sys.set_pc(sys.registers.pc + 2)?;
         sys.step()?;
         assert_eq!(sys.registers.generic[0],495);

         sys.set_pc(sys.registers.pc + 2)?;
         sys.step()?;
         assert_eq!(sys.registers.generic[1],380);
         return Ok(());
      }
   )?;
   return Ok(());
}

#[test]
pub fn should_do_compare()->Result<(),std::io::Error>{
   run_assembly(
      &Path::new("sim_cmp.s"),
      b".thumb
      .text
      CMP r0,r1 
      CMP r0,#0
      CMP r0,#1
      CMP r0,#2
      CMP r0,#1
      CMP r0,#20
      CMP r0,#1
      ",
      |entry_point, code|{
         let mut sys = load_code_into_system(entry_point, code)?;
         sys.registers.generic[0] = 0;
         sys.registers.generic[1] = 100;
         let mut incr = sys.step()?;
         println!("N:{} Z:{} C:{} V:{}",
            negative_flag(sys.xpsr),
            zero_flag(sys.xpsr),
            carry_flag(sys.xpsr),
            overflow_flag(sys.xpsr)
         );
         assert!(overflow_flag(sys.xpsr) != negative_flag(sys.xpsr)); //testing BLT
         sys.offset_pc(incr)?;

         sys.registers.generic[0] = 0; //testing BEQ
         incr = sys.step()?;
         assert_eq!(zero_flag(sys.xpsr), true);
         sys.offset_pc(incr)?;

         incr = sys.step()?; //testing BNE
         assert_eq!(zero_flag(sys.xpsr), false);
         sys.offset_pc(incr)?;

         sys.registers.generic[0] = 20; // testing BGT
         incr = sys.step()?;
         assert_eq!(zero_flag(sys.xpsr),false);
         assert_eq!(zero_flag(sys.xpsr),negative_flag(sys.xpsr));
         sys.offset_pc(incr)?;

         sys.registers.generic[0] = 2; //testing BHI
         incr = sys.step()?;
         assert_eq!(zero_flag(sys.xpsr),false);
         assert_eq!(carry_flag(sys.xpsr),true);
         sys.offset_pc(incr)?;

         sys.registers.generic[0] = 1; //testing BLS
         incr = sys.step()?;
         assert!(!carry_flag(sys.xpsr) || zero_flag(sys.xpsr));
         sys.offset_pc(incr)?;

         sys.registers.generic[0] = 1; //testing BLS
         incr = sys.step()?;
         assert!(!carry_flag(sys.xpsr) || zero_flag(sys.xpsr));
         return Ok(());
      }
   )?;
   return Ok(());
}

#[test]
pub fn should_do_shifts()->Result<(), std::io::Error>{
   let code = b".thumb
      .text
         MOV r0,#20
         LSL r0,r0,#2
         MOV r1,#1
         MOV r3,r0
         LSL r3,r1
         MOV r5,#124
         LSR r5,r5,#4
         MOV r6,#2
         LSR r5,r6
         MOV r0,#1
         LSR r0,#1
         MOV r0,#1
         LSL r0,#31
         LSL r0,#1
      ";

   run_assembly_armv6m(
      Path::new("sim_shift.s"),
      code,
      |entry_point, bin|{
         let mut sys = load_code_into_system(entry_point, bin)?;
         let i = sys.step()?;
         sys.offset_pc(i)?;

         let i = sys.step()?;
         sys.offset_pc(i)?;
         assert_eq!(sys.registers.generic[0],20 << 2);

         let i = sys.step()?;
         sys.offset_pc(i)?;

         let i = sys.step()?;
         sys.offset_pc(i)?;

         let i = sys.step()?;
         sys.offset_pc(i)?;
         assert_eq!(sys.registers.generic[3],20 << 3);

         let i = sys.step()?;
         sys.offset_pc(i)?;

         let i = sys.step()?;
         sys.offset_pc(i)?;
         assert_eq!(sys.registers.generic[5],124 >> 4);

         let i = sys.step()?;
         sys.offset_pc(i)?;

         let i = sys.step()?;
         sys.offset_pc(i)?;
         assert_eq!(sys.registers.generic[5],124 >> 6);

         let i = sys.step()?;
         sys.offset_pc(i)?;

         let i = sys.step()?;
         sys.offset_pc(i)?;
         assert_eq!(sys.registers.generic[0],0);
         assert!(get_carry_bit(sys.xpsr));

         let i = sys.step()?;
         sys.offset_pc(i)?;

         let i = sys.step()?;
         sys.offset_pc(i)?;
         assert_eq!(sys.registers.generic[0],1 << 31);

         let i = sys.step()?;
         sys.offset_pc(i)?;
         assert_eq!(sys.registers.generic[0],0);
         assert!(get_carry_bit(sys.xpsr));
         Ok(())
      }
   )?;
   return Ok(());
}

#[test]
pub fn should_support_ctrl_register()->Result<(), std::io::Error>{
   let bin_size = 1024;
   let code = b"
   .thumb
   .text 
      LDR r0, =0xFFFF
      LDR r1, =1024;
      MSR PSP, r1
      MSR CONTROL,r0
      PUSH {r0,r1}
      ";
   run_assembly_armv6m(
      Path::new("sim_ctrl_register.s"),
      code,
      |entry_point, binary|{
         let mut sys = load_code_into_system(entry_point, binary)?;
         //sys.expand_memory_to(bin_size);
         let i = sys.step()?;  //LDR r0, =0xFFFF
         sys.offset_pc(i)?;

         let i = sys.step()?;  //LDR r0, =1024
         sys.offset_pc(i)?;

         let i = sys.step()?;  //MSR PSP,r0
         assert_eq!(sys.registers.sp_process, 1024);
         assert!(sys.get_sp() != 1024);
         sys.offset_pc(i)?;


         let i = sys.step()?;  //MSR CONTROL,r0
         assert_eq!(sys.get_sp(), 1024);
         sys.offset_pc(i)?;

         let i = sys.step()?; //PUSH {r0,r1]
         assert_eq!(sys.get_sp(),1024 - (4*2));
         assert_eq!(sys.registers.sp_process,1024 - (4*2));
         sys.offset_pc(i)?;

         Ok(())
   })?;

   Ok(())
}

#[test]
pub fn should_support_stack()->Result<(), std::io::Error>{
   let bin_size = 1024;
   let code = b".thumb
      .text
         LDR r0, =1024
         MSR MSP,r0
         MOV r0,#5
         MOV r1,#17
         MOV r2,#56
         PUSH {r0,r1,r2}
         POP  {r4,r5,r6}
         ADD r0,r4,r5
         ADD r0,r0,r6
      .pool
      ";

   run_assembly_armv6m(
      Path::new("sim_stack.s"),
      code,
      |entry_point, binary|{
         let mut sys = load_code_into_system(entry_point, binary)?;
         //sys.expand_memory_to(bin_size);
         let i = sys.step()?;  //LDR r0, =1024
         sys.offset_pc(i)?;

         let i = sys.step()?;  //MSR MSP,r0
         assert_eq!(sys.get_sp(), 1024);
         assert_eq!(sys.registers.sp_main, 1024);
         sys.offset_pc(i)?;

         let i = sys.step()?; //MOV r0, #5
         sys.offset_pc(i)?;

         let i = sys.step()?; //MOV r1, #17
         sys.offset_pc(i)?;

         let i = sys.step()?; //MOV r2, #56
         sys.offset_pc(i)?;

         let i = sys.step()?; //PUSH {r0,r1,r2}
         assert_eq!(sys.get_sp(),1024 - (4*3));
         assert_eq!(sys.registers.sp_main,1024 - (4*3));
         sys.offset_pc(i)?;

         let i = sys.step()?; //POP {r4,r5,r6}
         assert_eq!(sys.registers.generic[4],5);
         assert_eq!(sys.registers.generic[5],17);
         assert_eq!(sys.registers.generic[6],56);
         sys.offset_pc(i)?;

         let i = sys.step()?;
         assert_eq!(sys.registers.generic[0], 5 + 17);
         sys.offset_pc(i)?;

         let i = sys.step()?;
         assert_eq!(sys.registers.generic[0], 5 + 17 + 56);
         assert_eq!(sys.get_sp(),1024);
         assert_eq!(sys.registers.sp_main, 1024);
         sys.offset_pc(i)?;

         Ok(())
   })?;

   Ok(())
}

#[test]
pub fn support_exceptions()->Result<(),std::io::Error>{
   let elf = write_asm_make_elf(
      "./assembly_tests/basic_exception.s",
      concat!(
         ".thumb\n",
         ".text\n",
         ".equ _STACK_SIZE,0x20\n",
         ".global _entry_point\n",
         "_vector_table:\n",
         "   .4byte _SP_RESET_VAL\n",
         "   .4byte _reset_handler\n",
         "   .4byte _nmi_handler\n",
         ".thumb_func\n",
         "_reset_handler:\n",
         "   BKPT\n",
         ".thumb_func\n",
         "_nmi_handler:\n",
         "   MOV r0,#7\n",
         "   BX LR\n",
         "_entry_point:\n",
            "MOV r0,#0\n",
            "LDR r0,[r0,#0]\n",
            "MSR MSP, r0\n",
            "MSR PSP, r0\n",
            "MOV r0,#56\n",
            "ADD r0,#20\n",
         "_STACK_START:\n",
         "   .align 3\n",
         "   .fill _STACK_SIZE,1,0\n",
         "_SP_RESET_VAL:\n",
         "   .size _SP_RESET_VAL, . - _STACK_START\n"
      ).as_bytes()
   )?;

   let mut ld_script = File::create("./assembly_tests/basic_exc_load.ld")?;
   ld_script.write_all(
      format!(
         "ENTRY(_entry_point);\n\
         SECTIONS{{\n\
            \t. = 0x0;\n\
            \t.text : {{*(.text)}}\n\
         }}\n"
      ).as_bytes()
   )?;

   let linked = Path::new("./assembly_tests/exceptions.out");
   link_elf(&linked, &elf, Path::new("./assembly_tests/load.ld"));

   run_elf(&linked, |entry_point, code|{
      let mut sys = load_code_into_system(entry_point, code)?;
      //let word: [u8;4] = sys.memory[..4].try_into().unwrap();
      let word = sys.alloc.get::<4>(0);
      assert!(sys.read_raw_ir() != 0);
      let sp_reset = from_arm_bytes(word);
      //assert_eq!(sp_reset,sys.memory.len() as u32);
      let i = sys.step()?; //MOV r0,#0
      sys.offset_pc(i)?;
      assert_eq!(sys.get_ipsr(),0);

      let i = sys.step()?; //LDR r0,[r0,#0]
      assert_eq!(sys.registers.generic[0],sp_reset);
      sys.offset_pc(i)?;
      assert_eq!(sys.get_ipsr(),0);

      let i = sys.step()?; //MSR MSP,r0
      sys.offset_pc(i)?;
      assert_eq!(sys.registers.sp_main,sp_reset);
      assert_eq!(sys.get_ipsr(),0);

      let i = sys.step()?; //MSR PSP,r0
      sys.offset_pc(i)?;
      assert_eq!(sys.registers.sp_process,sp_reset);
      assert_eq!(sys.get_ipsr(),0);

      let i = sys.step()?; //MOV r0, #56
      assert_eq!(sys.registers.generic[0],56);
      assert_eq!(sys.get_ipsr(),0);

      sys.active_exceptions[2] = ExceptionStatus::Pending;
      sys.check_for_exceptions(i);
      assert_eq!(sys.get_ipsr(),2);
      assert!(matches!(sys.mode,Mode::Handler));

      let i = sys.step()?; //MOV r0,#7
      assert_eq!(sys.registers.generic[0],7);
      assert_eq!(sys.get_ipsr(),2);
      assert!(matches!(sys.mode,Mode::Handler));
      sys.offset_pc(i)?;

      let i = sys.step()?; //BX LR (exception return);
      sys.offset_pc(i)?;
      assert_eq!(sys.get_ipsr(),0);
      assert!(matches!(sys.mode,Mode::Thread));

      let i = sys.step()?; //ADD r0, #20 
      sys.offset_pc(i)?;
      assert_eq!(sys.registers.generic[0],76);
      assert_eq!(sys.get_ipsr(),0);
      assert!(matches!(sys.mode,Mode::Thread));
      Ok(())
   })?;
   Ok(())
}

#[test]
pub fn exception_preemption_test()->Result<(),std::io::Error>{
   let elf = write_asm_make_elf(
      "./assembly_tests/exception_preemption.s",
      concat!(
         ".thumb\n",
         ".text\n",
         ".equ _STACK_SIZE,0x80\n",
         ".global _entry_point\n",
         "_vector_table:\n",
         "   .4byte _SP_RESET_VAL\n",
         "   .4byte _dummy_handler\n",
         "   .4byte _nmi_handler\n",
         "   .4byte _dummy_handler\n",
         "   .4byte _dummy_handler\n",
         "   .4byte _dummy_handler\n",
         "   .4byte _dummy_handler\n",
         "   .4byte _dummy_handler\n",
         "   .4byte _dummy_handler\n",
         "   .4byte _dummy_handler\n",
         "   .4byte _dummy_handler\n",
         "   .4byte _svc_handler\n",
         ".thumb_func\n",
         "_nmi_handler:\n",
         "   MOV r0,#7\n",
         "   BX LR\n",
         ".thumb_func\n",
         "_dummy_handler:\n",
         "   BKPT\n",
         ".thumb_func\n",
         "_svc_handler:\n",
         "   MOV r0,#20\n",
         "   MOV r0,#42\n",
         "   BX LR\n",

         "_entry_point:\n",
            "MOV r0,#0\n",
            "LDR r0,[r0,#0]\n",
            "MSR MSP, r0\n",
            "MSR PSP, r0\n",
            "MOV r0,#56\n",
            "ADD r0,#81\n",
         "_STACK_START:\n",
         "   .align 3\n",
         "   .fill _STACK_SIZE,1,0\n",
         "_SP_RESET_VAL:\n",
         "   .size _SP_RESET_VAL, . - _STACK_START\n"
      ).as_bytes()
   )?;

   let mut ld_script = File::create("./assembly_tests/exception_preemption.ld")?;
   ld_script.write_all(
      format!(
         "ENTRY(_entry_point);\n\
         SECTIONS{{\n\
            \t. = 0x0;\n\
            \t.text : {{*(.text)}}\n\
         }}\n"
      ).as_bytes()
   )?;


   let linked = Path::new("./assembly_tests/exc_preemption.out");
   link_elf(&linked, &elf, Path::new("./assembly_tests/exception_preemption.ld"));

   run_elf(&linked, |entry_point, code|{
      let mut sys = load_code_into_system(entry_point, code)?;
      //let word: [u8;4] = sys.memory[..4].try_into().unwrap();
      let word = sys.alloc.get::<4>(0);
      assert!(sys.read_raw_ir() != 0);
      let sp_reset = from_arm_bytes(word);
      //assert_eq!(sp_reset,sys.memory.len() as u32);

      for _ in 0..4{
         let i = sys.step()?; 
         sys.offset_pc(i)?;
         assert_eq!(sys.get_ipsr(),0);
         assert!(matches!(sys.mode, Mode::Thread));
      }

         let i = sys.step()?; 
         assert_eq!(sys.registers.generic[0],56);
         println!("pc before {:#x}",sys.read_raw_ir());

         sys.active_exceptions[11] = ExceptionStatus::Pending;
         sys.check_for_exceptions(i);
         assert_eq!(sys.get_ipsr(),11);
         assert!(matches!(sys.active_exceptions[11],ExceptionStatus::Active));
         assert!(matches!(sys.mode, Mode::Handler));
         println!("pc is now {:#x}",sys.read_raw_ir());
         
         println!("started executing SVC");
         let i = sys.step()?;//MOV r0,#20
         assert_eq!(sys.registers.generic[0],20);
         assert_eq!(sys.get_ipsr(),11);
         assert!(matches!(sys.mode, Mode::Handler));

         
         sys.active_exceptions[2] = ExceptionStatus::Pending;
         sys.check_for_exceptions(i);
         assert!(matches!(sys.active_exceptions[2],ExceptionStatus::Active));
         assert_eq!(sys.get_ipsr(),2);
         assert!(matches!(sys.mode, Mode::Handler));
         let nmi_pc = sys.read_raw_ir();
         println!("nmi pc {:#x}",sys.read_raw_ir());
         println!(
            "@: {:#x} in mem value: {:#x},{:#x}",
            nmi_pc,
            //sys.memory[nmi_pc as usize],
            sys.alloc.get::<1>(nmi_pc)[0],
            //sys.memory[nmi_pc as usize + 1],
            sys.alloc.get::<1>(nmi_pc + 1)[0]
           );

         
         let i = sys.step()?; //MOV r0, #7
         println!("{:?}",sys.registers.generic);
         assert_eq!(sys.registers.generic[0],7);
         assert!(matches!(sys.active_exceptions[2],ExceptionStatus::Active));
         assert_eq!(sys.get_ipsr(),2);
         assert!(matches!(sys.mode, Mode::Handler));
         sys.offset_pc(i)?;
         
         let i = sys.step()?;//BX LR (Exception return)
         sys.offset_pc(i)?;
         assert!(matches!(sys.active_exceptions[2],ExceptionStatus::Inactive));
         assert!(matches!(sys.active_exceptions[11],ExceptionStatus::Active));
         assert_eq!(sys.get_ipsr(),11);
         assert!(matches!(sys.mode, Mode::Handler));

         let i = sys.step()?;//MOV r0,#42
         sys.offset_pc(i)?;
         assert_eq!(sys.registers.generic[0],42);
         assert!(matches!(sys.active_exceptions[11],ExceptionStatus::Active));
         assert_eq!(sys.get_ipsr(),11);
         assert!(matches!(sys.mode, Mode::Handler));

         let i = sys.step()?; //BX LR (Exception return)
         sys.offset_pc(i)?;
         assert!(matches!(sys.active_exceptions[11],ExceptionStatus::Inactive));
         assert_eq!(sys.get_ipsr(),0);
         assert!(matches!(sys.mode, Mode::Thread));

         let i = sys.step()?;
         sys.offset_pc(i)?;
         assert_eq!(sys.registers.generic[0],56+81);
         assert!(matches!(sys.active_exceptions[11],ExceptionStatus::Inactive));
         assert_eq!(sys.get_ipsr(),0);
         assert!(matches!(sys.mode, Mode::Thread));

      Ok(())
   })?;

   Ok(())
}

#[test]
pub fn vtor_support()->Result<(),std::io::Error>{
   let code = b".thumb
      .text
      LDR r0, =0xE000ED08
      MOV r1,#233
      STR r1,[r0,#0]
      MOV r2,#0
      MOV r3,#48
      LDR r3,[r0,r2]
      ";

   run_assembly(
      Path::new("sim_vtor.s"),
      code,
      |entry_point, code|{
         let mut sys = load_code_into_system(entry_point, code)?;
         assert_eq!(sys.scs.vtor,0);
         let i = sys.step()?;
         sys.offset_pc(i)?;

         let i = sys.step()?;
         sys.offset_pc(i)?;

         let i = sys.step()?;//writes to VTOR should be ignored
         sys.offset_pc(i)?;
         assert_eq!(sys.scs.vtor,0);

         let i = sys.step()?;
         sys.offset_pc(i)?;

         let i = sys.step()?;
         sys.offset_pc(i)?;
         assert_eq!(sys.registers.generic[3],48);

         let i = sys.step()?;
         sys.offset_pc(i)?;
         assert_eq!(sys.registers.generic[3],0);
         Ok(())
      }
   )?;
   return Ok(());
}

#[test]
pub fn ccr_test()->Result<(),std::io::Error>{
   let code = b".thumb
         .text
         LDR r0,=0xE000ED14
         LDR r1,[r0,#0]
         MOV r2,#1
         STR r2,[r0,#0]
         LDR r1,[r0,#0]
      ";

   run_assembly(
      Path::new("sim_ccr.s"),
      code,
      |entry_point,code|{
         let mut sys = load_code_into_system(entry_point, code)?;
         let i = sys.step()?;
         sys.offset_pc(i)?;

         let i = sys.step()?;
         sys.offset_pc(i)?;
         assert_eq!(sys.registers.generic[1],0x208);

         let i = sys.step()?;
         sys.offset_pc(i)?;

         let i = sys.step()?;
         sys.offset_pc(i)?;
         assert_ne!(sys.scs.ccr,1);

         let i = sys.step()?;
         sys.offset_pc(i)?;
         assert_eq!(sys.registers.generic[1],0x208,"CCR is readonly");
         Ok(())
      }
   )?;

   Ok(())
}

#[test]
pub fn software_interrupt_triggers()->Result<(),std::io::Error>{
   let code = b".thumb
         .text
         NOP
         LDR r0,=0xE000ED04
         LDR r1,=0x80000000
         STR r1,[r0,#0]
         LDR r1,=0x10000000
         STR r1,[r0,#0]
         LDR r1,=0x08000000
         STR r1,[r0,#0]
         LDR r1,=0x04000000
         STR r1,[r0,#0]
         LDR r1,=0x02000000
         STR r1,[r0,#0]
      ";

   run_assembly_armv6m(
      Path::new("sim_sw_int_pri.s"),
      code, 
      |entry_point, code|{
         let mut sys = load_code_into_system(entry_point, code)?;

         let i = sys.step()?;
         sys.offset_pc(i)?;
         for status in sys.active_exceptions{
            assert!(matches!(status,ExceptionStatus::Inactive));
         }

         let i = sys.step()?;
         sys.offset_pc(i)?;

         let i = sys.step()?;
         sys.offset_pc(i)?;

         let i = sys.step()?;
         sys.offset_pc(i)?;
         let n = ArmException::Nmi.number() as usize;
         assert!(matches!(sys.active_exceptions[n],ExceptionStatus::Pending));

         let i = sys.step()?;
         sys.offset_pc(i)?;

         let i = sys.step()?;
         sys.offset_pc(i)?;
         let n = ArmException::PendSV.number() as usize;
         assert!(matches!(sys.active_exceptions[n],ExceptionStatus::Pending));

         let i = sys.step()?;
         sys.offset_pc(i)?;

         let i = sys.step()?;
         sys.offset_pc(i)?;
         let n = ArmException::PendSV.number() as usize;
         assert!(matches!(sys.active_exceptions[n],ExceptionStatus::Inactive));

         let i = sys.step()?;
         sys.offset_pc(i)?;

         let i = sys.step()?;
         sys.offset_pc(i)?;
         let n = ArmException::SysTick.number() as usize;
         assert!(matches!(sys.active_exceptions[n],ExceptionStatus::Pending));

         let i = sys.step()?;
         sys.offset_pc(i)?;

         let i = sys.step()?;
         sys.offset_pc(i)?;
         let n = ArmException::SysTick.number() as usize;
         assert!(matches!(sys.active_exceptions[n],ExceptionStatus::Inactive));
         Ok(())
      }
   )?;
   Ok(())
}

#[test]
pub fn nvic_test()->Result<(),std::io::Error>{
   let code = b".thumb
         .text
         STR r1,[r0, #0]
         STR r1,[r0, #0]
         NOP
         STR r1,[r0, #0]
         STR r1,[r0, #0]
         STR r1,[r0, #0]
         STR r1,[r0, #0]
         STR r1,[r0, #0]
      ";

   run_assembly_armv6m(
      Path::new("sim_nvic.s"),
      code, 
      |entry_point, code|{
         let mut sys = load_code_into_system(entry_point, code)?;

         // should memory map NVIC_ISER register
         let nvic_iser: u32 = 0xE000E100;
         let enabled_interrupts: u32 = 0x80000000 | 0b110;
         sys.registers.generic[0] = nvic_iser;
         sys.registers.generic[1] = enabled_interrupts; 

         let i = sys.step()?;
         sys.offset_pc(i)?;

         assert_eq!(sys.scs.is_nvic_interrupt_enabled(1),true);
         assert_eq!(sys.scs.is_nvic_interrupt_enabled(2),true);
         assert_eq!(sys.scs.is_nvic_interrupt_enabled(31),true);
         assert_eq!(sys.scs.enabled_interrupts,enabled_interrupts);

         // disabled interrupts can be pending but cannot become active
         let nvic_ispr: u32 = 0xE000E200; 
         let req_pending = 1;
         sys.registers.generic[0] = nvic_ispr;
         sys.registers.generic[1] = req_pending;

         let i = sys.step()?;
         let _ = sys.check_for_exceptions(i);
         sys.offset_pc(i)?;

         assert!(matches!(sys.active_exceptions[ArmException::ExternInterrupt(16).number() as usize],ExceptionStatus::Pending));

         let i = sys.step()?;
         let _ = sys.check_for_exceptions(i);
         sys.offset_pc(i)?;
         assert!(matches!(sys.active_exceptions[ArmException::ExternInterrupt(16).number() as usize],ExceptionStatus::Pending));
         

         sys.active_exceptions[ArmException::ExternInterrupt(16).number() as usize] = ExceptionStatus::Inactive;

         //should allow enabled interrupts to become pending
         let pending = 0x80000002;
         sys.registers.generic[1] = pending;

         let i = sys.step()?;
         sys.offset_pc(i)?;

         assert!(matches!(sys.active_exceptions[16 + 31],ExceptionStatus::Pending));
         assert!(matches!(sys.active_exceptions[16 + 1],ExceptionStatus::Pending));

         //should allow pending enabled interrupts to be cleared
         let nvic_icpr = 0xE000E280;
         let to_clear = 0x80000000;
         sys.registers.generic[0] = nvic_icpr;
         sys.registers.generic[1] = to_clear;

         let i = sys.step()?;
         sys.offset_pc(i)?;

         assert!(matches!(sys.active_exceptions[16 + 31],ExceptionStatus::Inactive));

         //can disable interrupts
         let nvic_icer = 0xE000E180;
         let to_disable = 0b10;
         sys.registers.generic[0] = nvic_icer;
         sys.registers.generic[1] = to_disable;

         let i = sys.step()?;
         sys.offset_pc(i)?;

         assert_eq!(sys.scs.is_nvic_interrupt_enabled(1),false);
         assert_eq!(sys.scs.enabled_interrupts,0x80000004);

         //can change priorities
         let nvic_ipr7 = 0xE000E41C;
         let priorities = 0xCF8F4F0F;
         sys.registers.generic[0] = nvic_ipr7;
         sys.registers.generic[1] = priorities;

         let i = sys.step()?;
         sys.offset_pc(i)?;

         assert_eq!(sys.scs.nvic_priority_of(47),3);
         assert_eq!(sys.scs.nvic_priority_of(46),2);
         assert_eq!(sys.scs.nvic_priority_of(45),1);
         assert_eq!(sys.scs.nvic_priority_of(44),0);
         assert_eq!(sys.scs.ipr[7],0xC0804000);

         let nvic_ipr0 = 0xE000E400;
         let priorities = 0x40;
         sys.registers.generic[0] = nvic_ipr0;
         sys.registers.generic[1] = priorities;

         let i = sys.step()?;
         sys.offset_pc(i)?;

         assert_eq!(sys.scs.nvic_priority_of(16),1);
         assert_eq!(sys.scs.ipr[0],0x40);
         Ok(())
      }

   )?;
   Ok(())
}

#[test]
pub fn control_interrupt_priorities()->Result<(),std::io::Error>{
   let code = b".thumb
         .text
         NOP
         LDR r0,=0xE000ED1C
         LDR r1,=0x80000000
         STR r1,[r0,#0]
         LDR r0,=0xE000ED20
         LDR r1,=0x80800000
         STR r1,[r0,#0]
      ";
   run_assembly_armv6m(
      Path::new("sim_sw_int_pri.s"),
      code, 
      |entry_point, code|{
         let mut sys = load_code_into_system(entry_point, code)?;

         let i = sys.step()?;
         sys.offset_pc(i)?;
         assert_eq!(ArmException::Svc.priority_group(&sys.scs),0);
         assert_eq!(ArmException::SysTick.priority_group(&sys.scs),0);
         assert_eq!(ArmException::PendSV.priority_group(&sys.scs),0);

         let i = sys.step()?;
         sys.offset_pc(i)?;

         let i = sys.step()?;
         sys.offset_pc(i)?;

         let i = sys.step()?;
         sys.offset_pc(i)?;
         assert_eq!(ArmException::Svc.priority_group(&sys.scs),0b10);
         assert_eq!(sys.scs.shpr2 & 1 << 31,1 <<31);

         let i = sys.step()?;
         sys.offset_pc(i)?;

         let i = sys.step()?;
         sys.offset_pc(i)?;

         let i = sys.step()?;
         sys.offset_pc(i)?;
         assert_eq!(ArmException::SysTick.priority_group(&sys.scs),0b10);
         assert_eq!(ArmException::PendSV.priority_group(&sys.scs),0b10);
         assert_eq!(sys.scs.shpr3 & 1 << 31,1 << 31);
         assert_eq!(sys.scs.shpr3 & 1 << 23,1 << 23);

         

         
         sys.registers.sp_main = 1 << 15; // dummy SP
         sys.registers.sp_process = 1 << 15;// dummy SP
         sys.active_exceptions[ArmException::ExternInterrupt(16).number() as usize] = ExceptionStatus::Pending;
         sys.active_exceptions[ArmException::SysTick.number() as usize] = ExceptionStatus::Pending;
         sys.scs.enabled_interrupts = 1;

         let _ = sys.check_for_exceptions(i);

         println!("{:?}",sys.active_exceptions);
         assert!(matches!(sys.active_exceptions[ArmException::ExternInterrupt(16).number() as usize],ExceptionStatus::Active));
         assert!(matches!(sys.active_exceptions[ArmException::SysTick.number() as usize],ExceptionStatus::Pending));
         Ok(())
      }
   )?;

   Ok(())
}

#[test]
pub fn cps_test()->Result<(),std::io::Error>{
   let code = b"
      .thumb
      .text
      NOP 
      CPSID i 
      NOP
      CPSIE i

   ";

   run_assembly_armv6m(
      Path::new("sim_irq_de.s"),
      code, 
      |entry_point, code|{
         let mut sys = load_code_into_system(entry_point, code)?;
         sys.registers.sp_main = 1 << 15; // dummy SP
         sys.registers.sp_process = 1 << 15;// dummy SP
         let i = sys.step()?;
         sys.offset_pc(i)?;

         sys.active_exceptions[17] = ExceptionStatus::Pending;
         sys.scs.enabled_interrupts = 2;
         let i = sys.step()?;
         sys.check_for_exceptions(i);
         sys.offset_pc(i)?;
         assert!(matches!(sys.active_exceptions[17],ExceptionStatus::Pending));

         let i = sys.step()?;
         sys.check_for_exceptions(i);
         sys.offset_pc(i)?;
         assert!(matches!(sys.active_exceptions[17],ExceptionStatus::Pending));

         let i = sys.step()?;
         sys.check_for_exceptions(i);
         sys.offset_pc(i)?;
         assert!(matches!(sys.active_exceptions[17],ExceptionStatus::Active));
         Ok(())
      }
   )?;
   Ok(())
}

#[test] 
pub fn  euclid_gcd()->Result<(),std::io::Error>{
   assert_eq!(run_euclid(1, 2).unwrap(), (1,1));
   assert_eq!(run_euclid(4, 2).unwrap(), (2,2));
   assert_eq!(run_euclid(816, 2260).unwrap(), (4,4));
   assert_eq!(run_euclid(201, 450).unwrap(), (3,3));
   Ok(())
}

fn run_euclid(r0_value: u32, r1_value: u32)->Result<(u32,u32), std::io::Error>{
   let code = b".thumb
      .text
      _gcd:
         CMP r0,r1
         BEQ .end
         BLT .less
         SUB r0,r0,r1
         B _gcd
      .less:
         SUB r1,r1,r0
         B _gcd
      .end:
         NOP";
   let res =  run_assembly(
      Path::new("sim_euclid.s"),
      code, 
      |entry_point, code|{
         let mut sys = load_code_into_system(entry_point, code)?;
         sys.registers.generic[0] = r0_value;
         sys.registers.generic[1] = r1_value;

         while sys.registers.pc < code.len(){
            let incr = sys.step()?;
            sys.offset_pc(incr)?;
         }
         return Ok((sys.registers.generic[0], sys.registers.generic[1]));
      }
   )?;

   return Ok(res);
}

#[test]
pub fn fibonnaci()->Result<(),std::io::Error>{
   assert_eq!(run_fibonnaci(0).unwrap(),1);
   assert_eq!(run_fibonnaci(1).unwrap(),2);
   assert_eq!(run_fibonnaci(2).unwrap(),3);
   assert_eq!(run_fibonnaci(3).unwrap(),5);
   assert_eq!(run_fibonnaci(4).unwrap(),8);
   assert_eq!(run_fibonnaci(5).unwrap(),13);
   assert_eq!(run_fibonnaci(6).unwrap(),21);
   Ok(())
}

fn run_fibonnaci(nth_term: u32)->Result<u32,std::io::Error>{
   let bin_size = 1024;
   let code = 
   b"
   .thumb
   .text
      LDR r1, =1024
      MSR MSP,r1
      MOV r1,#0
      MOV r2,#1
      PUSH {r0,r1,r2}
      BL _fib
      B _done
         .pool
      _fib:
         POP {r0,r1,r2}
         ADD r3,r1,r2
         CMP r0,#0
         BHI _recurse
         BX LR
         _recurse:
            SUB r0,#1
            PUSH {r0,r2,r3}
            B _fib
      _done:
         NOP
   ";

   let res =  run_assembly_armv6m(
      Path::new("sim_fibonnaci.s"),
      code, 
      |entry_point, code|{
         let mut sys = load_code_into_system(entry_point, code)?;
         //sys.expand_memory_to(bin_size);
         sys.registers.generic[0] = nth_term;

         while sys.registers.pc < code.len(){
            let incr = sys.step()?;
            sys.offset_pc(incr)?;
         }
         return Ok(sys.registers.generic[3]);
      }
   )?;
   return Ok(res);
}

#[test]
pub fn should_load()->Result<(),std::io::Error>{
   let code = 
   b".thumb
   .text
      LDR r0, =_some_var 
      LDR r1,[r0]
      LDRH r0, =_some_hw
      LDR  r1, [r0]
      NOP
      NOP
      NOP
      .pool
      _some_var: .word 0xBEEF
      _some_hw: .2byte 0x10AA
   ";
   run_assembly(
      &Path::new("sim_load.s"),
      code,
      |entry_point, binary|{
         let mut sys = load_code_into_system(entry_point, binary)?;
         //println!("mem: [{:?}]",sys.memory);
         let mut off = sys.step()?;
         let beef_ptr = sys.registers.generic[0]; 
         let word = sys.alloc.get::<4>(beef_ptr);
         assert_eq!(0xBEEF_u32,from_arm_bytes(word));
         sys.offset_pc(off)?;

         off = sys.step()?;
         assert_eq!(sys.registers.generic[1],0xBEEF_u32);
         sys.offset_pc(off)?;

         off = sys.step()?; 
         let hw_ptr = sys.registers.generic[0];
         let hw = sys.alloc.get::<2>(hw_ptr);
         assert_eq!(0x10AA_u16,from_arm_bytes_16b(hw));
         sys.offset_pc(off)?;

         off = sys.step()?;
         assert_eq!(sys.registers.generic[1] as u16,0x10AA_u16);



         return Ok(());
      }
   )?;
   return Ok(());
}

#[test]
pub fn should_store()->Result<(), std::io::Error>{
   let code = 
      b".thumb
      .text
         LDR r0, =_some_var
         MOV r1, #240
         STR r1, [r0, #0]
         NOP
         _some_var: .word 0xCCCC
   ";

   run_assembly(
      Path::new("sim_store.s"),
      code,
      |entry_point, binary|{
         let mut sys = load_code_into_system(entry_point, binary)?;
         let mut i = sys.step()?;
         let ptr = sys.registers.generic[0] as usize;
         sys.offset_pc(i)?;
         i = sys.step()?;
         sys.offset_pc(i)?;
         i = sys.step()?;
         sys.offset_pc(i)?;

         //let written_word: [u8;4] = sys.memory[ptr .. ptr + 4].try_into().unwrap();
         let written_word = sys.alloc.get::<4>(ptr as u32);
         assert_eq!(240, from_arm_bytes(written_word));

         return Ok(());
      }
   )?;

   return Ok(());
}

pub fn load_code_with_sections<P: AsRef<Path>>(elf: P)->Result<(System, Vec<SymbolDefinition>),ElfError>{

   let (elf_header,mut reader) = get_header(elf.as_ref())?;

   let section_headers = get_all_section_headers(&mut reader, &elf_header)?;
   assert!(!section_headers.is_empty());

   let strtab_idx = get_string_table_section_hdr(&elf_header, &section_headers).unwrap();
   let str_table_hdr = &section_headers[strtab_idx];

   let maybe_symtab: Vec<&SectionHeader> = section_headers.iter()
      .filter(|hdr| is_symbol_table_section_hdr(&elf_header, hdr))
      .collect();

   let sym_entries = get_section_symbols(&mut reader, &elf_header, &maybe_symtab[0]).unwrap();
   let symbols = get_all_symbol_names(&mut reader, &elf_header, &sym_entries, str_table_hdr).unwrap();
   let loadable = get_loadable_sections(&mut reader, &elf_header,&section_headers)?;

   let section_data = load_sections(&mut reader, &elf_header, &section_headers, loadable)?;

   return Ok((System::with_sections(section_data),symbols));
}

fn copy_inital_state(sys: &mut System, states: &Vec<u32>){
   let initial_state: [u32;PROC_VARIABLES] = states.chunks_exact(PROC_VARIABLES)
      .next()
      .unwrap()
      .try_into()
      .expect("should be 18 variables");

   sys.registers.generic[0] = initial_state[super::R0];
   sys.registers.generic[1] = initial_state[super::R1];
   sys.registers.generic[2] = initial_state[super::R2];
   sys.registers.generic[3] = initial_state[super::R3];
   sys.registers.generic[4] = initial_state[super::R4];
   sys.registers.generic[5] = initial_state[super::R5];
   sys.registers.generic[6] = initial_state[super::R6];
   sys.registers.generic[7] = initial_state[super::R7];
   sys.registers.generic[8] = initial_state[super::R8];
   sys.registers.generic[9] = initial_state[super::R9];
   sys.registers.generic[10] = initial_state[super::R10];
   sys.registers.generic[11] = initial_state[super::R11];
   sys.registers.generic[12] = initial_state[super::R12];
   sys.registers.sp_main = initial_state[super::MSP];
   sys.registers.sp_process = initial_state[super::PSP];
   sys.registers.lr = initial_state[super::LR];
   sys.registers.pc = initial_state[super::PC] as usize;
   let xpsr_bytes = into_arm_bytes(initial_state[super::XPSR]); 
   sys.xpsr = xpsr_bytes;
}

fn assert_states_match(sys: &System, state: &[u32; PROC_VARIABLES]){
   assert_eq!(sys.registers.generic[0],state[super::R0]);
   assert_eq!(sys.registers.generic[1],state[super::R1]);
   assert_eq!(sys.registers.generic[2],state[super::R2]);
   assert_eq!(sys.registers.generic[3],state[super::R3]);
   assert_eq!(sys.registers.generic[4],state[super::R4]);
   assert_eq!(sys.registers.generic[5],state[super::R5]);
   assert_eq!(sys.registers.generic[6],state[super::R6]);
   assert_eq!(sys.registers.generic[7],state[super::R7]);
   assert_eq!(sys.registers.generic[8],state[super::R8]);
   assert_eq!(sys.registers.generic[9],state[super::R9]);
   assert_eq!(sys.registers.generic[10],state[super::R10]);
   assert_eq!(sys.registers.generic[11],state[super::R11]);
   assert_eq!(sys.registers.generic[12],state[super::R12]);
   assert_eq!(sys.registers.sp_main,state[super::MSP]);
   assert_eq!(sys.registers.sp_process,state[super::PSP]);
   assert_eq!(sys.registers.lr,state[super::LR]);
   assert_eq!(sys.registers.pc,state[super::PC] as usize);
   assert_eq!(sys.xpsr,into_arm_bytes(state[super::XPSR]));
}
macro_rules! fail_log {
    ($case:expr,$($msg:tt)*) => {
       if !($case){
          println!($($msg)*);
       }
    }
}

fn are_states_equal(sys: &System, state: &[u32; PROC_VARIABLES])->bool{

   fail_log!(
      sys.registers.generic[0] == state[super::R0],
      "r0 doesnt match"
   );

   fail_log!(
      sys.registers.generic[1] == state[super::R1],
      "r1 doesnt match"
   );

   fail_log!(
      sys.registers.generic[2] == state[super::R2],
      "r2 doesnt match"
   );

   fail_log!(
      sys.registers.generic[3] == state[super::R3],
      "r3 doesnt match"
   );
   
   fail_log!(
      sys.registers.generic[4] == state[super::R4],
      "r4 doesnt match"
   );

   fail_log!(
      sys.registers.generic[5] == state[super::R5],
      "r5 doesnt match"
   );

   fail_log!(
      sys.registers.generic[6] == state[super::R6],
      "r6 doesnt match"
   );

   fail_log!(
      sys.registers.generic[7] == state[super::R7],
      "r7 doesnt match"
   );

   fail_log!(
      sys.registers.generic[8] == state[super::R8],
      "r8 doesnt match"
   );

   fail_log!(
      sys.registers.generic[9] == state[super::R9],
      "r9 doesnt match"
   );

   fail_log!(
      sys.registers.generic[10] == state[super::R10],
      "r10 doesnt match"
   );

   fail_log!(
      sys.registers.generic[11] == state[super::R11],
      "r11 doesnt match"
   );

   fail_log!(
      sys.registers.generic[12] == state[super::R12],
      "r12 doesnt match"
   );

   fail_log!(
      sys.registers.sp_main == state[super::MSP],
      "MSP doesnt match"
   );

   fail_log!(
      sys.registers.sp_process == state[super::PSP],
      "PSP doesnt match"
   );

   fail_log!(
      sys.registers.lr == state[super::LR],
      "LR doesnt match"
   );

   fail_log!(
      (sys.registers.pc as u32) == state[super::PC],
      "PC doesnt match"
   );

   fail_log!(
      sys.xpsr == into_arm_bytes(state[super::XPSR]),
      "XPSR doesnt match {:#x}(sim) != {:#x}(hw)",
      from_arm_bytes(sys.xpsr),
      state[super::XPSR]
   );

   return (sys.registers.generic[0] == state[super::R0]) && 
      (sys.registers.generic[1] == state[super::R1]) &&
      (sys.registers.generic[2] == state[super::R2]) &&
      (sys.registers.generic[3] == state[super::R3]) &&
      (sys.registers.generic[4] == state[super::R4]) &&
      (sys.registers.generic[5] == state[super::R5]) &&
      (sys.registers.generic[6] == state[super::R6]) &&
      (sys.registers.generic[7] == state[super::R7]) && 
      (sys.registers.generic[8] == state[super::R8]) &&
      (sys.registers.generic[9] == state[super::R9]) &&
      (sys.registers.generic[10] == state[super::R10]) &&
      (sys.registers.generic[11] == state[super::R11]) &&
      (sys.registers.generic[12] == state[super::R12]) &&
      (sys.registers.sp_main == state[super::MSP]) &&
      (sys.registers.sp_process == state[super::PSP]) &&
      (sys.registers.lr == state[super::LR]) &&
      (sys.registers.pc == state[super::PC] as usize) &&
      (sys.xpsr == into_arm_bytes(state[super::XPSR]));
}


fn step(sys: &mut System ){
   match sys.step(){
      Ok(offset) => {
         if sys.check_for_exceptions(offset).is_none(){
            match sys.offset_pc(offset){
                Ok(_) => {},
                Err(e) => {

                   let offset = match e{
                      ArmException::Svc => 2,
                      _ => 0
                   };
                   sys.set_exc_pending(e);
                   let _ = sys.check_for_exceptions(offset);
                },
            }
         }
      },
      Err(e)=>{
         let offset = match e{
             ArmException::Svc => 2,
             _ => 0
         };
         sys.set_exc_pending(e);
         let _ = sys.check_for_exceptions(offset);
      }
   }
}

#[test] #[ignore] 
pub fn hardware_linear_search(){
   let label = String::from("linear_search");
   let script = gdb_script(&label);
   println!("script generated:(\n{})",&script);
   std::fs::write("dump_proc_state_linear_search", &script).unwrap();
   let output = run_script_on_remote_cpu(
      "dump_proc_state_linear_search".into(), 
      "elf_samples/linear_search.elf".into()
   );

   println!("{}",&output);

   let states = parse_gdb_output(&output);
   print_states(states);
   std::fs::remove_file("dump_proc_state_linear_search").unwrap();
   panic!("want to see logs");
}

#[test] #[ignore]
pub fn hardware_fibonacci()->Result<(),ElfError>{
   let label = String::from("fibonacci");
   let script = gdb_script(&label);
   println!("generated:\n {}",&script);
   std::fs::write("elf_samples/fib/dump_proc",&script).unwrap();

   let output = run_script_on_remote_cpu(
      "elf_samples/fib/dump_proc", 
      "elf_samples/fib/fibonacci.elf"
   );

   let states = parse_gdb_output(&output);

   let (mut sys,_) = load_code_with_sections("elf_samples/fib/fibonacci.elf")?;

   copy_inital_state(&mut sys, &states);
   for state in states.chunks_exact(PROC_VARIABLES){
      let cpu_state: &[u32; PROC_VARIABLES] = state
         .try_into()
         .expect("should be 18 registers");
      assert_states_match(&sys, cpu_state);
      step(&mut sys);
   }
   Ok(())
}

fn update_fuzzy_stats(total_tests: u32, total_fails: u32){
   let stats_path = Path::new("elf_samples/fuzzy/stats");
   let data = format!("total_tests: {}\nfails: {}\n",total_tests,total_fails);
   let mut handle =  File::create(stats_path).unwrap();

   handle.write_all(data.as_bytes()).unwrap();
}

#[test] #[ignore]
fn fuzzy_testsuite()->Result<(),ElfError>{
   let bin_path = Path::new("elf_samples/fuzzy/build/fuzzy.elf");

   let generate_new_case = true;
   let label = String::from("sim_testcase_init");
   //let script = gdb_script(&label);
   //println!("generated:\n {}",&script);
   //std::fs::write("elf_samples/fuzzy/dump_proc",&script).unwrap();

   let stats_path = Path::new("elf_samples/fuzzy/stats");
   let current_stats = std::fs::read_to_string(stats_path)?;
   let mut rdr = current_stats.lines();
   let first_line = rdr.next().unwrap();
   let second_line = rdr.next().unwrap();

   let n_tests = first_line.strip_prefix("total_tests:").unwrap().trim();
   let n_fails = second_line.strip_prefix("fails:").unwrap().trim(); 

   let mut test_count = u32::from_str_radix(n_tests, 10).unwrap();
   let mut fail_count = u32::from_str_radix(n_fails, 10).unwrap();
   std::mem::drop(rdr);

   for _ in 0 .. 1{
      if generate_new_case{
         create_fuzzy_test(bin_path)?;
         test_count += 1;
      }

      let output = run_script_on_remote_cpu(
         "elf_samples/fuzzy/dump_proc",
         "elf_samples/fuzzy/build/fuzzy.elf"
      );

      let states = parse_gdb_output(&output);
      let mut stages = output.split("<<-->>");
      let (mut sys,_) = load_code_with_sections("elf_samples/fuzzy/build/fuzzy.elf")?;
      println!("hardware result {:?}",states);
      copy_inital_state(&mut sys, &states);
      //SPOOF VTOR value so it points to the ram table embeded by the pico SDK
      sys.set_vtor(0x10000100);
      
      for state in states.chunks_exact(PROC_VARIABLES){
         println!("{}",stages.next().unwrap());
         println!("executing on {:#x}",sys.read_raw_ir());
         let cpu_state: &[u32; PROC_VARIABLES] = state
            .try_into()
            .expect("should be 18 registers");
         if !are_states_equal(&sys, cpu_state){
            println!("ERROR: State inconsistency");
            println!("real hardware state");
            println!("{:?}",cpu_state);
            println!("simulator hardware state");
            println!("{:?}",vec![
               sys.registers.generic[0],
               sys.registers.generic[1],
               sys.registers.generic[2],
               sys.registers.generic[3],
               sys.registers.generic[4],
               sys.registers.generic[5],
               sys.registers.generic[6],
               sys.registers.generic[7],
               sys.registers.generic[8],
               sys.registers.generic[9],
               sys.registers.generic[10],
               sys.registers.generic[11],
               sys.registers.generic[12],
               sys.registers.sp_main,
               sys.registers.sp_process,
               sys.registers.lr,
               sys.registers.pc as u32,
               from_arm_bytes(sys.xpsr)
            ]);
            std::fs::write("elf_samples/fuzzy/failed/hardware_run",&output)?;
            fail_count += 1;
            if generate_new_case{
               update_fuzzy_stats(test_count, fail_count);
            }
            panic!("state inconsistency");
         }

         step(&mut sys);
      }
   }


   if generate_new_case{
      update_fuzzy_stats(test_count, fail_count);
   }
   Ok(())
}

fn create_fuzzy_test<P: AsRef<Path>>(bin_path: P)->Result<(),ElfError>{
   use std::io::Seek;
   use rand::prelude::*;

   let (elf_header, mut reader) = get_header(bin_path.as_ref())?;

   let section_headers = get_all_section_headers(&mut reader, &elf_header)?;

   let strtab_idx = get_string_table_section_hdr(&elf_header, &section_headers).unwrap();
   let str_table_hdr = &section_headers[strtab_idx];

   let maybe_symtab: Vec<&SectionHeader> = section_headers.iter()
      .filter(|hdr| is_symbol_table_section_hdr(&elf_header, hdr))
      .collect();

   let sym_entries = get_section_symbols(&mut reader, &elf_header, &maybe_symtab[0])
      .unwrap();

   let symbols = get_all_symbol_names(
      &mut reader,
      &elf_header,
      &sym_entries,
      str_table_hdr
   ).unwrap();

   
   let pool_label = symbols.iter().position(|sym| sym.name.eq("sim_testcase_pool"));
   assert!(pool_label.is_some());
   let sections = get_loadable_sections(&mut reader, &elf_header, &section_headers)?; 
   let t = sections.iter().position(|s| s.name.eq(".text"));

   let text_hdr = section_headers.iter().position(|hdr| {
      let name = to_native_endianness_32b(&elf_header, &hdr.name);
      name == sections[t.unwrap()].name_idx
   });

   let text_offset = sections[t.unwrap()].start;
   let relative_offset_in_section = symbols[pool_label.unwrap()].position as u32 - text_offset;
   let text_data_file_offset = to_native_endianness_32b(&elf_header,&section_headers[text_hdr.unwrap()].offset_of_entries_in_bytes);
   let abs_offset_in_file = text_data_file_offset + relative_offset_in_section;
   let first_word = if (abs_offset_in_file & !3) == abs_offset_in_file{
      abs_offset_in_file
   }else{
      (abs_offset_in_file + 4) & !3
   };

   println!(" first word of pool @{:#x} ({:#x})",first_word,first_word + text_offset);
   let mut rng = rand::thread_rng();
   let random_state: Vec<u8> = vec![
      u32_to_arm_bytes(rng.gen()),
      u32_to_arm_bytes(rng.gen()),
      u32_to_arm_bytes(rng.gen()),
      u32_to_arm_bytes(rng.gen()),
      u32_to_arm_bytes(rng.gen()),
      u32_to_arm_bytes(rng.gen()),
      u32_to_arm_bytes(rng.gen()),
      u32_to_arm_bytes(rng.gen())
   ].into_iter().flatten().collect();

   assert_eq!(random_state.len(),8 * 4);

   std::mem::drop(reader);
   use std::fs::OpenOptions;
   println!("injecting random data to 'sim_testcase_pool' literal pool");
   let mut writer = OpenOptions::new().write(true).open(bin_path)?;
   println!("writing {:?} to @ {:#x}(file offset)",random_state,first_word);
   writer.seek(SeekFrom::Start(first_word as u64))?;
   writer.write(&random_state)?;

   println!("injecting random data to 'sim_testcode_placeholder' ");
   let testcode_label = symbols.iter().position(|sym| sym.name.eq("sim_testcode_placeholder"));
   assert!(testcode_label.is_some());
   let relative_offset_in_section = symbols[testcode_label.unwrap()].position as u32 - text_offset;
   let abs_offset_in_file = text_data_file_offset + relative_offset_in_section;
   assert_eq!(abs_offset_in_file % 4,0);

   writer.seek(SeekFrom::Start(abs_offset_in_file as u64))?;
   let maybe_instruction: [u8;4] = u32_to_arm_bytes(rng.gen());
   println!("writing to {:?} to @ {:#x}(file offset)",maybe_instruction,abs_offset_in_file);
   writer.write(&maybe_instruction)?;
   return Ok(());
}
