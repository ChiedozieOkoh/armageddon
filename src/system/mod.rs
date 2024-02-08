use std::fmt::Display;

use crate::asm::{self, PROGRAM_COUNTER, DestRegister, SrcRegister, Literal};
use crate::binutils::{from_arm_bytes, clear_bit, set_bit, into_arm_bytes, get_set_bits};
use crate::asm::decode::{Opcode, instruction_size, InstructionSize, B16, B32};
use crate::asm::decode_operands::{Operands,get_operands, get_operands_32b};
use crate::dbg_ln;
use crate::system::instructions::{add_immediate,ConditionFlags,compare,subtract,multiply, xor} ;

use self::instructions::{cond_passed, shift_left, shift_right};
use self::registers::{Registers, Apsr, SpecialRegister, get_overflow_bit};

pub mod registers;
pub mod instructions;
pub mod simulator;

pub const TRACED_VARIABLES: usize = 8;

pub struct System{
   pub registers: Registers,
   pub xpsr: Apsr,
   pub control_register: [u8;4],
   pub event_register: bool,
   pub active_exceptions: [ExceptionStatus;48],
   pub scs: SystemControlSpace,
   pub mode: Mode,
   primask: bool,
   pending_primask: Option<bool>,
   pub memory: Vec<u8>,
   pub breakpoints: Vec<usize>
}

#[derive(Debug)]
pub enum Mode{
   Thread,
   Handler
}

const IPSR_MASK: u32 = 0xFFFFFFC0;
const EXC_RETURN_TO_HANDLER: u32 = 1;
const EXC_RETURN_TO_THREAD_MSP: u32 = 9;
const EXC_RETURN_TO_THREAD_PSP: u32 = 0xD;

macro_rules! unpack_operands {
    ($variant:expr, $_type:path, $($vari:ident),+) => {
       if let Some($_type($($vari),+)) = $variant{
          ($($vari),+)
       }else{
          panic!("failed to decode {} operands",stringify!($_type));
       }
    }
}

macro_rules! conditional_branches{
   () => {
      Opcode::_16Bit(B16::BEQ) 
         | Opcode::_16Bit(B16::BNEQ) 
         | Opcode::_16Bit(B16::B_CARRY_IS_SET)
         | Opcode::_16Bit(B16::B_CARRY_IS_CLEAR)
         | Opcode::_16Bit(B16::B_IF_NEGATIVE)
         | Opcode::_16Bit(B16::B_IF_POSITIVE)
         | Opcode::_16Bit(B16::B_IF_OVERFLOW)
         | Opcode::_16Bit(B16::B_IF_NO_OVERFLOW)
         | Opcode::_16Bit(B16::B_UNSIGNED_HIGHER)
         | Opcode::_16Bit(B16::B_UNSIGNED_LOWER_OR_SAME)
         | Opcode::_16Bit(B16::B_GTE)
         | Opcode::_16Bit(B16::B_LTE)
         | Opcode::_16Bit(B16::B_GT)
         | Opcode::_16Bit(B16::B_LT)
   }
}

type ProcessStackFrame = [u32;8];

impl System{
   pub fn create(capacity: usize)->Self{
      let registers = Registers::create();
      return System{
         registers,
         xpsr: [0;4],
         control_register: [0;4],
         event_register: false,
         active_exceptions: [ExceptionStatus::Inactive; 48],
         primask: false,
         pending_primask: None,
         scs: SystemControlSpace::reset(),
         mode: Mode::Thread, // when not in a exception the processor is in thread mode
         memory: vec![0;capacity],
         breakpoints: Vec::new(),
      }
   }

   pub fn create_from_text(text: Vec<u8>)->Self{
      let registers = Registers::create();
      return System{
         registers,
         xpsr: [0;4],
         control_register: [0;4],
         event_register: false,
         active_exceptions: [ExceptionStatus::Inactive; 48],
         scs: SystemControlSpace::reset(),
         primask: false,
         pending_primask: None,
         mode: Mode::Thread, // when not in a exception the processor is in thread mode
         memory: text,
         breakpoints: Vec::new(),
      }
   }

   pub fn expand_memory_to(&mut self, new_size: usize ){
      if new_size > self.memory.len(){
         self.memory.resize(new_size, 0);
      }
   }

   fn update_apsr(&mut self, flags: &ConditionFlags){
      let mut xpsr = from_arm_bytes(self.xpsr);
      xpsr = if flags.negative{ set_bit(31,xpsr)} else {clear_bit(31,xpsr)};
      xpsr = if flags.zero{set_bit(30,xpsr)} else {clear_bit(30,xpsr)};
      xpsr = if flags.carry{set_bit(29,xpsr)} else {clear_bit(29,xpsr)};
      xpsr = if flags.overflow{set_bit(28,xpsr)} else {clear_bit(28,xpsr)};

      self.xpsr = into_arm_bytes(xpsr);
   }

   fn epsr_t_bit(&self)-> bool{
      let xpsr = from_arm_bytes(self.xpsr);
      (xpsr & (1 << 24)) != 0
   }

   fn set_epsr_t_bit(&mut self, v: bool){
      let mut xpsr_val = from_arm_bytes(self.xpsr);
      xpsr_val = if v{
         xpsr_val | (1 << 24)
      }else {
         xpsr_val & !(1 << 24)
      };

      self.xpsr = into_arm_bytes(xpsr_val);
   }

   fn sp_select_bit(&self) -> bool{
      from_arm_bytes(self.control_register) & 0x02 > 0
   }

   fn set_sp_select_bit(&mut self, bit: bool){
      let mut sp_sel = from_arm_bytes(self.control_register);
      sp_sel = if bit{
         sp_sel | 0x2
      }else{
         sp_sel & (!(0x2))
      };
      self.control_register = into_arm_bytes(sp_sel);
   }

   pub fn get_sp(&self)->u32{
      if self.sp_select_bit(){
         match self.mode{
            Mode::Thread => self.registers.sp_process,
            Mode::Handler => panic!("process SP is unavailable while in handler mode"),
         }
      }else{
         self.registers.sp_main
      }
   }

   pub fn get_sp_frame_alignment(&self)->bool{
      self.get_sp() & 0x4 > 0
   }

   fn set_sp(&mut self, v: u32)->Result<(), ArmException>{
      fault_if_not_aligned(v, 4)?;
      if self.sp_select_bit(){
         match self.mode{
            Mode::Thread => {
               println!("set process SP");
               self.registers.sp_process = v; 
               return Ok(());
            },
            Mode::Handler => panic!("process SP is unavailable while in handler mode"),
         }
      }else{
         println!("set main SP");
         self.registers.sp_main = v;
         return Ok(());
      }
   }

   fn read_any_register(&self, register: u8)-> u32{
      match register{
         0 ..=12 => self.registers.generic[register as usize] ,
         13 => self.get_sp(),
         14 => self.registers.lr,
         15 => self.read_pc_word_aligned(),
         _ => panic!("r{} is not a known register, is there a decoding error?",register)
      }
   }

   pub fn set_pc(&mut self, addr: usize)->Result<(),ArmException>{
      if addr > u32::MAX as usize{
         return Err(ArmException::HardFault(format!("tried to set PC to ({}), value is unrepresentable by 32bits",addr)));
      }
      if !is_aligned(addr as u32, 2){
         return Err(ArmException::HardFault(format!("{} is not 2 byte aligned",addr)));
      }
      self.registers.pc = addr as usize;
      return Ok(());
   }

