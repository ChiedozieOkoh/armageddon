use core::fmt;

use super::{HalfWord,Word};
use crate::binutils::from_arm_bytes_16b;

use crate::{dbg_ln, dbg_print};

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

impl fmt::Display for B16{
   fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
      match self{
         B16::UNDEFINED => write!(f,"UDF"),
         B16::ADCS => write!(f,"ADCS"),
         B16::ADD_Imm3 => write!(f,"ADD"),
         B16::ADD_Imm8 => write!(f,"ADD"),
         B16::ADDS_REG => write!(f,"ADD"),
         B16::ADDS_REG_T2 => write!(f,"ADD"),
         B16::ADD_REG_SP_IMM8 => write!(f,"ADD"),
         B16::INCR_SP_BY_IMM7 => write!(f,"ADD"),
         B16::INCR_SP_BY_REG => write!(f,"ADD"),
         B16::ADR => write!(f,"ADR"),
         B16::ANDS => write!(f,"AND"),
         B16::ASRS_Imm5 => write!(f,"ASR"),
         B16::ASRS_REG => write!(f,"ASR"),
         B16::SVC => write!(f,"SVC"),
         B16::BEQ => write!(f,"BEQ"),
         B16::BNEQ => write!(f,"BNE"),
         B16::B_CARRY_IS_SET => write!(f,"BCS"),
         B16::B_CARRY_IS_CLEAR => write!(f,"BCC"),
         B16::B_IF_NEGATIVE => write!(f,"BMI"),
         B16::B_IF_POSITIVE => write!(f,"BPL"),
         B16::B_IF_OVERFLOW => write!(f,"BVS"),
         B16::B_IF_NO_OVERFLOW => write!(f,"BVC"),
         B16::B_UNSIGNED_HIGHER => write!(f,"BHI"),
         B16::B_UNSIGNED_LOWER_OR_SAME => write!(f,"BLS"),
         B16::B_GTE => write!(f,"BGE"),
         B16::B_LTE => write!(f,"BLE"),
         B16::B_GT => write!(f,"BGT"),
         B16::B_LT => write!(f,"BLT"),
         B16::B_ALWAYS => write!(f,"BAL"),
         B16::BIT_CLEAR_REGISTER => write!(f,"BIC"),
         B16::BREAKPOINT => write!(f,"BKPT"),
         B16::BR_LNK_EXCHANGE => write!(f,"BLX"),
         B16::BR_EXCHANGE => write!(f,"BX"),
         B16::CMP_NEG_REG => write!(f,"CMN"),
         B16::CMP_Imm8 => write!(f,"CMP"),
         B16::CMP_REG_T1 => write!(f,"CMP"),
         B16::CMP_REG_T2 => write!(f,"CMP"),
         B16::CPS => Ok(()),//the instruction name is known by decode_operands
         B16::XOR_REG => write!(f,"EOR"),
         B16::LDM => write!(f,"LDM"),//load from base address sequencialy to register list
         B16::LDR_Imm5 => write!(f,"LDR"),
         B16::LDR_SP_Imm8 => write!(f,"LDR"),
         B16::LDR_PC_Imm8 => write!(f,"LDR"),
         B16::LDR_REGS => write!(f,"LDR"),
         B16::LDRB_Imm5 => write!(f,"LDRB"),
         B16::LDRB_REGS => write!(f,"LDRB"),
         B16::LDRH_Imm5 => write!(f,"LDRH"),
         B16::LDRH_REGS => write!(f,"LDRH"),
         B16::LDRSB_REGS => write!(f,"LDRSB"),
         B16::LDRSH_REGS => write!(f,"LDRSH"),
         B16::LSL_Imm5 => write!(f,"LSL"),
         B16::LSL_REGS => write!(f,"LSL"),
         B16::LSR_Imm5 => write!(f,"LSR"),
         B16::LSR_REGS => write!(f,"LSR"),
         B16::MOV_Imm8 => write!(f,"MOV"),
         B16::MOV_REGS_T1 => write!(f,"MOV"),
         B16::MOV_REGS_T2 => write!(f,"MOV"),
         B16::MUL => write!(f,"MUL"),
         B16::MVN => write!(f,"MVN"),
         B16::NOP => write!(f,"NOP"),
         B16::ORR => write!(f,"ORR"),
         B16::POP => write!(f,"POP"),
         B16::PUSH => write!(f,"PUSH"),
         B16::REV => write!(f,"REV"),
         B16::REV_16 => write!(f,"REV16"),
         B16::REVSH => write!(f,"REVSH"),
         B16::ROR => write!(f,"ROR"),
         B16::RSB => write!(f,"RSB"),
         B16::SBC => write!(f,"SBC"),
         B16::SEV => write!(f,"SEV"),
         B16::STM => write!(f,"STM"),
         B16::STR_Imm5 => write!(f,"STR"),
         B16::STR_Imm8 => write!(f,"STR"),
         B16::STR_REG => write!(f,"STR"),
         B16::STRB_Imm5 => write!(f,"STRB"),
         B16::STRB_REG => write!(f,"STRB"),
         B16::STRH_Imm5 => write!(f,"STRH"),
         B16::STRH_REG => write!(f,"STRH"),
         B16::SUB_Imm3 => write!(f,"SUB"),
         B16::SUB_Imm8 => write!(f,"SUB"),
         B16::SUB_REG => write!(f,"SUB"),
         B16::SUB_SP_Imm7 => write!(f,"SUB"),
         B16::SXTB => write!(f,"SXTB"),
         B16::SXTH => write!(f,"SXTH"),
         B16::TST => write!(f,"TST"),
         B16::UXTB => write!(f,"UXTB"),
         B16::UXTH => write!(f,"UXTH"),
         B16::WFE => write!(f,"WFE"),
         B16::WFI => write!(f,"WFI"),
         B16::YIELD => write!(f,"YIELD")
     }
   }
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

