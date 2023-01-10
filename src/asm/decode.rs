use super::{HalfWord,Word};
use crate::binutils::from_arm_bytes_16b;
#[allow(non_camel_case_types)] #[derive(Debug,PartialEq)]
pub enum B16{
   UNDEFINED,
   ADCS,
   ADD_Imm3,
   ADD_Imm8,
   ADDS_REG,
   ADDS_REG_T2,
   ADD_REG_SP_IMM8,
   INCR_SP_BY_IMM7,
   INCR_SP_BY_REG,
   INCR_REG_BY_SP,
   ADR,
   ANDS,
   ASRS_Imm5,
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
   B_UNSIGNED_HIGHER,
   B_UNSIGNED_LOWER_OR_SAME,
   B_GTE,
   B_LTE,
   B_GT,
   B_LT,
   B_ALWAYS,
   BIT_CLEAR_REGISTER,
   BREAKPOINT,
   BR_LNK_EXCHANGE,
   BR_EXCHANGE,
   CMP_NEG_REG,
   CMP_Imm8,
   CMP_REG_T1,
   CMP_REG_T2,
   CPS,
   XOR_REG,
   LDM,//load from base address sequencialy to register list
   LDR_Imm5,
   LDR_SP_Imm8,
   LDR_PC_Imm8,
   LDR_REGS,
   LDRB_Imm5,
   LDRB_REGS,
   LDRH_Imm5,
   LDRH_REGS,
   LDRSB_REGS,
   LDRSH_REGS,
   LSL_Imm5,
   LSL_REGS,
   LSR_Imm5,
   LSR_REGS,
   MOV_Imm8,
   MOV_REGS_T1,
   MOV_REGS_T2,
   MUL,
   MVN,
   NOP,
   ORR,
   POP,
   PUSH,
   REV,
   REV_16,
   REVSH,
   ROR,
   RSB,
   SBC,
   SEV,
   STM,
   STR_Imm5,
   STR_Imm8,
   STR_REG,
   STRB_Imm5,
   STRB_REG,
   STRH_Imm5,
   STRH_REG,
   SUB_Imm3,
   SUB_Imm8,
   SUB_REG,
   SUB_SP_Imm7,
   SXTB,
   SXTH,
   TST,
   UXTB,
   UXTH,
   WFE,
   WFI,
   YIELD
}

#[allow(non_camel_case_types)] #[derive(Debug,PartialEq)]
pub enum B32{
   UNDEFINED,
   BR_AND_LNK,
   DMB,
   DSB,
   ISB,
   MRS,
   MSR,
}
#[allow(non_camel_case_types)] #[derive(Debug,PartialEq)]
pub enum Opcode{
   _32Bit(B32),
   _16Bit(B16)
} 

impl From<&HalfWord> for Opcode{
   fn from(hw: &HalfWord)->Self{
      let mut code = match hw{
         [0x00,0xBF] => Some(Opcode::_16Bit(B16::NOP)),
         [0x40,0xBF] => Some(Opcode::_16Bit(B16::SEV)),
         [0x20,0xBF] => Some(Opcode::_16Bit(B16::WFE)),
         [0x30,0xBF] => Some(Opcode::_16Bit(B16::WFI)),
         [0x10,0xBF] => Some(Opcode::_16Bit(B16::YIELD)),
         _ => None
      };

      if code.is_some(){
         return code.unwrap();
      }


      let op_code = (hw[1] & 0xFC) >> 2;
      match op_code{
         0x0 ..=0xF => shift_add_sub_mv_cmpr(hw),
         0x10 => data_proccess(hw),
         0x11 => special_data(hw),
         0x12 | 0x13 => Opcode::_16Bit(B16::LDR_PC_Imm8),
         0x14 ..=0x27 => load_store_single(hw),
         0x28 | 0x29 => Opcode::_16Bit(B16::ADR),
         0x2A | 0x2B => Opcode::_16Bit(B16::ADD_REG_SP_IMM8),
         0x2C ..=0x2F => misc(hw),
         0x30 | 0x31 => Opcode::_16Bit(B16::STM),
         0x32 | 0x33 => Opcode::_16Bit(B16::LDM),
         0x34 ..=0x37 => cond_branch(hw),
         0x38 | 0x39 => Opcode::_16Bit(B16::B_ALWAYS),
         _ => Opcode::_16Bit(B16::UNDEFINED),
      }
   }
}