   pub fn read_raw_ir(&self)->u32{
      self.registers.pc as u32
   }

   pub fn read_pc_word_aligned(&self)->u32{
      println!("(wrd algin : {} + {} = {} ) & {:04b}",
               self.registers.pc,4,
               ((self.registers.pc + 4 ) as u32) & 0xFFFFFFFC,0xFFFFFFFC_u32);
      return ((self.registers.pc + 4) as u32 ) & 0xFFFFFFFC;
   }

   pub fn offset_pc(&mut self, offset: i32 )->Result<(),ArmException>{
      let new_addr = Self::offset_read_pc(self.registers.pc as u32,offset)?;
      println!("pc {0}({0:x}) -> {1}({1:x})",self.registers.pc,new_addr);
      self.registers.pc = new_addr as usize;
      return Ok(());
   }

   fn bx_interworking_pc_offset(&mut self, addr: u32)->Result<i32, ArmException>{
      if (addr & 0xF0000000 == 0xF0000000) && matches!(self.mode, Mode::Handler){
         match self.exception_return(addr){
            Ok(exc_n) => {
               dbg_ln!("returned from exception {}",exc_n);
               return Ok(0);
            }
            Err(e)=>{
               println!("ERR: {:?} occured during exception return",e);
               self.lockup();
               return Err(e);
            }
         }
      }else{
         let bit = (addr & 0x1) != 0;
         self.set_epsr_t_bit(bit);
         if bit == false{
            return Err(
               ArmException::HardFault(
                  format!(
                     "EPSR.T bit set to 0, addr {} is not interworking but should be",
                     addr
               ))
            );
         }
         return Ok(((addr & 0xFFFFFFFE_u32) as i32) - (self.registers.pc as i32));
      }
   }

   fn in_privileged_mode(&self)->bool{
      match self.mode{
         Mode::Handler => {
            true
         },
         Mode::Thread =>{
            (from_arm_bytes(self.control_register) & 0x1) == 0
         }
      }
   }

   fn offset_read_pc(pc: u32, offset: i32)->Result<u32, ArmException>{
      let new_addr = if offset.is_negative(){
         pc - (offset.wrapping_abs() as u32)
      }else{
         pc + (offset as u32)
      };

      if !is_aligned(new_addr , 2){
         return Err(ArmException::HardFault(format!("invalid address ({})  writes to PC must be 2 byte aligned", new_addr)));
      }

      return Ok(new_addr);
   }

   fn lockup(&mut self){
      self.registers.pc = 0xFFFFFFFE;
      panic!("locked up at priority lvl:  {}",self.execution_priority(self.primask,&self.scs));
   }

   pub fn set_exc_pending(&mut self, exc: ArmException){
      self.active_exceptions[exc.number() as usize] = ExceptionStatus::Pending;
   }

   pub fn check_for_exceptions(&mut self){
      for i in 0 .. self.active_exceptions.len(){
         match &self.active_exceptions[i]{
            ExceptionStatus::Pending => {
               let maybe_exp: Option<ArmException> = ArmException::from_exception_number(i as u32);
               match maybe_exp{
                  Some(exc) => {
                     println!(
                        "execution == {} exception == {}",
                        self.execution_priority(self.primask,&self.scs),
                        exc.priority_group(&self.scs)
                     );
                     if exc.priority_group(&self.scs) < self.execution_priority(self.primask,&self.scs){
                        match self.init_exception(exc){
                           Ok(n) => {
                              if n.is_some(){
                                 return;
                              }
                           },
                           Err(exc) => {
                              panic!("error during exception entry {:?}",exc);
                              let current_priority = self.execution_priority(self.primask, &self.scs);
                              if current_priority == -1 || current_priority == -2{
                                 self.lockup();
                              }
                              return;
                           },
                        }
                     }
                  },
                  None => { println!("WARNING unrecognised pending exception {}",i); }
               }
            },
            _ => {}
         }
      }
   }

   fn init_exception(&mut self, exc_type: ArmException)->Result<Option<u32>,ArmException>{
      if self.execution_priority(self.primask,&self.scs) < exc_type.priority_group(&self.scs){
         println!("{:?} exception will remain pending",exc_type);
         self.active_exceptions[exc_type.number() as usize] = ExceptionStatus::Pending;
         self.scs.set_vec_pending(exc_type.number());
         return Ok(None);
      }else{
         println!("initialising {:? } exception",exc_type);
         self.save_context_frame(&exc_type)?;
         let offset = self.jump_to_exception(&exc_type)?;
         println!("exception offset: {:#x}",offset);
         self.offset_pc(offset)?;
         println!("exception branched pc -> {:#x}",offset);
         return Ok(Some(self.get_ipsr()));
      }
   }

   fn save_context_frame(&mut self,exc_type: &ArmException)->Result<(),ArmException>{
      let sp = self.get_sp();
      let mut xpsr = from_arm_bytes(self.xpsr);
      xpsr = if self.get_sp_frame_alignment(){
         xpsr | (1 << 9)
      }else{
         xpsr & (!(1 << 9))
      };

      let next_instr_address = exc_type.return_address(self.registers.pc as u32,true);

      let offset = std::mem::size_of::<ProcessStackFrame>();
      let frame_ptr = (sp - offset as u32) & !0x4;
      self.set_sp(frame_ptr)?;
      write_memory(self,frame_ptr, into_arm_bytes(self.registers.generic[0]))?;
      write_memory(self,frame_ptr + 4, into_arm_bytes(self.registers.generic[1]))?;
      write_memory(self,frame_ptr + 8, into_arm_bytes(self.registers.generic[2]))?;
      write_memory(self,frame_ptr + 12, into_arm_bytes(self.registers.generic[3]))?;
      write_memory(self,frame_ptr + 16, into_arm_bytes(self.registers.generic[12]))?;
      write_memory(self,frame_ptr + 20, into_arm_bytes(self.registers.lr))?;
      write_memory(self,frame_ptr + 24, into_arm_bytes(next_instr_address))?;
      write_memory(self,frame_ptr + 28, into_arm_bytes(xpsr))?;

      const main_stack_return_to_handler_mode: u32 = 0xFFFFFFF1;
      const main_stack_return_to_thread_mode: u32 = 0xFFFFFFF9;
      const process_stack_return_to_thread_mode: u32 = 0xFFFFFFFD;
      self.registers.lr = match self.mode{
         Mode::Handler => {0xFFFFFFF1},
         Mode::Thread =>{
            if self.sp_select_bit(){0xFFFFFFFD}else{0xFFFFFFF9}
         }
      };
      return Ok(());
   }

   fn reset_ipsr(&mut self){
      assert!(matches!(self.mode,Mode::Thread));
      let new_xpsr = from_arm_bytes(self.xpsr) & IPSR_MASK;
      self.xpsr = into_arm_bytes(new_xpsr);
   }

   fn set_ipsr(&mut self, exc_type: &ArmException){
      let mut new_xpsr = from_arm_bytes(self.xpsr);
      let mask: u32 = exc_type.number() & 0x3F;
      new_xpsr = (new_xpsr & !0x3F) | mask;
      println!("xpsr: {:#x} new ipsr: {:#x}",new_xpsr,mask);
      self.xpsr = into_arm_bytes(new_xpsr);
   }