impl fmt::Display for B32{
   fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
      match self{
         B32::UNDEFINED => write!(f,"UDF.W"),
         B32::BR_AND_LNK => write!(f,"BL"),
         B32::DMB => write!(f,"DMB"),
         B32::DSB => write!(f,"DSB"),
         B32::ISB => write!(f,"ISB"),
         B32::MRS => write!(f,"MRS"),
         B32::MSR => write!(f,"MSR")
      }
   }
}

#[allow(non_camel_case_types)] #[derive(Debug,PartialEq)]
pub enum Opcode{
   _32Bit(B32),
   _16Bit(B16)
} 

impl fmt::Display for Opcode{
   fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
      match self{
         Opcode::_32Bit(opc) => write!(f,"{}",opc),
         Opcode::_16Bit(opc) => write!(f,"{}",opc) 
      }       
   }
}

impl From<HalfWord> for Opcode{
   fn from(hw: HalfWord)->Self{
      let code = match hw{
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

#[derive(Debug,PartialEq)]
pub enum InstructionSize{
   B16,
   B32
}

impl InstructionSize{
   #[inline]
   pub fn in_bytes(self)->u32{
      match self{
        InstructionSize::B16 => 2,
        InstructionSize::B32 => 4,
    }
   }
}

#[inline]
pub fn instruction_size(opcode: [u8;2])->InstructionSize{
   let header = opcode[1] & 0xF8;
   match header{
      0xE8 | 0xF0 | 0xF8 => InstructionSize::B32,
      _ => InstructionSize::B16
   }
}

#[inline]
fn data_proccess(hw: HalfWord)->Opcode{
   let native = from_arm_bytes_16b(hw);
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

fn special_data(hw: HalfWord)->Opcode{
   let native = from_arm_bytes_16b(hw);
   if native & 0xFF87 == 0x4487{
      return Opcode::_16Bit(B16::INCR_SP_BY_REG);
   }
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
fn load_store_single(hw: HalfWord)->Opcode{
   dbg_ln!("{:#x}",hw[1]);
   let op_a = (hw[1] & 0xF0) >> 4;
   let op_b = (hw[1] & 0x0E) >> 1;

   dbg_ln!("a:{},b:{}",op_a,op_b);
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
fn misc(hw: HalfWord)->Opcode{
   let code = (from_arm_bytes_16b(hw) & 0x0FE0) >> 5;
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

fn cond_branch(hw: HalfWord)->Opcode{
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

fn shift_add_sub_mv_cmpr(hw: HalfWord)->Opcode{
   let code = (hw[1] & 0x3E) >> 1;
   match code{
      0 ..=3 => {
         if from_arm_bytes_16b(hw) & 0x01C0 == 0{
            Opcode::_16Bit(B16::MOV_REGS_T2)
         }else{
            Opcode::_16Bit(B16::LSL_Imm5)
         }
      },
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

impl From<Word> for Opcode{
   fn from(word: Word)->Self{
      let op1 = (word[1] & 0x1C) >> 3;
      let op = (word[3] & 0xC0)>>7;

      match (op1,op){
         (0,_) =>  panic!("cannot decode a 16bit instruction as 32bit"),
         (1|3,_)=> Opcode::_32Bit(B32::UNDEFINED),
         (2,0) => Opcode::_32Bit(B32::UNDEFINED), 
         (2,1) => Opcode::_32Bit(branch_and_misc(word)),
         _ => unreachable!(),
      }
   }
}

#[inline]
fn branch_and_misc(bytes: Word)->B32{
   let first_u16 = from_arm_bytes_16b([bytes[0],bytes[1]]);
   let op1 = (first_u16 & 0x07F0) >> 4;
   let op2 = (bytes[3] & 0x70) >> 4;
   for i in bytes{
      dbg_print!("{:x}",i);
   }
   dbg_ln!();
   dbg_ln!("{:x}",bytes[3]);
   dbg_ln!("{:x},{:x}",op2,op1);
   match (op2,op1){
      (0,0x38|0x39) => B32::MSR,
      (0,0x3B) => control(bytes),
      (0,0x3E | 0x3F) => B32::MRS,
      (2,0x7F) => B32::UNDEFINED,
      (0x5|0x7,_) => B32::BR_AND_LNK,
      _ => unreachable!(),
   }
}

#[inline]
fn control(bytes: Word)->B32{
   dbg_ln!("control prefix: {:x}",bytes[2] & 0xF0);
   match bytes[2] & 0xF0{
      0x40 => B32::DSB,
      0x50 => B32::DMB,
      0x60 => B32::ISB,
      _ => unreachable!(),
   }
}

pub fn decode_opcodes(bytes: &[u8])->Vec<Opcode>{
   let mut i: usize = 0;
   let mut opcodes = Vec::new();
   while i < bytes.len(){
      let hw: [u8;2] = bytes[i..i+2].try_into().expect("should be 2byte aligned"); 
      let thumb_instruction = Opcode::from(hw);
      if thumb_instruction == Opcode::_16Bit(B16::UNDEFINED){
         if i + 4 > bytes.len(){
            break;
         }
         let word: [u8;4] = bytes[i..i+4].try_into().expect("should be 4byte aligned");
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

