use super::{HalfWord,Word};
#[allow(non_camel_case_types)] #[derive(Debug,PartialEq)]
pub enum Opcode{
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
   BR_AND_LNK,
   BR_LNK_EXCHANGE,
   BR_EXCHANGE,
   CMP_NEG_REG,
   CMP_Imm8,
   CMP_REG_T1,
   CMP_REG_T2,
   DMB,
   DSB,
   XOR_REG,
   ISB,
   LDM,//load from base address sequencialy to register list
   LDM_WRITE_BACK_END, //load from base address sequencialy to register list, 
                      //write the address immediately after address which was last loaded to Rn
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
   MRS,
   MSR,
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

impl From<&HalfWord> for Opcode{
   fn from(a: &HalfWord)->Self{
      if adcs_mask(&a){
         return Opcode::ADCS;
      }

      if addi_mask(&a){
         return Opcode::ADD_Imm3;
      }

      if addi8_mask(&a){
         return Opcode::ADD_Imm8;
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

      if asr_imm5_mask(&a){
         return Opcode::ASRS_Imm5;
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

      if cmn_mask(&a){
         return Opcode::CMP_NEG_REG;
      }

      if cmp_mask(&a){
         return Opcode::CMP_Imm8;
      }

      if cmp_reg_t1_mask(&a){
         return Opcode::CMP_REG_T1;
      }

      if cmp_reg_t2_mask(&a){
         return Opcode::CMP_REG_T2;
      }

      if xor_reg_mask(&a){
         return Opcode::XOR_REG;
      }

      if ldm_mask(&a){
         return Opcode::LDM;
      }

      if ldr_imm5_mask(&a){
         return Opcode::LDR_Imm5;
      }

      if ldr_imm8_mask(&a){
         return Opcode::LDR_SP_Imm8;
      }

      if ldr_pc_imm8_mask(&a){
         return Opcode::LDR_PC_Imm8;
      }

      if ldr_reg_mask(&a){
         return Opcode::LDR_REGS;
      }

      if ldrb_imm5_mask(&a){
         return Opcode::LDRB_Imm5;
      }

      if ldrb_reg_mask(&a){
         return Opcode::LDRB_REGS;
      }

      if ldrh_imm5_mask(&a){
         return Opcode::LDRH_Imm5;
      }

      if ldrh_reg_mask(&a){
         return Opcode::LDRH_REGS;
      }

      if ldrsb_reg_mask(&a){
         return Opcode::LDRSB_REGS;
      }

      if ldrsh_reg_mask(&a){
         return Opcode::LDRSH_REGS;
      }

      if lsl_imm5_mask(&a){
         return Opcode::LSL_Imm5;
      }

      if lsl_reg_mask(&a){
         return Opcode::LSL_REGS;
      }
      if lsr_imm5_mask(&a){
         return Opcode::LSR_Imm5;
      }

      if lsr_reg_mask(&a){
         return Opcode::LSR_REGS;
      }
      
      if mov_imm8_mask(&a){
         return Opcode::MOV_Imm8;
      }

      if mov_regs_t1_mask(&a){
         return Opcode::MOV_REGS_T1;
      }

      if mov_regs_t2_mask(&a){
         return Opcode::MOV_REGS_T2;
      }

      if mul_mask(&a){
         return Opcode::MUL;
      }

      if mvn_mask(&a){
         return Opcode::MVN;
      }

      if nop_mask(&a){
         return Opcode::NOP;
      }

      if orr_mask(&a){
         return Opcode::ORR;
      }

      if pop_mask(&a){
         return Opcode::POP;
      }

      if push_mask(&a){
         return Opcode::PUSH;
      }

      if rev_mask(&a){
         return Opcode::REV;
      }

      if rev16_mask(&a){
         return Opcode::REV_16;
      }

      if revsh_mask(&a){
         return Opcode::REVSH;
      }

      if ror_mask(&a){
         return Opcode::ROR;
      }

      if rsb_mask(&a){
         return Opcode::RSB;
      }

      if sbc_mask(&a){
         return Opcode::SBC;
      }

      if sev_mask(&a){
         return Opcode::SEV;
      }

      if stm_mask(&a){
         return Opcode::STM;
      }

      if str_imm5_mask(&a){
         return Opcode::STR_Imm5;
      }

      if str_imm8_mask(&a){
         return Opcode::STR_Imm8;
      }

      if str_reg_mask(&a){
         return Opcode::STR_REG;
      }

      if strb_imm5_mask(&a){
         return Opcode::STRB_Imm5;
      }

      if strb_reg_mask(&a){
         return Opcode::STRB_REG;
      }

      if strh_imm5_mask(&a){
         return Opcode::STRH_Imm5;
      }

      if strh_reg_mask(&a){
         return Opcode::STRH_REG;
      }

      if sub_imm3_mask(&a){
         return Opcode::SUB_Imm3;
      }

      if sub_imm8_mask(&a){
         return Opcode::SUB_Imm8;
      }

      if sub_reg_mask(&a){
         return Opcode::SUB_REG;
      }

      if sub_sp_imm7_mask(&a){
         return Opcode::SUB_SP_Imm7;
      }

      if sxtb_mask(&a){
         return Opcode::SXTB;
      }

      if sxth_mask(&a){
         return Opcode::SXTH;
      }

      if tst_mask(&a){
         return Opcode::TST;
      }

      if uxtb_mask(&a){
         return Opcode::UXTB;
      }

      if uxth_mask(&a){
         return Opcode::UXTH;
      }

      if wfe_mask(&a){
         return Opcode::WFE;
      }

      if wfi_mask(&a){
         return Opcode::WFI;
      }

      if yield_mask(&a){
         return Opcode::YIELD;
      }

      Opcode::UNDEFINED
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
      return Some(Opcode::B_ALWAYS);
   }

   if hw[1] & 0xF0 != 0xD0{
      return None;
   }

   let cond = hw[1] & 0x0F;
   match cond{
      0x00 => Some(Opcode::BEQ),
      0x01 => Some(Opcode::BNEQ),
      0x02 => Some(Opcode::B_CARRY_IS_SET),
      0x03 => Some(Opcode::B_CARRY_IS_CLEAR),
      0x04 => Some(Opcode::B_IF_NEGATIVE),
      0x05 => Some(Opcode::B_IF_POSITIVE),
      0x06 => Some(Opcode::B_IF_OVERFLOW),
      0x07 => Some(Opcode::B_IF_NO_OVERFLOW),
      0x08 => Some(Opcode::B_UNSIGNED_HIGHER),
      0x09 => Some(Opcode::B_UNSIGNED_LOWER_OR_SAME),
      0x0A => Some(Opcode::B_GTE),
      0x0B => Some(Opcode::B_LT),
      0x0C => Some(Opcode::B_GT),
      0x0D => Some(Opcode::B_LTE),
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
   let translated_value = super::from_arm_bytes_16b(*hw);
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
   super::from_arm_bytes_16b(*hw) == 0xBF40
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
         return Opcode::BR_AND_LNK;
      }
      
      if udf_word_mask(&a){
         return Opcode::UNDEFINED;
      }
      return Opcode::UNDEFINED;
   }
}

#[inline]
const fn bl_mask(bytes: &Word)->bool{
   (bytes[1] & 0xF8 == 0xF0)  && (bytes[3] & 0xD0 == 0xD0)
}

#[inline]
const fn udf_word_mask(bytes: &Word)->bool{
   let hw_slice: HalfWord = [bytes[0],bytes[1]];
   (super::from_arm_bytes_16b(hw_slice) & 0xFFF0 == 0xF7F0) && (bytes[3] & 0xF0 == 0xA0)
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