#[inline]
fn data_proccess(hw: &HalfWord)->Opcode{
   let native = from_arm_bytes_16b(*hw);
   let code = (0x03C0 & native) >> 6;
   match code{
      0 => Opcode::_16Bit(B16::ANDS),
      1 => Opcode::_16Bit(B16::XOR_REG),
      2 => Opcode::_16Bit(B16::LSL_REGS),
      3 => Opcode::_16Bit(B16::LSR_REGS),
      4 => Opcode::_16Bit(B16::ASRS_REG),
      5 => Opcode::_16Bit(B16::ADCS),
      6 => Opcode::_16Bit(B16::SBC),
      7 => Opcode::_16Bit(B16::ROR),
      8 => Opcode::_16Bit(B16::TST),
      9 => Opcode::_16Bit(B16::RSB),
      10 => Opcode::_16Bit(B16::CMP_REG_T1),
      11 => Opcode::_16Bit(B16::CMP_NEG_REG),
      12 => Opcode::_16Bit(B16::ORR),
      13 => Opcode::_16Bit(B16::MUL),
      14 => Opcode::_16Bit(B16::BIT_CLEAR_REGISTER),
      15 => Opcode::_16Bit(B16::MVN),
      _ => unreachable!()
   }
}

fn special_data(hw: &HalfWord)->Opcode{
   let native = from_arm_bytes_16b(*hw);
   let code = (0x03C0 & native) >> 6;
   match code{
      0 => Opcode::_16Bit(B16::ADDS_REG_T2),
      1 => Opcode::_16Bit(B16::ADDS_REG_T2),
      2 => Opcode::_16Bit(B16::ADDS_REG_T2),
      3 => Opcode::_16Bit(B16::ADDS_REG_T2),
      4 => Opcode::_16Bit(B16::UNDEFINED),
      5 => Opcode::_16Bit(B16::CMP_REG_T2),
      6 => Opcode::_16Bit(B16::CMP_REG_T2),
      7 => Opcode::_16Bit(B16::CMP_REG_T2),
      8 => Opcode::_16Bit(B16::MOV_REGS_T1),
      9 => Opcode::_16Bit(B16::MOV_REGS_T1),
      10 => Opcode::_16Bit(B16::MOV_REGS_T1),
      11 => Opcode::_16Bit(B16::MOV_REGS_T1),
      12 => Opcode::_16Bit(B16::BR_EXCHANGE),
      13 => Opcode::_16Bit(B16::BR_EXCHANGE),
      14 => Opcode::_16Bit(B16::BR_LNK_EXCHANGE),
      15 => Opcode::_16Bit(B16::BR_LNK_EXCHANGE),
      _ => unreachable!()
   }
}

#[inline]
fn load_store_single(hw: &HalfWord)->Opcode{
   println!("{:#x}",hw[1]);
   let op_a = (hw[1] & 0xF0) >> 4;
   let op_b = (hw[1] & 0x0E) >> 1;

   println!("a:{},b:{}",op_a,op_b);
   match op_a{
      5 => {
         match op_b{
            0 => Opcode::_16Bit(B16::STR_REG),
            1 => Opcode::_16Bit(B16::STRH_REG),
            2 => Opcode::_16Bit(B16::STRB_REG),
            3 => Opcode::_16Bit(B16::LDRSB_REGS),
            4 => Opcode::_16Bit(B16::LDR_REGS),
            5 => Opcode::_16Bit(B16::LDRH_REGS),
            6 => Opcode::_16Bit(B16::LDRB_REGS),
            7 => Opcode::_16Bit(B16::LDRSH_REGS),
            _ => unreachable!(),
         }
      },
      6 => {
         match op_b{
            0 | 1 | 2 | 3 => Opcode::_16Bit(B16::STR_Imm5),
            4 | 5 | 6 | 7 => Opcode::_16Bit(B16::LDR_Imm5),
            _ => unreachable!(),
         }
      },
      7 => {
         match op_b{
            0 | 1 | 2 | 3 => Opcode::_16Bit(B16::STRB_Imm5),
            4 | 5 | 6 | 7 => Opcode::_16Bit(B16::LDRB_Imm5),
            _ => unreachable!()
         }
      },
      8 => {
         match op_b{
            0 | 1 | 2 | 3 => Opcode::_16Bit(B16::STRH_Imm5),
            4 | 5 | 6 | 7 => Opcode::_16Bit(B16::LDRH_Imm5),
            _ => unreachable!()
         }
      },
      9 => {
         match op_b{
            0 | 1 | 2 | 3 => Opcode::_16Bit(B16::STR_Imm8),
            4 | 5 | 6 | 7 => Opcode::_16Bit(B16::LDR_SP_Imm8),
            _ => unreachable!()
         }
      },
      _ => unreachable!()
   }
}

