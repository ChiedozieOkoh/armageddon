use super::{HalfWord,Word};

#[allow(non_camel_case_types)]
#[derive(Debug,PartialEq)]
pub enum Opcode{
   UNDEFINED,
   ADCS,
   ADDI,
   ADDI8,
   ADDS_REG,
   ADDS_REG_T2,
   ADD_SPI,
   ADD_REG_SP_IMM8,
   INCR_SP_BY_IMM7,
   INCR_SP_BY_REG,
   INCR_REG_BY_SP,
   ADR,
   ANDS,
   ASRS_I,
   ASRS_REG,
   SVC,
   BEQ,
   BNEQ,
   B_CARRY_IS_SET,
   B_CARRY_IS_CLEAR,
   B_IF_NEGATIVE,
   B_IF_POSITIVE,
   B_IF_OVERFLOW,
   B_IF_NO_OVERFLOW,
   B_UNSIGNED_HI,
   B_UNSIGNED_LOW,
   B_GTE,
   B_LTE,
   B_GT,
   B_LT,
   B_ALWAYS,
   BIT_CLEAR_REGISTER,
   BREAKPOINT,
   BR_AND_LNK,
   BR_LNK_EXCHANGE,
   BR_EXCHANGE
} 

impl From<&HalfWord> for Opcode{
   fn from(a: &HalfWord)->Self{
      if adcs_mask(&a){
         return Opcode::ADCS;
      }

      if addi_mask(&a){
         return Opcode::ADDI;
      }

      if addi8_mask(&a){
         return Opcode::ADDI8;
      }

      if adds_3reg_mask(&a){
         return Opcode::ADDS_REG;
      }

      if incr_reg_by_sp(&a){
         return Opcode::INCR_REG_BY_SP;
      }

      if incr_sp_by_reg_mask(&a){
         return Opcode::INCR_SP_BY_REG;
      }

      if adds_2reg_mask(&a){
         return Opcode::ADDS_REG_T2;
      }

      if add_sp_with_immediate_and_store_in_reg_mask(&a){
         return Opcode::ADD_REG_SP_IMM8;
      }

      if incr_sp_by_imm7_mask(&a){
         return Opcode::INCR_SP_BY_IMM7;
      }

      if adr_mask(&a){
         return Opcode::ADR;
      }

      if ands_mask(&a){
         return Opcode::ANDS;
      }

      if asrsi_mask(&a){
         return Opcode::ASRS_I;
      }

      if asrs_reg_mask(&a){
         return Opcode::ASRS_REG;
      }

      if ukwn_branch_mask(&a){
         return Opcode::UNDEFINED;
      }

      if svc_mask(&a){
         return Opcode::SVC;
      }

      let maybe_branch = try_get_branch(&a);
      if maybe_branch.is_some(){
         return maybe_branch.unwrap();
      }

      if bics_mask(&a){
         return Opcode::BIT_CLEAR_REGISTER
      }

      if bkpt_mask(&a){
         return Opcode::BREAKPOINT;
      }

      if blx_mask(&a){
         return Opcode::BR_LNK_EXCHANGE;
      }

      if bx_mask(&a){
         return Opcode::BR_EXCHANGE;
      }
      Opcode::UNDEFINED
   }
}

#[inline]
const fn adcs_mask(hw: &HalfWord)->bool{
   //(hw[0] == 0x41) && (hw[1] == 0x40)
   (hw[0] & 0x40 > 0 ) && hw[1] == 0x41
}

#[inline]
const fn addi_mask(hw: &HalfWord)->bool{
   hw[1] == 0x1C || hw[1] == 0x1D
}

#[inline]
const fn addi8_mask(hw: &HalfWord)->bool{
   //(hw[1] >> 3 ) == 0x03
   hw[1] & 0xF8 == 0x30
}

#[inline]
const fn adds_3reg_mask(hw: &HalfWord)->bool{
   //hw[0] == 0x18 || hw[0] == 0x19
   hw[1] & 0xFE == 0x18
}

#[inline]
const fn adds_2reg_mask(hw: &HalfWord)->bool{
   hw[1] == 0x44
}

#[inline]
const fn add_sp_with_immediate_and_store_in_reg_mask(hw: &HalfWord)->bool{
   hw[1] & 0xF8 == 0xA8
}

#[inline]
const fn incr_sp_by_imm7_mask(hw: &HalfWord)->bool{
   hw[1] == 0xB0 && (hw[0] & 0x80 == 0)
}

