use super::{HalfWord,Word};

#[allow(non_camel_case_types)]
pub enum Opcode{
   UNDEFINED,
   ADCS,
   ADDI,
   ADDI8,
   ADDS_REG,
   ADD_REG,
   ADD_SPI,
   INCR_SP_BY,
   INCR_SP_BY_REG,
   ADD_REG_SP,
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

impl From<HalfWord> for Opcode{
   fn from(a: HalfWord)->Self{
      if adsc_mask(&a){
         return Opcode::ADCS;
      }

      if addi_mask(&a){
         return Opcode::ADDI;
      }

      if addi8_mask(&a){
         return Opcode::ADDI8;
      }

      if adds_reg_mask(&a){
         return Opcode::ADDS_REG;
      }

      if add_reg_sp_mask(&a){
         return Opcode::ADD_REG_SP;
      }

      if incr_sp_by_reg_mask(&a){
         return Opcode::INCR_SP_BY_REG;
      }

      if add_reg_mask(&a){
         return Opcode::ADD_REG;
      }

      if add_sp_with_immediate_and_store_in_reg_mask(&a){
         return Opcode::ADD_SPI;
      }

      if incr_sp_by_mask(&a){
         return Opcode::INCR_SP_BY;
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
const fn adsc_mask(hw: &HalfWord)->bool{
   (hw[0] == 0x41) && (hw[1] == 0x40)
}

#[inline]
const fn addi_mask(hw: &HalfWord)->bool{
   hw[0] == 0x1C || hw[0] == 0x1D
}

#[inline]
const fn addi8_mask(hw: &HalfWord)->bool{
   hw[0] >> 4 == 0x03 && (hw[0] | (1 << 3) != hw[0])
} 

#[inline]
const fn adds_reg_mask(hw: &HalfWord)->bool{
   hw[0] == 0x18 || hw[0] == 0x19
}

#[inline]
const fn add_reg_mask(hw: &HalfWord)->bool{
   hw[0] == 0x44
}

#[inline]
const fn add_sp_with_immediate_and_store_in_reg_mask(hw: &HalfWord)->bool{
   hw[0] >> 4 == 0x0A  && (hw[0] & (1 << 3) > 0)
}

#[inline]
const fn incr_sp_by_mask(hw: &HalfWord)->bool{
   hw[0] == 0xB0 && (hw[1] >> 7 == 0)
}

#[inline]
const fn add_reg_sp_mask(hw: &HalfWord)->bool{
   hw[0] == 0x44 && (hw[1] & 0x68 == 0x68)
}

#[inline]
const fn incr_sp_by_reg_mask(hw: &HalfWord)->bool{
   hw[0] == 0x44 && (hw[1] & 0x85 == 0x85)
}

#[inline]
const fn adr_mask(hw: &HalfWord)->bool{
   (hw[0] >> 4) == 0x0A && ((hw[0] | 1 << 3)!= hw[0])
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
impl From<Word> for Opcode{
   fn from(a: Word)->Self{
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