#[inline]
fn misc(hw: &HalfWord)->Opcode{
   let code = (from_arm_bytes_16b(*hw) & 0x0FE0) >> 5;
   match code{
      0 | 1 | 2 | 3 => Opcode::_16Bit(B16::INCR_SP_BY_IMM7),
      4 | 5 | 6 | 7 => Opcode::_16Bit(B16::SUB_SP_Imm7),
      16 | 17 => Opcode::_16Bit(B16::SXTH),
      18 | 19 => Opcode::_16Bit(B16::SXTB),
      0x14 | 0x15 => Opcode::_16Bit(B16::UXTH),
      0x16 | 0x17 => Opcode::_16Bit(B16::UXTB),
      0x20 ..=0x2F => Opcode::_16Bit(B16::PUSH),
      0x33 => Opcode::_16Bit(B16::CPS),
      0x50 | 0x51 => Opcode::_16Bit(B16::REV),
      0x52 | 0x53 => Opcode::_16Bit(B16::REV_16),
      0x56 | 0x57 => Opcode::_16Bit(B16::REVSH),
      0x60 ..= 0x6F => Opcode::_16Bit(B16::POP),
      0x70 ..=0x77 => Opcode::_16Bit(B16::BREAKPOINT),
      _ => unreachable!(),
   }
}

fn cond_branch(hw: &HalfWord)->Opcode{
   let cond = hw[1] & 0x0F;
   match cond{
      0x00 => Opcode::_16Bit(B16::BEQ),
      0x01 => Opcode::_16Bit(B16::BNEQ),
      0x02 => Opcode::_16Bit(B16::B_CARRY_IS_SET),
      0x03 => Opcode::_16Bit(B16::B_CARRY_IS_CLEAR),
      0x04 => Opcode::_16Bit(B16::B_IF_NEGATIVE),
      0x05 => Opcode::_16Bit(B16::B_IF_POSITIVE),
      0x06 => Opcode::_16Bit(B16::B_IF_OVERFLOW),
      0x07 => Opcode::_16Bit(B16::B_IF_NO_OVERFLOW),
      0x08 => Opcode::_16Bit(B16::B_UNSIGNED_HIGHER),
      0x09 => Opcode::_16Bit(B16::B_UNSIGNED_LOWER_OR_SAME),
      0x0A => Opcode::_16Bit(B16::B_GTE),
      0x0B => Opcode::_16Bit(B16::B_LT),
      0x0C => Opcode::_16Bit(B16::B_GT),
      0x0D => Opcode::_16Bit(B16::B_LTE),
      0x0E => Opcode::_16Bit(B16::UNDEFINED),
      0x0F => Opcode::_16Bit(B16::SVC),
      _ => unreachable!()
   }
}

fn shift_add_sub_mv_cmpr(hw: &HalfWord)->Opcode{
   let code = (hw[1] & 0x3E) >> 1;
   match code{
      0 ..=3 => Opcode::_16Bit(B16::LSL_Imm5),
      4 ..=7 => Opcode::_16Bit(B16::LSR_Imm5),
      0x08 ..=0x0B => Opcode::_16Bit(B16::ASRS_Imm5),
      0x0C => Opcode::_16Bit(B16::ADDS_REG),
      0x0D => Opcode::_16Bit(B16::SUB_REG),
      0x0E => Opcode::_16Bit(B16::ADD_Imm3),
      0x0F => Opcode::_16Bit(B16::SUB_Imm3),
      0x10 ..=0x13 => Opcode::_16Bit(B16::MOV_Imm8),
      0x14 ..=0x17 => Opcode::_16Bit(B16::CMP_Imm8),
      0x18 ..=0x1B => Opcode::_16Bit(B16::ADD_Imm8),
      0x1C ..=0x1F => Opcode::_16Bit(B16::SUB_Imm8),
      _ => unreachable!(),
   }
}