   pub fn get_ipsr(&self)->u32{
      from_arm_bytes(self.xpsr) & 0x3F
   }


   fn jump_to_exception(&mut self, exc_type: &ArmException)->Result<i32, ArmException>{
      self.mode = Mode::Handler;
      self.set_ipsr(&exc_type);
      self.set_sp_select_bit(false);
      //panic!("SCS_UpdateStatusRegs() is not implemented");
      self.active_exceptions[exc_type.number() as usize] = ExceptionStatus::Active;
      self.scs.set_vec_active(exc_type.number());
      let vector_table = self.scs.vtor;
      let handler_addr = vector_table + (4 * exc_type.number());
      let handler_ptr = from_arm_bytes(load_memory::<4>(self, handler_addr)?);
      self.event_register = true;
      dbg_ln!("handler address: {:#x}, handler ptr: {:#x}",handler_addr,handler_ptr);
      self.bx_interworking_pc_offset(handler_ptr)
   }

   fn get_npriv(&self)->bool{
      (from_arm_bytes(self.control_register) & 1) > 0
   }

   fn load_context_frame(&mut self,exc_return_address: u32)->Result<(),ArmException>{
      let frame_ptr = self.get_sp();
      self.registers.generic[0] = from_arm_bytes(load_memory(self, frame_ptr)?);
      self.registers.generic[1] = from_arm_bytes(load_memory(self, frame_ptr + 4)?);
      self.registers.generic[2] = from_arm_bytes(load_memory(self, frame_ptr + 8)?);
      self.registers.generic[3] = from_arm_bytes(load_memory(self, frame_ptr + 12)?);
      self.registers.generic[12] = from_arm_bytes(load_memory(self, frame_ptr + 16)?);
      self.registers.lr = from_arm_bytes(load_memory(self, frame_ptr + 20)?);
      let new_pc = from_arm_bytes(load_memory(self, frame_ptr + 24)?);
      if !is_aligned(new_pc, 2){
         panic!("WTAF the pc value {:#x} onto the context frame is invalid",new_pc);
      }

      self.registers.pc = new_pc as usize;
      let frame_xpsr = from_arm_bytes(load_memory(self, frame_ptr + 28)?);
      let frame_alignment =  frame_xpsr & 0x200 > 0;
      let new_sp = (self.get_sp() + 0x20) | (frame_alignment as u32 >> 3);
      self.set_sp(new_sp)?;
      let new_xpsr = if matches!(self.mode,Mode::Thread) && self.get_npriv(){
         println!("forced into thread mode");
         0xF1000000 & frame_xpsr
      }else{
         0xF100003F & frame_xpsr
      };
      self.xpsr = into_arm_bytes(new_xpsr);
      return Ok(());
   }

   fn exception_return(&mut self,return_address: u32)->Result<u32,ArmException>{
      assert!(matches!(self.mode,Mode::Handler));
      let handled_exception = self.get_ipsr();
      assert_eq!(return_address & 0x0FFFFFF0,0x0FFFFFF0,"exception return address is invalid");
      
      let mut nested_exceptions = 0;
      for status in self.active_exceptions.iter(){
         match status{
            ExceptionStatus::Active => {
               nested_exceptions += 1;
            },
            _ => {},
         }
      }

      assert!(nested_exceptions > 0, "emulator err: return from an already inactive handler");

      let exc_ret_type = return_address & 0xF;
      match exc_ret_type{
         EXC_RETURN_TO_HANDLER => {
            assert!(nested_exceptions > 1,"exception return type is return to handler but only one exception is active");
            self.mode = Mode::Handler;
            self.set_sp_select_bit(false);
            assert_eq!(self.get_sp(),self.registers.sp_main);
         },
         EXC_RETURN_TO_THREAD_MSP =>{
            assert!(nested_exceptions == 1,"exception return type is return to thread, so there should only be 1 active");
            self.mode = Mode::Thread;
            self.set_sp_select_bit(false);
            assert_eq!(self.get_sp(),self.registers.sp_main);
         },
         EXC_RETURN_TO_THREAD_PSP =>{
            assert!(nested_exceptions == 1,"exception return type is return to thread, so there should only be 1 active");
            self.mode = Mode::Thread;
            self.set_sp_select_bit(true);
            assert_eq!(self.get_sp(),self.registers.sp_process);
         },
         _ => panic!("invalid exception return type {}",exc_ret_type)
      }

      self.active_exceptions[handled_exception as usize] = ExceptionStatus::Inactive;
      self.scs.clear_vec_active();

      self.load_context_frame(return_address)?;
      match self.mode{
         Mode::Thread => assert!(self.get_ipsr() == 0,"Thread mode must mean IPSR is 0"),
         Mode::Handler => assert!(self.get_ipsr() != 0, "Handler mode must mean IPSR > 0 "),
      }

      self.event_register = true;
      return Ok(handled_exception);
   }


   #[inline]
   pub fn on_breakpoint(&self)->bool{
      self.breakpoints.contains(&self.registers.pc)
   }

   #[inline]
   pub fn add_breakpoint(&mut self, addr: u32){
      if !self.breakpoints.contains(&(addr as usize)){
         self.breakpoints.push(addr as usize);
      }
   }

   #[inline]
   pub fn is_breakpoint(&self, addr: u32)->bool{
      self.breakpoints.contains(&(addr as usize))
   }

   #[inline]
   pub fn remove_breakpoint(&mut self,addr: u32){
      self.breakpoints.retain(|brkpt| *brkpt != (addr as usize));
   }

