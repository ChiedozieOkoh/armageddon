
use std::any::Any;
use std::fs::File;
use std::io::Write;
use std::io::{Error,ErrorKind};
use std::path::Path;

use crate::binutils::from_arm_bytes;
use crate::system::instructions::{zero_flag, negative_flag, carry_flag, overflow_flag};
use crate::system::registers::{get_overflow_bit, get_carry_bit};
use crate::tests::asm::{write_asm, asm_file_to_elf, asm_file_to_elf_armv6m};
use crate::tests::elf::{write_asm_make_elf, link_elf};
use crate::tests::system::{run_script_on_remote_cpu, parse_gdb_output, print_states};
use crate::system::{ArmException, System, ExceptionStatus, Mode};
use crate::elf::decoder::to_native_endianness_32b;
use super::gdb_script;

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
   let mut sys = System::create(0);
   for c in code{
      sys.memory.push(*c);
   }
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
         println!("memory: {:?}",sys.memory);
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

         sys.step()?;

         assert_eq!(sys.registers.generic[0], 21);
         assert!(!negative_flag(sys.xpsr));
         assert!(!zero_flag(sys.xpsr));
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
         sys.expand_memory_to(bin_size);
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
         sys.expand_memory_to(bin_size);
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
      let word: [u8;4] = sys.memory[..4].try_into().unwrap();
      assert!(sys.read_raw_ir() != 0);
      let sp_reset = from_arm_bytes(word);
      assert_eq!(sp_reset,sys.memory.len() as u32);
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

      sys.step()?; //MOV r0, #56
      assert_eq!(sys.registers.generic[0],56);
      assert_eq!(sys.get_ipsr(),0);

      sys.active_exceptions[2] = ExceptionStatus::Pending;
      sys.check_for_exceptions();
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
      let word: [u8;4] = sys.memory[..4].try_into().unwrap();
      assert!(sys.read_raw_ir() != 0);
      let sp_reset = from_arm_bytes(word);
      assert_eq!(sp_reset,sys.memory.len() as u32);

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
         sys.check_for_exceptions();
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
         sys.check_for_exceptions();
         assert!(matches!(sys.active_exceptions[2],ExceptionStatus::Active));
         assert_eq!(sys.get_ipsr(),2);
         assert!(matches!(sys.mode, Mode::Handler));
         let nmi_pc = sys.read_raw_ir();
         println!("nmi pc {:#x}",sys.read_raw_ir());
         println!(
            "@: {:#x} in mem value: {:#x},{:#x}",
            nmi_pc,
            sys.memory[nmi_pc as usize],
            sys.memory[nmi_pc as usize + 1],
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
pub fn can_disable_interrupts()->Result<(),std::io::Error>{
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
         sys.expand_memory_to(bin_size);
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
      NOP
      NOP
      NOP
      .pool
      _some_var: .word 0xBEEF
   ";
   run_assembly(
      &Path::new("sim_load.s"),
      code,
      |entry_point, binary|{
         let mut sys = load_code_into_system(entry_point, binary)?;
         println!("mem: [{:?}]",sys.memory);
         let off = sys.step()?;
         let beef_ptr = sys.registers.generic[0]; 

         let word: [u8;4] = sys.memory[beef_ptr as usize .. beef_ptr as usize + 4]
            .try_into()
            .unwrap();
         assert_eq!(0xBEEF_u32,from_arm_bytes(word));

         sys.offset_pc(off)?;
         sys.step()?;

         assert_eq!(sys.registers.generic[1],0xBEEF_u32);
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

         let written_word: [u8;4] = sys.memory[ptr .. ptr + 4].try_into().unwrap();
         assert_eq!(240, from_arm_bytes(written_word));

         return Ok(());
      }
   )?;

   return Ok(());
}

#[test] #[ignore] 
pub fn hardware_linear_search(){
   let label = String::from("linear_search");
   let script = gdb_script(&label, 12);
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