#[inline]
const fn adcs_mask(hw: &HalfWord)->bool{
   hw[1] == 0x41 && (hw[0] & 0xC0 == 0x40)
}

#[inline]
fn addi_mask(hw: &HalfWord)->bool{
   hw[1] & 0xFE == 0x1C
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
   hw[1] == 0x40 && (hw[0] & 0xC0 == 0)
}

#[inline]
const fn asr_imm5_mask(hw: &HalfWord)->bool{
   hw[1] & 0xF8 == 0x10
}

#[inline]
const fn asrs_reg_mask(hw: &HalfWord)->bool{
   hw[1] == 0x41 && (hw[0] & 0xC0 == 0)
}

#[inline]
const fn ukwn_branch_mask(hw: &HalfWord)->bool{
   hw[1] == 0xDE
}

#[inline]
const fn svc_mask(hw: &HalfWord)->bool{
   hw[1] == 0xDF
}

#[inline]
const fn try_get_branch(hw: &HalfWord)->Option<Opcode>{
   if hw[1] & 0xF8 == 0xE0 {
      return Some(Opcode::_16Bit(B16::B_ALWAYS));
   }

   if hw[1] & 0xF0 != 0xD0{
      return None;
   }

   let cond = hw[1] & 0x0F;
   match cond{
      0x00 => Some(Opcode::_16Bit(B16::BEQ)),
      0x01 => Some(Opcode::_16Bit(B16::BNEQ)),
      0x02 => Some(Opcode::_16Bit(B16::B_CARRY_IS_SET)),
      0x03 => Some(Opcode::_16Bit(B16::B_CARRY_IS_CLEAR)),
      0x04 => Some(Opcode::_16Bit(B16::B_IF_NEGATIVE)),
      0x05 => Some(Opcode::_16Bit(B16::B_IF_POSITIVE)),
      0x06 => Some(Opcode::_16Bit(B16::B_IF_OVERFLOW)),
      0x07 => Some(Opcode::_16Bit(B16::B_IF_NO_OVERFLOW)),
      0x08 => Some(Opcode::_16Bit(B16::B_UNSIGNED_HIGHER)),
      0x09 => Some(Opcode::_16Bit(B16::B_UNSIGNED_LOWER_OR_SAME)),
      0x0A => Some(Opcode::_16Bit(B16::B_GTE)),
      0x0B => Some(Opcode::_16Bit(B16::B_LT)),
      0x0C => Some(Opcode::_16Bit(B16::B_GT)),
      0x0D => Some(Opcode::_16Bit(B16::B_LTE)),
      _ => None
   }
}

#[inline]
const fn bics_mask(hw: &HalfWord)->bool{
   hw[1] == 0x43 && (hw[0] & 0xC0 ==  0x80)
}

#[inline]
const fn bkpt_mask(hw: &HalfWord)->bool{
   hw[1] == 0xBE
}

#[inline]
const fn blx_mask(hw: &HalfWord)->bool{
   hw[1] == 0x47 && hw[0] & 0x87 == 0x80
}

#[inline]
const fn bx_mask(hw: &HalfWord)->bool{
   hw[1] == 0x47 && hw[0] & 0x87 == 0
}

#[inline]
const fn cmn_mask(hw: &HalfWord)->bool{
   hw[1] == 0x42 && (hw[0] & 0xC0 == 0xC0)
}

#[inline]
const fn cmp_mask(hw: &HalfWord)->bool{
   hw[1] & 0xF8 == 0x28
}

#[inline]
const fn cmp_reg_t1_mask(hw: &HalfWord)->bool{
   hw[1] == 0x42 && hw[0] & 0xC0 == 0x80
}

#[inline]
const fn cmp_reg_t2_mask(hw: &HalfWord)->bool{
   hw[1] == 0x45 
}

#[inline]
const fn cps_mask(hw: &HalfWord)->bool{
   hw[1] == 0xB6 && ((hw[0] & 0x60) == 0x60)
}

#[inline]
const fn xor_reg_mask(hw: &HalfWord)->bool{
   hw[1] == 0x40 && (hw[0] & 0xC0 == 0x40)
}

#[inline]
const fn ldm_mask(hw: &HalfWord)->bool{
   hw[1] & 0xF8 == 0xC8 
}

#[inline]
const fn ldr_imm5_mask(hw: &HalfWord)->bool{
   hw[1] & 0xF8 == 0x68
}