   pub fn step(&mut self)->Result<i32, ArmException>{
      let maybe_code: [u8;2] = load_thumb_instr(&self, self.registers.pc as u32)?;
      let instr_size = instruction_size(maybe_code);
      match instr_size{
         InstructionSize::B16 => {
            let code = Opcode::from(maybe_code);
            dbg_ln!(
               "@:{:#x} raw {:#x},{:#x} => {} :: {:?}",
               self.registers.pc as u32,
               maybe_code[0],
               maybe_code[1],
               code,
               get_operands(&code, maybe_code)
            );
            match code {
               Opcode::_16Bit(B16::ADD_Imm3)=>{
                  let (dest, src, imm3) = unpack_operands!(
                     get_operands(&code,maybe_code),
                     Operands::RegPairImm3,
                     a,b,c
                  );

                  //println!("executing {}, {}{}{}",code,dest,src,imm3);
                  let (sum,flags) = add_immediate(
                     self.registers.generic[src.0 as usize],
                     imm3.0
                  );
                  self.registers.generic[dest.0 as usize] = sum;
                  self.update_apsr(&flags);

                  return Ok(instr_size.in_bytes() as i32);
               }, 

               Opcode::_16Bit(B16::ADD_Imm8)=>{
                  let (dest, imm8) = unpack_operands!(
                     get_operands(&code,maybe_code),
                     Operands::DestImm8,
                     a,b
                  );

                  let (sum,flags) = add_immediate(
                     self.registers.generic[dest.0 as usize],
                     imm8.0
                  );

                  //println!("executing {}, {}{}",code,dest,imm8);
                  self.registers.generic[dest.0 as usize] = sum;
                  self.update_apsr(&flags);
                  return Ok(instr_size.in_bytes() as i32);
               },

               Opcode::_16Bit(B16::ADDS_REG) =>{
                  let (dest, src, arg) = unpack_operands!(
                     get_operands(&code,maybe_code),
                     Operands::RegisterTriplet,
                     a,b,c
                  );

                  //println!("executing {}, {}{}",code,src,arg);
                  let (sum,flags) = add_immediate(
                     self.registers.generic[src.0 as usize],
                     self.registers.generic[arg.0 as usize]
                  );

                  self.registers.generic[dest.0 as usize] = sum;
                  self.update_apsr(&flags);
                  return Ok(instr_size.in_bytes() as i32);
               },

               Opcode::_16Bit(B16::ADD_REG_SP_IMM8) =>{
                  let (dest,imm) = unpack_operands!(
                     get_operands(&code,maybe_code),
                     Operands::ADD_REG_SP_IMM8,
                     a,b
                  );

                  let (sum,_) = add_immediate(
                     self.get_sp(),
                     imm.0
                  );

                  self.registers.generic[dest.0 as usize] = sum;
                  return Ok(instr_size.in_bytes() as i32);
               },

               Opcode::_16Bit(B16::INCR_SP_BY_IMM7) =>{
                  let imm = unpack_operands!(
                     get_operands(&code,maybe_code),
                     Operands::INCR_SP_BY_IMM7,
                     a
                  );

                  let (sum,_) = add_immediate(
                     self.get_sp(),
                     imm.0
                  );

                  self.set_sp(sum)?;

                  return Ok(instr_size.in_bytes() as i32);
               },

               Opcode::_16Bit(B16::INCR_SP_BY_REG) => {
                  let src = unpack_operands!(
                     get_operands(&code,maybe_code),
                     Operands::INCR_SP_BY_REG,
                     r
                  );

                  let (sum,_) = add_immediate(
                     self.get_sp(),
                     self.read_any_register(src.0)
                  );

                  self.set_sp(sum)?;

                  return Ok(instr_size.in_bytes() as i32);
               },

               conditional_branches!() => {
                  let offset = unpack_operands!(
                     get_operands(&code,maybe_code),
                     Operands::COND_BRANCH,
                     i
                  );

                  if cond_passed(self.xpsr, &code){
                     println!("{} == true",&code);
                     return Ok(offset);
                  }else{
                     println!("{} == false",&code);
                     return Ok(instr_size.in_bytes() as i32);
                  }

               },

               Opcode::_16Bit(B16::B_ALWAYS) =>{
                  let offset = unpack_operands!(
                     get_operands(&code,maybe_code),
                     Operands::B_ALWAYS,
                     i
                  );
                  return Ok(offset);
               },

               Opcode::_16Bit(B16::BR_EXCHANGE) =>{
                  let register = unpack_operands!(
                     get_operands(&code, maybe_code),
                     Operands::BR_EXCHANGE,
                     r
                  );
                  let addr = self.read_any_register(register.0);
                  println!("will branch to {}",addr);
                  return self.bx_interworking_pc_offset(addr);
               },

               Opcode::_16Bit(B16::BR_LNK_EXCHANGE) =>{
                  let register = unpack_operands!(
                     get_operands(&code, maybe_code),
                     Operands::BR_LNK_EXCHANGE,
                     r
                  );
                  let interwork_addr = (self.registers.pc  as u32) + instr_size.in_bytes();
                  self.registers.lr = interwork_addr | 0x1;
                  let branch_target = self.read_any_register(register.0);
                  if register.0 == PROGRAM_COUNTER{
                     println!("WARN: reading from PC register for BLX is unpredictable undefined behaviour");
                  }
                  return self.bx_interworking_pc_offset(branch_target);
               },

               Opcode::_16Bit(B16::CMP_Imm8) => {
                  let (src, imm8) = unpack_operands!(
                     get_operands(&code,maybe_code),
                     Operands::CMP_Imm8,
                     a,b
                  );
                  println!("executing {} {},{}",code,src,imm8);
                  println!("comparing {} to {}", self.registers.generic[src.0 as usize], imm8.0);
                  let flags = compare(self.registers.generic[src.0 as usize], imm8.0);
                  println!("cmp result -> {:?}",flags);
                  self.update_apsr(&flags);
                  return Ok(instr_size.in_bytes() as i32);
               },

               Opcode::_16Bit(B16::CMP_REG_T1) => {
                  let (first, secnd) = unpack_operands!(
                     get_operands(&code,maybe_code),
                     Operands::PureRegisterPair,
                     a,b
                  );

                  let flags = compare(
                     self.registers.generic[first.0 as usize],
                     self.registers.generic[secnd.0 as usize]
                  );
                  self.update_apsr(&flags);
                  return Ok(instr_size.in_bytes() as i32);
               },

               Opcode::_16Bit(B16::CMP_REG_T2)=> {
                  let (first, secnd) = unpack_operands!(
                     get_operands(&code,maybe_code),
                     Operands::PureRegisterPair,
                     a,b
                  );

                  let flags = compare(
                     self.registers.generic[first.0 as usize],
                     self.registers.generic[secnd.0 as usize]
                  );
                  self.update_apsr(&flags);
                  return Ok(instr_size.in_bytes() as i32);
               },

               Opcode::_16Bit(B16::CPS) => {
                  let interrupt_flag = unpack_operands!(
                     get_operands(&code,maybe_code),
                     Operands::EnableInterupt,
                     a
                  );
                  if self.execution_priority(interrupt_flag, &self.scs) <= self.execution_priority(self.primask, &self.scs) {
                     self.primask = interrupt_flag;
                  }else{
                     self.pending_primask = Some(interrupt_flag);
                  }
                  return Ok(instr_size.in_bytes() as i32);
               },

               Opcode::_16Bit(B16::XOR_REG)=>{
                  let (dest,arg) = unpack_operands!(
                     get_operands(&code,maybe_code),
                     Operands::RegisterPair,
                     a,b
                  );
                  
                  let overflow = get_overflow_bit(self.xpsr);
                  let (res,flags) = xor(
                     self.registers.generic[dest.0 as usize],
                     self.registers.generic[arg.0 as usize], 
                     overflow
                  );

                  self.registers.generic[dest.0 as usize] = res;
                  self.update_apsr(&flags);

                  return Ok(instr_size.in_bytes() as i32);
               },
               
               Opcode::_16Bit(B16::SUB_Imm3) => {
                  let (dest,src,imm3) = unpack_operands!(
                     get_operands(&code,maybe_code),
                     Operands::RegPairImm3,
                     a,b,c
                  );

                  println!("executing {}, {}{}{}",code,dest,src,imm3);
                  let (sum,flags) = subtract(
                     self.registers.generic[src.0 as usize], 
                     imm3.0
                  );

                  self.registers.generic[dest.0 as usize] = sum;
                  self.update_apsr(&flags);
                  return Ok(instr_size.in_bytes() as i32);
               },

               Opcode::_16Bit(B16::SUB_Imm8) => {
                  let (dest,imm8) = unpack_operands!(
                     get_operands(&code,maybe_code),
                     Operands::DestImm8,
                     a,b
                  );

                  let (sum,flags) = subtract(
                     self.registers.generic[dest.0 as usize],
                     imm8.0
                  );

                  self.registers.generic[dest.0 as usize] = sum;
                  self.update_apsr(&flags);
                  return Ok(instr_size.in_bytes() as i32);
               },

               Opcode::_16Bit(B16::SUB_REG) => {
                  let (dest,src,arg) = unpack_operands!(
                     get_operands(&code, maybe_code),
                     Operands::RegisterTriplet,
                     a,b,c
                  );

                  let (sum,flags) = subtract(
                     self.registers.generic[src.0 as usize],
                     self.registers.generic[arg.0 as usize]
                  );

                  self.registers.generic[dest.0 as usize] = sum;
                  self.update_apsr(&flags);
                  return Ok(instr_size.in_bytes() as i32);
               },

               Opcode::_16Bit(B16::MUL) => {
                  let (dest,arg) = unpack_operands!(
                     get_operands(&code, maybe_code),
                     Operands::RegisterPair,
                     a,b
                  );

                  let (product,negative,zero) = multiply(
                     self.registers.generic[dest.0 as usize],
                     self.registers.generic[arg.0 as usize]
                  );

                  self.registers.generic[dest.0 as usize] = product;
                  let mut xpsr = from_arm_bytes(self.xpsr);
                  if negative{
                     xpsr = set_bit(31,xpsr);
                  }else{
                     xpsr = clear_bit(31,xpsr);
                  }

                  if zero{
                     xpsr = set_bit(30,xpsr);
                  }else{
                     xpsr = clear_bit(30,xpsr);
                  }

                  self.xpsr = into_arm_bytes(xpsr);
                  return Ok(instr_size.in_bytes() as i32);
               },

               Opcode::_16Bit(B16::MOV_Imm8)=> {
                  let (dest, imm8) = unpack_operands!(
                     get_operands(&code, maybe_code),
                     Operands::DestImm8,
                     a,i
                  );
                  println!("opr: {},{}",dest,imm8);
                  self.do_move(dest.0 as usize, imm8.0);
                  return Ok(instr_size.in_bytes() as i32);
               },

               Opcode::_16Bit(B16::MOV_REGS_T1)=> {
                  let (dest, src) = unpack_operands!(
                     get_operands(&code, maybe_code),
                     Operands::MOV_REG,
                     a,b
                  );
                  let v = self.registers.generic[src.0 as usize];
                  self.do_move(dest.0 as usize, v);
                  return Ok(instr_size.in_bytes() as i32);
               },

               Opcode::_16Bit(B16::MOV_REGS_T2)=>{
                  let (dest, src) = unpack_operands!(
                     get_operands(&code, maybe_code),
                     Operands::MOV_REG,
                     a,b
                  );

                  let v = self.registers.generic[src.0 as usize];
                  self.do_move(dest.0 as usize, v);
                  return Ok(instr_size.in_bytes() as i32);
               },

               Opcode::_16Bit(B16::MVN)=>{
                  let (dest,src) = unpack_operands!(
                     get_operands(&code, maybe_code),
                     Operands::MOV_REG,
                     a,b
                  );

                  let v = !self.registers.generic[src.0 as usize];
                  self.do_move(dest.0 as usize, v);
                  return Ok(instr_size.in_bytes()as i32);
               },

               Opcode::_16Bit(B16::ORR)=>{
                  let (dest,arg) = unpack_operands!(
                     get_operands(&code, maybe_code),
                     Operands::RegisterPair,
                     a,b
                  );

                  let v = self.registers.generic[dest.0 as usize] | self.registers.generic[arg.0 as usize];
                  self.registers.generic[dest.0 as usize] = v;
                  let overflow = get_overflow_bit(self.xpsr);
                  let flags = ConditionFlags{
                     negative: (0x80000000 & v) > 0,
                     carry: false,
                     zero: v == 0,
                     overflow
                  };

                  self.update_apsr(&flags); 

                  return Ok(instr_size.in_bytes() as i32);
               }

               Opcode::_16Bit(B16::LDR_REGS)=>{
                  let (dest,base,offset) = unpack_operands!(
                     get_operands(&code, maybe_code),
                     Operands::LDR_REG,
                     a,b,c
                  );

                  let addr = self.registers.generic[base.0 as usize] + self.registers.generic[offset.0 as usize];
                  let value: [u8;4] = load_memory::<4>(&self, addr)?;
                  self.registers.generic[dest.0 as usize] = from_arm_bytes(value);
                  return Ok(instr_size.in_bytes() as i32);
               },

               Opcode::_16Bit(B16::LDR_Imm5)=>{
                  let (dest,base,offset) = unpack_operands!(
                     get_operands(&code, maybe_code),
                     Operands::LDR_Imm5,
                     a,b,c
                  );

                  let addr = self.registers.generic[base.0 as usize] + offset.0;
                  let value: [u8;4] = load_memory(&self, addr)?;
                  self.registers.generic[dest.0 as usize] = from_arm_bytes(value);
                  return Ok(instr_size.in_bytes() as i32);
               }

               Opcode::_16Bit(B16::LDR_PC_Imm8) => {
                  let (dest,src,offset) = unpack_operands!(
                     get_operands(&code, maybe_code),
                     Operands::LDR_Imm8,
                     a,b,i
                  );

                  assert_eq!(src.0,15);
                  let addr = Self::offset_read_pc(self.read_pc_word_aligned(), offset.0 as i32)?;
                  let value = load_memory::<4>(&self, addr)?;
                  self.registers.generic[dest.0 as usize] = from_arm_bytes(value);
                  return Ok(instr_size.in_bytes() as i32);
               },

               Opcode::_16Bit(B16::LDM) =>{
                  let (base, list) = unpack_operands!(
                     get_operands(&code, maybe_code),
                     Operands::LoadableList,
                     b,l
                  );
                  let mut data_ptr = self.registers.generic[base.0 as usize];
                  let registers = get_set_bits(list);
                  for r in registers{
                     let v = from_arm_bytes(load_memory(self, data_ptr)?);
                     self.registers.generic[r as usize] = v;
                     data_ptr += 4;
                  }

                  let write_back = ((1 << base.0) & list) == 0;
                  if write_back{
                     self.registers.generic[base.0 as usize] = data_ptr;
                  }
                  return Ok(instr_size.in_bytes() as i32);
               },

               Opcode::_16Bit(B16::LSL_Imm5) =>{
                  let (dest,src,ammount) = unpack_operands!(
                     get_operands(&code,maybe_code),
                     Operands::LS_Imm5,
                     a,b,c
                  );

                  let overflow = get_overflow_bit(self.xpsr);
                  let arg = self.registers.generic[src.0 as usize];
                  let (res, flags) = shift_left(arg, ammount.0, overflow);
                  self.registers.generic[dest.0 as usize] = res;
                  self.update_apsr(&flags);

                  return Ok(instr_size.in_bytes() as i32);
               },

               Opcode::_16Bit(B16::LSL_REGS) =>{
                  let (dest,arg) = unpack_operands!(
                     get_operands(&code,maybe_code),
                     Operands::RegisterPair,
                     a,b
                  );

                  let overflow = get_overflow_bit(self.xpsr);
                  let val = self.registers.generic[dest.0 as usize];
                  let shift = self.registers.generic[arg.0 as usize] & 0xFF;

                  let (res, flags) = shift_left(val, shift, overflow);

                  self.registers.generic[dest.0 as usize] = res;
                  self.update_apsr(&flags);

                  return Ok(instr_size.in_bytes() as i32);
               },

               Opcode::_16Bit(B16::LSR_Imm5) =>{
                  let (dest,src,ammount) = unpack_operands!(
                     get_operands(&code,maybe_code),
                     Operands::LS_Imm5,
                     a,b,c
                  );

                  let overflow = get_overflow_bit(self.xpsr);
                  let arg = self.registers.generic[src.0 as usize];
                  let (res, flags) = shift_right(arg, ammount.0, overflow);
                  self.registers.generic[dest.0 as usize] = res;
                  self.update_apsr(&flags);

                  return Ok(instr_size.in_bytes() as i32);
               },

               Opcode::_16Bit(B16::LSR_REGS) =>{
                  let (dest,arg) = unpack_operands!(
                     get_operands(&code,maybe_code),
                     Operands::RegisterPair,
                     a,b
                  );

                  let overflow = get_overflow_bit(self.xpsr);
                  let val = self.registers.generic[dest.0 as usize];
                  let shift = self.registers.generic[arg.0 as usize] & 0xFF;

                  let (res, flags) = shift_right(val, shift, overflow);

                  self.registers.generic[dest.0 as usize] = res;
                  self.update_apsr(&flags);

                  return Ok(instr_size.in_bytes() as i32);
               },

               Opcode::_16Bit(B16::STR_Imm5) => {
                  let (v_reg,base,offset) = unpack_operands!(
                     get_operands(&code, maybe_code),
                     Operands::STR_Imm5,
                     a,b,i
                  );

                  let base_v = self.registers.generic[base.0 as usize];
                  let addr = base_v + offset.0;
                  let val = self.registers.generic[v_reg.0 as usize];

                  println!("*(&int({:#x})) := {}",addr,val);
                  write_memory::<4>(self, addr, into_arm_bytes(val))?;
                  return Ok(instr_size.in_bytes() as i32);
               },

               Opcode::_16Bit(B16::STM) =>{
                  let (base_register,list) = unpack_operands!(
                     get_operands(&code, maybe_code),
                     Operands::LoadableList,
                     r,l
                  );
                  let registers = get_set_bits(list);
                  if ((1 << base_register.0) & list > 0) && (registers[0] != base_register.0){
                     println!("WARN: r{} will be written back to, but it is not the first in the register list\n the result is UNKNOWN",base_register.0);
                  }
                  let mut array_ptr = self.registers.generic[base_register.0 as usize];
                  for r in registers{
                     write_memory(
                        self, 
                        array_ptr,
                        into_arm_bytes(self.registers.generic[r as usize]
                     ))?;
                     array_ptr += 4;
                  }
                  self.registers.generic[base_register.0 as usize] = array_ptr;
                  Ok(instr_size.in_bytes() as i32)
               },

               Opcode::_16Bit(B16::SVC)=> {
                  if self.execution_priority(self.primask,&self.scs) == -1 || self.execution_priority(self.primask,&self.scs) == -2{
                     self.lockup();
                     Ok(0)
                  }else{
                     Err(ArmException::Svc)
                  }
               },

               Opcode::_16Bit(B16::PUSH) => {
                  let list = unpack_operands!(
                     get_operands(&code, maybe_code),
                     Operands::RegisterList,
                     a
                  ); 
                  println!("executing {}, {}",code,list);
                  let set_bits = get_set_bits(list);
                  let offset = (4 * set_bits.len()) as u32;
                  let new_sp = self.get_sp() - offset;
                  let mut addr = new_sp;
                  for reg_bit in set_bits{
                     let v = if reg_bit == asm::LINK_REGISTER{
                        self.registers.lr
                     }else {
                        self.registers.generic[reg_bit as usize]
                     };
                     println!("PUSH wrote {:?} to {}",into_arm_bytes(v), addr);
                     write_memory(self, addr, into_arm_bytes(v))?;
                     addr = addr + 4;
                  }

                  self.set_sp(new_sp)?;
                  return Ok(instr_size.in_bytes() as i32);
               },

               Opcode::_16Bit(B16::POP) =>{
                  let mut offset = instr_size.in_bytes() as i32;
                  let list = unpack_operands!(
                     get_operands(&code, maybe_code),
                     Operands::RegisterList,
                     a
                  ); 
                  println!("SP before POP {}",self.get_sp());
                  let set_bits = get_set_bits(list);
                  let old_sp = self.get_sp() + (4 * set_bits.len() as u32);
                  let mut addr = self.get_sp();
                  for reg_bit in set_bits{
                     let v = load_memory::<4>(self,addr)?;
                     println!("POP loaded {:?} to r{} from addr {}", v, reg_bit, addr);
                     if reg_bit == asm::PROGRAM_COUNTER{
                        offset = self.bx_interworking_pc_offset(from_arm_bytes(v))?;
                     }else{
                        self.registers.generic[reg_bit as usize] = from_arm_bytes(v);
                     }
                     addr += 4;
                  }

                  self.set_sp(old_sp)?;
                  return Ok(offset);
               },

               Opcode::_16Bit(B16::NOP) => {
                  return Ok(instr_size.in_bytes() as i32);
               },

               Opcode::_16Bit(B16::WFE) => {
                  if self.event_register{
                     self.event_register = false;
                     return Ok(instr_size.in_bytes() as i32);
                  }else{
                     return Ok(0_i32);
                  }
               }
               _ => todo!()
            } 
         },
         InstructionSize::B32 => {
            let word: [u8;4] = load_instr_32b(&self, self.registers.pc as u32)?;
            let instr_32b = Opcode::from(word);
            match instr_32b{
               Opcode::_32Bit(B32::MSR) => {
                  let (special, src) = unpack_operands!(
                     get_operands_32b(&instr_32b, word),
                     Operands::MSR,
                     s,y
                  );

                  if special.needs_privileged_access() && !self.in_privileged_mode(){
                     println!("Do not have access rights to {:?} in {:?} mode",special,self.mode);
                     return Ok(instr_size.in_bytes() as i32);
                  }

                  assert!(src.0 != 13 && src.0 != 15,"MSR for R13 and R15 is UNDEFINED");
                  let src_value = self.read_any_register(src.0);
                  match special{
                     SpecialRegister::MSP => {
                        self.registers.sp_main = src_value & 0xFFFFFFFC_u32;
                     },
                     SpecialRegister::PSP => {
                        self.registers.sp_process = src_value & 0xFFFFFFFC_u32;
                     },
                     SpecialRegister::CONTROL => {
                        let mut cr = from_arm_bytes(self.control_register); 
                        if matches!(self.mode,Mode::Thread){
                           //can only only change the sp_sel bit in thread mode
                           cr = cr | (src_value & 3);
                        }else{
                           cr = cr | (src_value & 1);
                        }
                        self.control_register = into_arm_bytes(cr);
                     },
                     _ => todo!()
                  }

                  return Ok(instr_size.in_bytes() as i32);
               },

               Opcode::_32Bit(B32::BR_AND_LNK) => {
                  let next_instr_addr = self.registers.pc as u32  + instr_size.in_bytes();
                  let interworking_addr = next_instr_addr | 0x1;
                  println!("BL set LR to {0}({0:x})",interworking_addr);
                  self.registers.lr = interworking_addr;

                  let offset = unpack_operands!(
                     get_operands_32b(&instr_32b, word),
                     Operands::BR_LNK,
                     i
                  );

                  println!("executing {}, {}",instr_32b,offset);
                  return Ok(offset);
               },
               _ => unreachable!()
            }
         }
      }
   }