#[inline]
const fn incr_reg_by_sp(hw: &HalfWord)->bool{
   hw[1] == 0x44 && (hw[0] & 0x78 == 0x68)
}

#[inline]
const fn incr_sp_by_reg_mask(hw: &HalfWord)->bool{
   hw[1] == 0x44 && (hw[0] & 0x87 == 0x85)
}

#[inline]
const fn adr_mask(hw: &HalfWord)->bool{
   hw[1] & 0xF8 == 0xA0 
}

#[inline]
const fn ands_mask(hw: &HalfWord)->bool{
   hw[0] == 0x40 && (hw[1] >> 6 == 0)
}

#[inline]
const fn asrsi_mask(hw: &HalfWord)->bool{
   hw[0] >> 4 == 0x01 && (hw[0] | (1 << 3) != hw[0])
}

#[inline]
const fn asrs_reg_mask(hw: &HalfWord)->bool{
   hw[0] == 0x41 && (hw[1] >> 6 == 0)
}

#[inline]
const fn ukwn_branch_mask(hw: &HalfWord)->bool{
   hw[0] == 0xDE
}

#[inline]
const fn svc_mask(hw: &HalfWord)->bool{
   hw[0] == 0xDF
}

#[inline]
const fn try_get_branch(hw: &HalfWord)->Option<Opcode>{
   if hw[0] & 0xF0 == 0xE0 && hw[0] | 0x08 != hw[0]{
      return Some(Opcode::B_ALWAYS);
   }
   if hw[0] & 0xF0 != 0xC0{
      return None;
   }
   let cond = hw[0] & 0x0F;
   match cond{
      0x00 => Some(Opcode::BEQ),
      0x01 => Some(Opcode::BNEQ),
      0x02 => Some(Opcode::B_CARRY_IS_SET),
      0x03 => Some(Opcode::B_CARRY_IS_CLEAR),
      0x04 => Some(Opcode::B_IF_NEGATIVE),
      0x05 => Some(Opcode::B_IF_POSITIVE),
      0x06 => Some(Opcode::B_IF_OVERFLOW),
      0x07 => Some(Opcode::B_IF_NO_OVERFLOW),
      0x08 => Some(Opcode::B_UNSIGNED_HI),
      0x09 => Some(Opcode::B_UNSIGNED_LOW),
      0x0A => Some(Opcode::B_GTE),
      0x0B => Some(Opcode::B_LT),
      0x0C => Some(Opcode::B_GT),
      0x0D => Some(Opcode::B_LTE),
      _ => None
   }
}

#[inline]
const fn bics_mask(hw: &HalfWord)->bool{
   hw[0] == 0x43 && (hw[1] & 0x80 == 0x80)
}

#[inline]
const fn bkpt_mask(hw: &HalfWord)->bool{
   hw[0] == 0xBE
}

#[inline]
const fn blx_mask(hw: &HalfWord)->bool{
   hw[0] == 0x47 && hw[1] & 0x87 == 0x80
}

#[inline]
const fn bx_mask(hw: &HalfWord)->bool{
   hw[0] == 0x47 && hw[1] & 0x87 == 0
}
impl From<&Word> for Opcode{
   fn from(a: &Word)->Self{
      if bl_mask(&a){
         return Opcode::BR_AND_LNK;
      }
      return Opcode::UNDEFINED;
   }
}

#[inline]
const fn bl_mask(bytes: &Word)->bool{
   (bytes[0] & 0xF0 == 0xF0)  && (bytes[2] & 0xC0 == 0xC0)
}

pub fn decode_opcodes(bytes: &[u8])->Vec<Opcode>{
   let mut i: usize = 0;
   let mut opcodes = Vec::new();
   while i < bytes.len(){
      let hw: &[u8;2] = &bytes[i..i+2].try_into().expect("should be 2byte aligned"); 
      let thumb_instruction = Opcode::from(hw);
      if thumb_instruction == Opcode::UNDEFINED{
         if i + 4 > bytes.len(){
            break;
         }
         let word: &[u8;4] = &bytes[i..i+4].try_into().expect("should be 4byte aligned");
         let instruction_32bit = Opcode::from(word);
         opcodes.push(instruction_32bit);
         i += 4;
      }else{
         opcodes.push(thumb_instruction);
         i += 2;
      }
   }
   opcodes
}