#[inline]
const fn ldr_imm8_mask(hw: &HalfWord)->bool{
   hw[1] & 0xF8 == 0x98
}

#[inline]
const fn ldr_pc_imm8_mask(hw: &HalfWord)->bool{
   hw[1] & 0xF8 == 0x48 
}

#[inline]
const fn ldr_reg_mask(hw: &HalfWord)->bool{
   hw[1] & 0xFE == 0x58 
}

#[inline]
const fn ldrb_imm5_mask(hw: &HalfWord)->bool{
   hw[1] & 0xF8 == 0x78
}

#[inline]
const fn ldrb_reg_mask(hw: &HalfWord)->bool{
   hw[1] & 0xFE == 0x5C
}

#[inline]
const fn ldrh_imm5_mask(hw: &HalfWord)->bool{
   hw[1] & 0xF8 == 0x88
}

#[inline]
const fn ldrh_reg_mask(hw: &HalfWord)->bool{
   hw[1] & 0xFE == 0x5A
}

#[inline]
const fn ldrsb_reg_mask(hw: &HalfWord)->bool{
   hw[1] & 0xFE == 0x56
}

#[inline]
const fn ldrsh_reg_mask(hw: &HalfWord)->bool{
   hw[1] & 0xFE == 0x5E
}

#[inline]
const fn lsl_imm5_mask(hw: &HalfWord)->bool{
   let translated_value = from_arm_bytes_16b(*hw);
   hw[1] & 0xF8 == 0 && (translated_value & 0x07C0 != 0)
}

#[inline]
const fn lsl_reg_mask(hw: &HalfWord)->bool{
   hw[1] == 0x40 && (hw[0] & 0xC0 == 0x80)
}


#[inline]
const fn lsr_imm5_mask(hw: &HalfWord)->bool{
   hw[1] & 0xF8 == 0x08 
}

#[inline]
const fn lsr_reg_mask(hw: &HalfWord)->bool{
   hw[1] == 0x40 && (hw[0] & 0xC0 == 0xC0)
}

#[inline]
const fn mov_imm8_mask(hw: &HalfWord)->bool{
   hw[1] & 0xF8 == 0x20
}

#[inline]
const fn mov_regs_t1_mask(hw: &HalfWord)->bool{
   hw[1] == 0x46 
}

#[inline]
const fn mov_regs_t2_mask(hw: &HalfWord)->bool{
   hw[1] == 0 && (hw[0] & 0xC0 == 0)
}

#[inline]
const fn mul_mask(hw: &HalfWord)->bool{
   hw[1] == 0x43 && (hw[0] & 0xC0 == 0x40)
}

#[inline]
const fn mvn_mask(hw: &HalfWord)->bool{
   hw[1] == 0x43 && (hw[0] & 0xC0 == 0xC0)
}

#[inline]
const fn nop_mask(hw: &HalfWord)->bool{
   hw[1] == 0xBF && hw[0] == 0
}

#[inline]
const fn orr_mask(hw: &HalfWord)->bool{
   hw[1] == 0x43 && (hw[0] & 0xC0 == 0)
}

#[inline]
const fn pop_mask(hw: &HalfWord)->bool{
   hw[1] & 0xFE == 0xBC 
}

#[inline]
const fn push_mask(hw: &HalfWord)->bool{
   hw[1] & 0xFE == 0xB4
}

#[inline]
const fn rev_mask(hw: &HalfWord)->bool{
   hw[1] == 0xBA && (hw[0] & 0xC0 == 0)
}

#[inline]
const fn rev16_mask(hw: &HalfWord)->bool{
   hw[1] == 0xBA && (hw[0] & 0xC0 == 0x40)
}

#[inline]
const fn revsh_mask(hw: &HalfWord)->bool{
   hw[1] == 0xBA && (hw[0] & 0xC0 == 0xC0)
}

#[inline]
const fn ror_mask(hw: &HalfWord)->bool{
   hw[1] == 0x41 && (hw[0] & 0xC0 == 0xC0)
}

#[inline]
const fn rsb_mask(hw: &HalfWord)->bool{
   hw[1] == 0x42 && (hw[0] & 0xC0 == 0x40)
}

#[inline]
const fn sbc_mask(hw: &HalfWord)->bool{
   hw[1] == 0x41 && (hw[0] & 0xC0 == 0x80)
}