   pub fn reset(&mut self){
      self.mode = Mode::Thread;
      self.reset_ipsr();
      self.primask = false;
      self.control_register = [0;4];
      self.scs = SystemControlSpace::reset();
      self.active_exceptions = [ExceptionStatus::Inactive;48];
      self.event_register = false;
      let main_sp_reset_val: u32 = from_arm_bytes(load_memory(&self, self.scs.vtor).unwrap());
      let reset_handler_ptr: u32 = from_arm_bytes(load_memory(&self,self.scs.vtor + 4).unwrap()) & (!1);
      self.registers.sp_main = main_sp_reset_val;
      self.registers.pc = reset_handler_ptr as usize;
   }

   fn do_move(&mut self, dest: usize, value: u32){
      self.registers.generic[dest] = value;
      let mut new_xpsr =  from_arm_bytes(self.xpsr);
      new_xpsr = if self.registers.generic[dest] & 0x80000000 > 0{
         set_bit(31,new_xpsr)
      }else{
         clear_bit(31,new_xpsr)
      };

      new_xpsr = if self.registers.generic[dest] == 0{
         set_bit(30,new_xpsr)
      }else{
         clear_bit(30,new_xpsr)
      };

      self.xpsr = into_arm_bytes(new_xpsr);
   }

   pub fn check_permission(&self, addr: u32, acc: Access)->Result<(),ArmException>{
      let (attributes,exec_never) = self.get_permissions(addr);
      let err_msg = Err(ArmException::HardFault(
            format!("Invalid attempt to {:?} at address {:#x}, :{:#x} is {}",
            acc,
            addr,
            addr,
            attributes)
         )
      );
      let level = if self.in_privileged_mode(){
         attributes.privileged
      }else{
         attributes.unprivileged
      };
      match acc{
        Access::READ => match level{
           AccessPermission::NoAccess=> err_msg,
           _ => Ok(())
        },
        Access::WRITE => match level{
           AccessPermission::NoAccess | AccessPermission::ReadOnly => err_msg,
           _ => Ok(())
        },
        Access::Execute => if exec_never {
           Err(ArmException::HardFault(format!("cannot execute instruction at {:#x} it is marked as XN",addr)))
        }else{
           match level {
              AccessPermission::NoAccess => err_msg,
              _ => Ok(())
           }
        },
      }
   }

