use crate::binutils::{from_arm_bytes, clear_bit, set_bit, into_arm_bytes};
use crate::asm::decode::{Opcode, instruction_size, InstructionSize, B16};
use crate::asm::decode_operands::{Operands,get_operands};
use crate::system::instructions::{add_immediate,ConditionFlags,compare,subtract,multiply} ;

use self::instructions::cond_passed;
use self::registers::{Registers, Apsr};
pub mod registers;
pub mod instructions;

pub const TRACED_VARIABLES: usize = 8;

pub struct System{
   pub registers: Registers,
   pub xpsr: Apsr,
   pub control_register: [u8;4],
   pub memory: Vec<u8>
}

pub enum SpType{
   MAIN,
   PROCESS
}

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

impl System{
   pub fn create(capacity: usize)->Self{
      let registers = Registers::create();
      return System{
         registers,
         xpsr: [0;4],
         control_register: [0;4],
         memory: vec![0;capacity]
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

   pub fn set_pc(&mut self, addr: usize)->Result<(),SysErr>{
      if addr > u32::MAX as usize{
         println!("tried to set PC to ({}), value is unrepresentable by 32bits",addr);
         return Err(SysErr::HardFault);
      }
      if !is_aligned(addr as u32, 2){
         return Err(SysErr::HardFault);
      }
      self.registers.pc = addr as usize;
      return Ok(());
   }

   pub fn get_pc(&self)->u32{
      return (self.registers.pc + 4) as u32;
   }

   pub fn read_pc_word_aligned(&self)->u32{
      return ((self.registers.pc + 4) as u32 ) & 0xFFFFFF00;
   }

   pub fn offset_pc(&mut self, offset: i32 )->Result<(),SysErr>{
      let new_addr = Self::offset_read_pc(self.registers.pc as u32,offset)?;
      println!("pc {} -> {}",self.registers.pc,new_addr);
      self.registers.pc = new_addr as usize;
      return Ok(());
   }

   fn offset_read_pc(pc: u32, offset: i32)->Result<u32, SysErr>{
      let new_addr = if offset.is_negative(){
         pc - (offset.wrapping_abs() as u32)
      }else{
         pc + (offset as u32)
      };

      if !is_aligned(new_addr , 2){
         return Err(SysErr::HardFault);
      }

      return Ok(new_addr);

   }

   pub fn step(&mut self)->Result<i32, SysErr>{
      let maybe_code: &[u8;2] = load_memory::<2>(&self, self.registers.pc as u32)?;
      let instr_size = instruction_size(maybe_code);
      match instr_size{
         InstructionSize::B16 => {
            let code = Opcode::from(maybe_code);
            println!("executing {}",code);
            match code {
               Opcode::_16Bit(B16::ADD_Imm3)=>{
                  let (dest, src, imm3) = unpack_operands!(
                     get_operands(&code,maybe_code),
                     Operands::RegPairImm3,
                     a,
                     b,
                     c
                  );

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
                     a,
                     b
                  );

                  let (sum,flags) = add_immediate(
                     self.registers.generic[dest.0 as usize],
                     imm8.0
                  );

                  self.registers.generic[dest.0 as usize] = sum;
                  self.update_apsr(&flags);
                  return Ok(instr_size.in_bytes() as i32);
               },

               Opcode::_16Bit(B16::ADDS_REG) =>{
                  let (dest, src, arg) = unpack_operands!(
                     get_operands(&code,maybe_code),
                     Operands::RegisterTriplet,
                     a,
                     b,
                     c
                  );

                  let (sum,flags) = add_immediate(
                     self.registers.generic[src.0 as usize],
                     self.registers.generic[arg.0 as usize]
                  );

                  self.registers.generic[dest.0 as usize] = sum;
                  self.update_apsr(&flags);
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
               }

               Opcode::_16Bit(B16::CMP_Imm8) => {
                  let (src, imm8) = unpack_operands!(
                     get_operands(&code,maybe_code),
                     Operands::CMP_Imm8,
                     a,b
                  );
                  
                  let flags = compare(self.registers.generic[src.0 as usize], imm8.0);
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

               Opcode::_16Bit(B16::SUB_Imm3) => {
                  let (dest,src,imm3) = unpack_operands!(
                     get_operands(&code,maybe_code),
                     Operands::RegPairImm3,
                     a,b,c
                  );

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

               Opcode::_16Bit(B16::LDR_REGS)=>{
                  let (dest,base,offset) = unpack_operands!(
                     get_operands(&code, maybe_code),
                     Operands::LDR_REG,
                     a,b,c
                  );

                  let addr = self.registers.generic[base.0 as usize] + self.registers.generic[offset.0 as usize];
                  let value: &[u8;4] = load_memory::<4>(&self, addr)?;
                  self.registers.generic[dest.0 as usize] = from_arm_bytes(*value);
                  return Ok(instr_size.in_bytes() as i32);
               },

               Opcode::_16Bit(B16::LDR_PC_Imm8) => {
                  let (dest,src,offset) = unpack_operands!(
                     get_operands(&code, maybe_code),
                     Operands::LDR_Imm8,
                     a,b,i
                  );

                  assert_eq!(src.0,15);
                  let addr = Self::offset_read_pc(self.read_pc_word_aligned(), offset.0 as i32)?;
                  let value = load_memory(&self, addr)?;
                  self.registers.generic[dest.0 as usize] = from_arm_bytes(*value);
                  return Ok(instr_size.in_bytes() as i32);
               }
               _ => unreachable!()
            } 
         },
         InstructionSize::B32 => {
            let word: &[u8;4] = load_memory::<4>(&self, self.registers.pc as u32)?;
            let _instr_32b = Opcode::from(word);
            todo!();
         }
      }
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
}

pub fn get_stack_pointer_type(sys: &System)->SpType{
   let v = from_arm_bytes(sys.control_register);
   if (v & 0x2) > 0 {
      return SpType::PROCESS;
   }else{
      return SpType::MAIN;
   }
}

pub fn is_thread_privelaged(sys: &System)->bool{
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

pub fn load_memory<'a, const T: usize>(sys: &'a System, v_addr: u32)->Result<&'a [u8;T],SysErr>{
   if !is_aligned(v_addr, T as u32){
      return Err(SysErr::HardFault);
   }

   let mem: &'a[u8;T] = sys.memory[v_addr as usize .. (v_addr as usize + T)]
      .try_into()
      .expect("should not access out of bounds memory");
   return Ok(mem)
}

pub fn write_memory<const T: usize>(sys: &mut System, v_addr: u32, value: [u8;T])->Result<(), SysErr>{
   if !is_aligned(v_addr, T as u32){
      return Err(SysErr::HardFault);
   }

   sys.memory[v_addr as usize ..(v_addr as usize + T )].copy_from_slice(&value);
   return Ok(());
}

fn is_aligned(v_addr: u32, size: u32)->bool{
   let mask: u32 = size - 1;
   return v_addr & mask == 0;
}

#[derive(Clone,Debug)]
pub enum SysErr{
   HardFault,
}

//
//#[repr(u8)]
//enum AddressAttributes{
//   Normal = 0x1,
//   Device = 0x2,
//   DevSharable = 0x4,
//   DevNonShare = 0x8,
//   ExecuteNever = 0x16,
//   StronglyOrdered = 0x32
//}
//
//fn default_address_map(v_addr: u32)-> u8{
//   match v_addr{
//      0x0 ..= 0x1FFFFFFF => AddressAttributes::Normal as u8,
//      0x20000000 ..= 0x3FFFFFFF => AddressAttributes::Normal as u8,
//      0x40000000 ..= 0x5FFFFFFF => AddressAttributes::Device as u8,
//      0x60000000 ..= 0x7FFFFFFF => AddressAttributes::Normal as u8,
//      0x80000000 ..= 0x9FFFFFFF => AddressAttributes::Normal as u8,
//      0xA0000000 ..= 0xBFFFFFFF => AddressAttributes::DevSharable as u8 | AddressAttributes::ExecuteNever as u8,
//      0xC0000000 ..= 0xDFFFFFFF => AddressAttributes::DevNonShare as u8 | AddressAttributes::ExecuteNever as u8,
//      0xE0000000 ..= 0xE00FFFFF => AddressAttributes::StronglyOrdered as u8,
//      0xE0100000 ..= 0xFFFFFFFF => AddressAttributes::ExecuteNever as u8
//   }
//}
//