#[inline]
const fn sev_mask(hw: &HalfWord)->bool{
   from_arm_bytes_16b(*hw) == 0xBF40
}

#[inline]
const fn stm_mask(hw: &HalfWord)->bool{
   hw[1] & 0xF8 == 0xC0
}

#[inline]
const fn str_imm5_mask(hw: &HalfWord)->bool{
   hw[1] & 0xF8  == 0x60 
}

#[inline]
const fn str_imm8_mask(hw: &HalfWord)->bool{
   hw[1] & 0xF8  == 0x90 
}

#[inline]
const fn str_reg_mask(hw: &HalfWord)->bool{
   hw[1] & 0xFE == 0x50 
}

#[inline]
const fn strb_imm5_mask(hw: &HalfWord)->bool{
   hw[1] & 0xF8 == 0x70
}

#[inline]
const fn strb_reg_mask(hw: &HalfWord)->bool{
   hw[1] & 0xFE == 0x54
}

#[inline]
const fn strh_imm5_mask(hw: &HalfWord)->bool{
   hw[1] & 0xF8 == 0x80
}

#[inline]
const fn strh_reg_mask(hw: &HalfWord)->bool{
   hw[1] & 0xFE == 0x52
}

#[inline]
const fn sub_imm3_mask(hw: &HalfWord)->bool{
   hw[1] & 0xFE == 0x1E 
}

#[inline]
const fn sub_imm8_mask(hw: &HalfWord)->bool{
   hw[1] & 0xF8 == 0x38 
}

#[inline]
const fn sub_reg_mask(hw: &HalfWord)->bool{
   hw[1] & 0xFE == 0x1A
}

#[inline]
const fn sub_sp_imm7_mask(hw: &HalfWord)->bool{
   hw[1] == 0xB0 && (hw[0] & 0x80 == 0x80)
}

#[inline]
const fn sxtb_mask(hw: &HalfWord)->bool{
   hw[1] == 0xB2 && (hw[0] & 0xC0 == 0x40)
}

#[inline]
const fn sxth_mask(hw: &HalfWord)->bool{
   hw[1] == 0xB2 && (hw[0] & 0xC0 == 0)
}

#[inline]
const fn tst_mask(hw: &HalfWord)->bool{
   hw[1] == 0x42 && (hw[0] & 0xC0 == 0)
}

#[inline]
const fn uxtb_mask(hw: &HalfWord)->bool{
   hw[1] == 0xB2 && (hw[0] & 0xC0 == 0xC0)
}

#[inline]
const fn uxth_mask(hw: &HalfWord)->bool{
   hw[1] == 0xB2 && (hw[0] & 0xC0 == 0x80)
}

#[inline]
const fn wfe_mask(hw: &HalfWord)->bool{
   hw[1] == 0xBF && hw[0] == 0x20
}

#[inline]
const fn wfi_mask(hw: &HalfWord)->bool{
   hw[1] == 0xBF && hw[0] == 0x30
}

#[inline]
const fn yield_mask(hw: &HalfWord)->bool{
   hw[1] == 0xBF && hw[0] == 0x10
}

impl From<&Word> for Opcode{
   fn from(a: &Word)->Self{
      if bl_mask(&a){
         return Opcode::_32Bit(B32::BR_AND_LNK);
      }
      
      if udf_word_mask(&a){
         return Opcode::_32Bit(B32::UNDEFINED);
      }
      return Opcode::_32Bit(B32::UNDEFINED);
   }
}

#[inline]
const fn bl_mask(bytes: &Word)->bool{
   (bytes[1] & 0xF8 == 0xF0)  && (bytes[3] & 0xD0 == 0xD0)
}

#[inline]
const fn udf_word_mask(bytes: &Word)->bool{
   let hw_slice: HalfWord = [bytes[0],bytes[1]];
   (from_arm_bytes_16b(hw_slice) & 0xFFF0 == 0xF7F0) && (bytes[3] & 0xF0 == 0xA0)
}

pub fn decode_opcodes(bytes: &[u8])->Vec<Opcode>{
   let mut i: usize = 0;
   let mut opcodes = Vec::new();
   while i < bytes.len(){
      let hw: &[u8;2] = &bytes[i..i+2].try_into().expect("should be 2byte aligned"); 
      let thumb_instruction = Opcode::from(hw);
      if thumb_instruction == Opcode::_16Bit(B16::UNDEFINED){
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