   pub fn get_permissions(&self, addr: u32)->(MemPermission,bool){
      //TODO allow configurable address attributes map with MPU
      self.default_permissions(addr)
   }

   ///bool is the execute never flag
   #[inline]
   pub fn default_permissions(&self, addr: u32)->(MemPermission,bool){
      let sig = (addr & 0xE0000000) >> 29;
      match sig{
         0b000 => (MemPermission::full_access(),false),
         0b001 => (MemPermission::full_access(),false),
         0b010 => (MemPermission::full_access(),true),
         0b011 => (MemPermission::full_access(),false),
         0b100 => (MemPermission::full_access(),false),
         0b101 => (MemPermission::full_access(),true),
         0b110 => (MemPermission::full_access(),true), 
         0b111 => (MemPermission::full_access(),true), 
         _ => unreachable!()
      }
   }

   fn execution_priority(&self,primask: bool, scs: &SystemControlSpace)->i32{
      let mut cur_priority: i32 = 4;
      let boosted_priority = if primask {0}else{4};

      for (i,status) in self.active_exceptions.iter().enumerate(){
         match status{
            ExceptionStatus::Active => {
               let exception = ArmException::from_exception_number(i as u32).unwrap();
               let exc_priority = exception.priority_group(scs);
               cur_priority = std::cmp::min(cur_priority,exc_priority);
            },
            _ => {}
         }
      }

      std::cmp::min(cur_priority,boosted_priority)
   }
}

pub fn is_thread_privileged(sys: &System)->bool{
   let v = from_arm_bytes(sys.control_register);
   return (v & 1) == 0;
}

pub fn get_exception_number(sys: &System)-> u32{
   return from_arm_bytes(sys.xpsr) & 0x3F;
}

fn is_system_in_le_mode(_sys: &System)-> bool{
   //TODO simulate this properly
   //NOTE access to PPB (0xE0000000 -> xE0100000 should always be little endian
   return true;
}

fn fault_if_not_aligned(addr: u32, alignment_in_bytes: usize)->Result<(), ArmException>{
   if !is_aligned(addr, alignment_in_bytes as u32){
      return Err( ArmException::HardFault(format!("address({}) is not correctly aligned for {} byte access",addr,alignment_in_bytes)));
   }else{
      return Ok(());
   }
}

pub fn load_memory<const T: usize>(sys: &System, v_addr: u32)->Result<[u8;T],ArmException>{
   fault_if_not_aligned(v_addr, T)?;
   sys.check_permission(v_addr, Access::READ)?;
   let mem: [u8;T] = sys.memory[v_addr as usize .. (v_addr as usize + T)]
      .try_into()
      .expect("should not access out of bounds memory");
   return Ok(mem);
}

pub fn load_thumb_instr(sys: &System, v_addr: u32)->Result<[u8;2],ArmException>{
   fault_if_not_aligned(v_addr, 2)?;
   sys.check_permission(v_addr, Access::Execute)?;
   let mem: [u8;2] = sys.memory[v_addr as usize .. (v_addr as usize + 2)]
      .try_into()
      .expect("should not access out of bounds memory");
   return Ok(mem);
}

fn load_instr_32b(sys: &System, addr: u32)->Result<[u8;4],ArmException>{
   //for armv6-m instruction fetches are always 16bit aligned 
   fault_if_not_aligned(addr, 2)?;
   sys.check_permission(addr, Access::Execute)?;
   let mem: [u8;4] = sys.memory[addr as usize .. (addr as usize + 4)]
      .try_into()
      .expect("should not access out of bounds memory");
   return Ok(mem);
}

/*
pub fn load_memory<'a, const T: usize>(sys: &'a System, v_addr: u32)->Result<&'a [u8;T],SysErr>{
   if !is_aligned(v_addr, T as u32){
      return Err(SysErr::HardFault);
   }

   let mem: &'a[u8;T] = sys.memory[v_addr as usize .. (v_addr as usize + T)]
      .try_into()
      .expect("should not access out of bounds memory");
   return Ok(mem)
}
*/

pub fn write_memory<const T: usize>(sys: &mut System, v_addr: u32, value: [u8;T])->Result<(), ArmException>{
   fault_if_not_aligned(v_addr, T)?;
   sys.check_permission(v_addr, Access::WRITE)?;
   sys.memory[v_addr as usize ..(v_addr as usize + T )].copy_from_slice(&value);
   return Ok(());
}

fn is_aligned(v_addr: u32, size: u32)->bool{
   let mask: u32 = size - 1;
   return v_addr & mask == 0;
}

#[derive(Clone,Copy,Debug)]
pub enum ExceptionStatus{
   Active,
   Inactive,
   Pending,
   ActiveAndPending
}

#[derive(Clone,Debug)]
pub enum ArmException{
   Reset,
   Nmi,
   HardFault(String),
   Svc,
   PendSV,
   SysTick,
   ExternInterrupt(u32),
}

impl ArmException{
   pub fn from_xpsr(sys: &System)->Option<Self>{
      let ipsr = from_arm_bytes(sys.xpsr) & 0x2F;
      return Self::from_exception_number(ipsr);
   }

   pub fn from_exception_number(n: u32)->Option<Self>{
      match n{
         0 => None,
         1 => Some(Self::Reset),
         2 => Some(Self::Nmi),
         3 => Some(Self::HardFault("".into())),
         4 ..=10 => None, //is describe as RESERVED in the ISA
         11 => Some(Self::Svc),
         12 ..=13 => None, //RESERVED
         14 => Some(Self::PendSV),
         15 => Some(Self::SysTick),
         n  => Some(Self::ExternInterrupt(n))
      }
   }

   pub fn return_address(&self,current_address: u32, sync: bool)-> u32{
      let next = (current_address + 2) & 0xFFFFFFFE;
      println!("{:?}@{:#x} will return to {:#x}",&self,current_address,next);
      match self{
         ArmException::Reset => panic!("cannot return from reset exception"),
         ArmException::Nmi => next,
         ArmException::HardFault(_) => if sync{ current_address }else{ next },
         ArmException::Svc => next,
         ArmException::PendSV => next,
         ArmException::SysTick => next,
         ArmException::ExternInterrupt(_) => next,
      }
   }

   pub fn number(&self)->u32{
      match self{
         Self::Reset => 1,
         Self::Nmi => 2,
         Self::HardFault(_) => 3,
         Self::Svc => 11,
         Self::PendSV => 14,
         Self::SysTick => 15,
         Self::ExternInterrupt(n) => *n,
      }
   }

   pub fn is_fault(&self)->bool{
      match self{
         Self::Reset => false,
         Self::Nmi => false,
         Self::HardFault(_) => true,
         Self::Svc => false,
         Self::PendSV => false,
         Self::SysTick => false,
         Self::ExternInterrupt(_) => false,
      }
   }

   pub fn priority_group(&self, _scs: &SystemControlSpace)->i32{
      match self{
         Self::Reset => -3,
         Self::Nmi => -2,
         Self::HardFault(_) => -1,
         Self::Svc =>{ ((_scs.shpr2 & 0x80000000) >> 31) as i32 },
         Self::PendSV =>{ ((_scs.shpr3 & 0x00800000) >> 23) as i32 },
         Self::SysTick =>{ ((_scs.shpr3 & 0x80000000) >> 31) as i32 },
         Self::ExternInterrupt(n) => _scs.nvic_priority_of(*n),
      }
   }
}

pub struct SystemControlSpace{
   pub icsr: u32,
   pub vtor: u32,
   shpr2: u32,
   shpr3: u32,
   ipr: [u32;8]
}

impl SystemControlSpace{
   pub fn reset()->Self{
      Self { icsr: 0, vtor: 0, shpr2: 0, shpr3: 0, ipr: [0; 8] }
   }

   pub fn set_vec_active(&mut self, exc_n: u32){
      self.icsr |= exc_n;
   }

   pub fn clear_vec_active(&mut self){
      self.icsr &= IPSR_MASK;
   }

   pub fn set_vec_pending(&mut self, exc_n: u32){
      self.icsr |= exc_n << 12;
   }

   pub fn clear_vec_pending(&mut self){
      self.icsr &= !(IPSR_MASK << 12);
   }

   pub fn nvic_priority_of(&self, exec: u32)->i32{
      let word_offset = (exec - 16) & 0xFFFFFFFC;
      let intra_word_offset = (exec - 16) - word_offset;
      let mut shift = 7; 
      shift += 8 * intra_word_offset;
      return ((self.ipr[word_offset as usize] & (1 << shift)) >> shift) as i32;
   }
}

fn assert_executable(addr: u32)->Result<(),ArmException>{
   if default_address_map(addr) & (AddressAttributes::ExecuteNever as u8) > 0 {
      return Err(
         ArmException::HardFault(format!("Cannot execute address code on {:#010x} it is XN",addr))
      );
   }else{
      return Ok(());
   }
}

#[repr(u8)]
enum AddressAttributes{
   Normal = 0x1,
   Device = 0x2,
   DevSharable = 0x4,
   DevNonShare = 0x8,
   ExecuteNever = 0x10,
   StronglyOrdered = 0x20
}

pub struct MemPermission{
   pub privileged: AccessPermission,
   pub unprivileged: AccessPermission
}

impl Display for MemPermission{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
       write!(f,"privileged({:?}) :: unprivileged({:?})",&self.privileged,&self.unprivileged)
    }
}

#[derive(Debug)]
pub enum AccessPermission{
   NoAccess,
   ReadOnly,
   ReadAndWrite
}

impl MemPermission{

   #[inline]
   pub fn full_access() ->Self{
      Self{
         privileged: AccessPermission::ReadAndWrite,
         unprivileged: AccessPermission::ReadAndWrite
      }
   }

   pub fn from_mpu_rasr(raw: u32)->Result<Self, ArmException>{
      let perms = (raw & 0x03000000) >> 24;
      match perms {
         0b000 => Ok(Self { 
            privileged: AccessPermission::NoAccess,
            unprivileged: AccessPermission::NoAccess 
         }),
         0b001 => Ok(Self{
            privileged: AccessPermission::ReadAndWrite,
            unprivileged: AccessPermission::NoAccess,
         }),
         0b010 => Ok(Self{
            privileged: AccessPermission::ReadAndWrite,
            unprivileged: AccessPermission::ReadOnly
         }),
         0b011 => Ok(Self::full_access()),
         0b100 => Err(ArmException::HardFault("AP value of 0x4 is undefined".into())),
         0b101 => Ok(Self{
            privileged: AccessPermission::ReadAndWrite,
            unprivileged: AccessPermission::NoAccess
         }),
         0b110 | 0b111 => Ok(Self{
            privileged: AccessPermission::ReadOnly,
            unprivileged: AccessPermission::ReadOnly
         }),
         _ => unreachable!(),
      }
   }
}


fn is_region_ppb(v_addr: u32)->bool{
   match v_addr{
      0xE0000000 ..= 0xE00FFFFF => true,
      _ => false
   }
}

#[derive(Debug)]
pub enum Access{
   READ,
   WRITE,
   Execute
}

fn default_address_map(v_addr: u32)-> u8{
   match v_addr{
      0x0 ..= 0x1FFFFFFF => AddressAttributes::Normal as u8,
      0x20000000 ..= 0x3FFFFFFF => AddressAttributes::Normal as u8,
      0x40000000 ..= 0x5FFFFFFF => AddressAttributes::Device as u8,
      0x60000000 ..= 0x7FFFFFFF => AddressAttributes::Normal as u8,
      0x80000000 ..= 0x9FFFFFFF => AddressAttributes::Normal as u8,
      0xA0000000 ..= 0xBFFFFFFF => AddressAttributes::DevSharable as u8 | AddressAttributes::ExecuteNever as u8,
      0xC0000000 ..= 0xDFFFFFFF => AddressAttributes::DevNonShare as u8 | AddressAttributes::ExecuteNever as u8,
      0xE0000000 ..= 0xE00FFFFF => AddressAttributes::StronglyOrdered as u8 | AddressAttributes::ExecuteNever as u8,
      0xE0100000 ..= 0xFFFFFFFF => AddressAttributes::ExecuteNever as u8
   }
}

