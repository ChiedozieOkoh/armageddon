use std::fmt::Debug;

use crate::binutils::get_set_bits;

use super::{
   DestRegister,
   HalfWord,
   Literal,
   Register,
   SrcRegister,
   Word
};

use crate::asm::decode::{Opcode,B16};
use crate::binutils::{from_arm_bytes_16b, get_bitfield, BitList};

#[allow(non_camel_case_types)]
#[derive(Debug)]
pub enum Operands{
   ADD_Imm3(DestRegister,SrcRegister,Literal<3>),
   ADD_Imm8(DestRegister,Literal<8>),
   ADD_REG_SP_IMM8(DestRegister,Literal<8>),
   INCR_SP_BY_IMM7(Literal<7>),
   INCR_REG_BY_SP(Register),
   INCR_SP_BY_REG(Register),
   ADR(DestRegister,Literal<8>),
   ASRS_Imm5(DestRegister,SrcRegister,Literal<5>),
   COND_BRANCH(Literal<8>),
   B_ALWAYS(Literal<11>),
   BREAKPOINT(Literal<8>),
   BR_LNK_EXCHANGE(Register),
   BR_EXCHANGE(Register),
   CMP_Imm8(Register,Literal<8>),
   LDM(Register,BitList),
   LDR_Imm5(DestRegister,SrcRegister,Literal<5>),
   LDR_Imm8(DestRegister,Literal<8>),
   LDR_REG(DestRegister,SrcRegister,Register),
   LS_Imm5(DestRegister,SrcRegister,Literal<5>),
   MOV_Imm8(DestRegister,Literal<8>),
   MOV_REG(DestRegister,SrcRegister),
   RegisterPair(DestRegister,Register),
   RegisterTriplet(DestRegister,SrcRegister,Register),
   PureRegisterPair(Register,Register),
   MVN(DestRegister,SrcRegister),
   RegisterList(BitList),
}

pub fn pretty_print(operands: &Operands)->String{
   match operands{
      Operands::LDM(base_register,register_list) => {
         let registers = get_set_bits(*register_list);
         let mut list_str = String::new();
         if (1 << base_register.0) & register_list > 0 {
            list_str.push_str(&format!("r{},",base_register.0));
         }else{
            list_str.push_str(&format!("r{}!,",base_register.0));
         }
         list_str.push_str(&fmt_register_list(registers));
         list_str
      },
      Operands::LDR_Imm5(dest,base ,imm5) => {
         format!("{},[{},{}]",dest,base,imm5)
      },
      Operands::LDR_Imm8(dest,imm8) => {
         format!("{},[SP,{}]",dest,imm8)
      },
      Operands::LDR_REG(dest,src,offset) => {
         format!("{},[{},{}]",dest,src,offset)
      },
      Operands::RegisterList(list) => {
         let registers = get_set_bits(*list);
         fmt_register_list(registers)
      },
      _ => {
         println!("tt--{:?}",operands);
         let dbg_operands = format!("{:?}",operands);
         remove_everything_outside_brackets(&dbg_operands)
      }
   }
}


fn fmt_register_list(registers: Vec<u8>)->String{
   let list = registers.iter()
      .map(|n| format!("{}",Register::from(*n)))
      .reduce(|acc,i| acc + "," + &i)
      .unwrap();

   let mut fin = String::new();
   fin.push('{');
   fin.push_str(&list);
   fin.push('}');
   fin
}

fn remove_everything_outside_brackets(text: &str)->String{
   let mut in_brackets = false;
   let mut new_str = String::new();
   for ch in text.chars(){
      if ch == '(' || ch == ')'{
         in_brackets = !in_brackets;
         continue;
      }
      if in_brackets{
         new_str.push(ch);
      }
   }
   new_str
}

pub fn get_operands(code: &Opcode, hw: &HalfWord)-> Option<Operands>{
   match code{
      Opcode::_16Bit(opcode) => {
         match opcode{
            B16::ADCS => Some(get_def_reg_pair_as_operands(hw)),
            B16::ADD_Imm3 =>  Some(get_add_imm3_operands(hw)),
            B16::ADD_Imm8 => Some(get_add_imm8_operands(hw)),
            B16::ADDS_REG => Some(get_add_reg_t1_operands(hw)),
            B16::ADDS_REG_T2 => Some(get_add_reg_t2_operands(hw)),
            B16::ADD_REG_SP_IMM8 => Some(get_add_reg_sp_imm8_operands(hw)),
            B16::INCR_SP_BY_IMM7 => Some(get_add_reg_sp_imm7_operands(hw)),
            B16::INCR_REG_BY_SP => Some(get_incr_reg_by_sp_operands(hw)),
            B16::INCR_SP_BY_REG => Some(get_incr_sp_by_reg_operands(hw)),
            B16::ADR => Some(get_adr_operands(hw)),
            B16::ANDS => Some(get_def_reg_pair_as_operands(hw)),
            B16::ASRS_Imm5 => Some(get_asr_imm5_operands(hw)),
            B16::ASRS_REG => Some(get_def_reg_pair_as_operands(hw)),
            B16::BEQ => Some(get_cond_branch_operands(hw)), 
            B16::BNEQ => Some(get_cond_branch_operands(hw)), 
            B16::B_CARRY_IS_SET => Some(get_cond_branch_operands(hw)), 
            B16::B_CARRY_IS_CLEAR => Some(get_cond_branch_operands(hw)), 
            B16::B_IF_NEGATIVE => Some(get_cond_branch_operands(hw)), 
            B16::B_IF_POSITIVE => Some(get_cond_branch_operands(hw)), 
            B16::B_IF_OVERFLOW => Some(get_cond_branch_operands(hw)), 
            B16::B_IF_NO_OVERFLOW => Some(get_cond_branch_operands(hw)), 
            B16::B_UNSIGNED_HIGHER => Some(get_cond_branch_operands(hw)), 
            B16::B_UNSIGNED_LOWER_OR_SAME => Some(get_cond_branch_operands(hw)), 
            B16::B_GTE => Some(get_cond_branch_operands(hw)), 
            B16::B_LTE => Some(get_cond_branch_operands(hw)), 
            B16::B_GT => Some(get_cond_branch_operands(hw)), 
            B16::B_LT => Some(get_cond_branch_operands(hw)), 
            B16::B_ALWAYS => Some(get_uncond_branch_operands(hw)), 
            B16::BIT_CLEAR_REGISTER => Some(get_def_reg_pair_as_operands(hw)),
            B16::BREAKPOINT => Some(get_breakpoint_operands(hw)),
            B16::BR_LNK_EXCHANGE => Some(get_br_lnk_exchange_operands(hw)),
            B16::BR_EXCHANGE => Some(get_br_exchange_operands(hw)),
            B16::CMP_NEG_REG => Some(get_cmp_neg_reg_operands(hw)),
            B16::CMP_Imm8 => Some(get_cmp_imm8_operands(hw)),
            B16::CMP_REG_T1 => Some(get_cmp_reg_t1_operands(hw)),
            B16::CMP_REG_T2 => Some(get_cmp_reg_t2_operands(hw)),
            B16::XOR_REG => Some(get_def_reg_pair_as_operands(hw)),
            B16::LDM => Some(get_ldm_operands(hw)),
            B16::LDR_Imm5 => Some(get_ldr_imm5_operands(hw)),
            B16::LDR_SP_Imm8 => Some(get_ldr_imm8_operands(hw)),
            B16::LDR_PC_Imm8 => Some(get_ldr_imm8_operands(hw)),
            B16::LDR_REGS => Some(get_ldr_reg_operands(hw)),
            B16::LDRB_Imm5 => Some(get_ldr_imm5_operands(hw)),
            B16::LDRB_REGS => Some(get_ldr_reg_operands(hw)),
            B16::LDRH_Imm5 => Some(get_ldr_imm5_operands(hw)),
            B16::LDRH_REGS => Some(get_ldr_reg_operands(hw)),
            B16::LDRSB_REGS => Some(get_ldr_reg_operands(hw)),
            B16::LDRSH_REGS => Some(get_ldr_reg_operands(hw)),
            B16::LSL_Imm5 => Some(get_ls_imm5_operands(hw)),
            B16::LSL_REGS => Some(get_def_reg_pair_as_operands(hw)),
            B16::LSR_Imm5 => Some(get_ls_imm5_operands(hw)),
            B16::LSR_REGS => Some(get_def_reg_pair_as_operands(hw)),
            B16::MOV_Imm8 => Some(get_mov_imm8_operands(hw)),
            B16::MOV_REGS_T1 => Some(get_mov_reg_operands::<4>(hw)),
            B16::MOV_REGS_T2 => Some(get_mov_reg_operands::<3>(hw)),
            B16::MUL => Some(get_def_reg_pair_as_operands(hw)),
            B16::MVN => Some(get_mvn_operands(hw)),
            B16::NOP => None,
            B16::POP => Some(get_pop_operands(hw)),
            B16::PUSH => Some(get_push_operands(hw)),
            B16::ORR => Some(get_def_reg_pair_as_operands(hw)),
            _  =>  panic!("{:?} has not been implemented",code)

         }
      }
      Opcode::_32Bit(_) => panic!("cannot parse 16b operands from 32b instruction {:?}",code)
   }
}

pub fn get_operands_32b(code: &Opcode, bytes: &Word)->Option<Opcode>{
   match code {
      _ => panic!("{:?} has not been implemented",code)
   }
}

fn get_def_reg_pair(hw: &HalfWord)->(DestRegister,Register){
   let (dest,other) = get_def_reg_pair_u8(hw);
   (dest.into(),other.into())
}

fn get_def_reg_pair_as_operands(hw: &HalfWord)->Operands{
   let (dest,other) = get_def_reg_pair(hw);
   Operands::RegisterPair(dest, other)
}

fn get_pure_reg_pair(hw: &HalfWord)->(Register,Register){
   let (dest,other) = get_def_reg_pair_u8(hw);
   (dest.into(),other.into())
}

#[inline]
fn get_def_reg_pair_u8(hw: &HalfWord)->(u8,u8){
   let dest: u8 = hw[0] & 0x07;
   let other: u8 = (hw[0] & 0x38) >> 3;
   (dest,other)
}

fn get_add_imm3_operands(hw: &HalfWord)->Operands{
   let (dest,other) = get_def_reg_pair(hw);
   let native: u16 = from_arm_bytes_16b(*hw);
   let imm3: Literal<3> = get_bitfield::<3>(native as u32, 6);
   Operands::ADD_Imm3(dest,other.0.into(),imm3)
}

fn get_add_imm8_operands(hw: &HalfWord)->Operands{
   let dest: u8 = hw[1] & 0x07;
   let imm8: Literal<8> = (hw[0] as u32).into();
   Operands::ADD_Imm8(dest.into(),imm8)
}

fn get_9b_register_triplet(hw: &HalfWord)->(DestRegister,SrcRegister,Register){
   let dest: DestRegister = (hw[0] & 0x07).into();
   let src: SrcRegister = ((hw[0] & 0x38) >> 3).into();
   let native: u16 = from_arm_bytes_16b(*hw);
   let second_arg: Register = (((native & 0x01C0) >> 6) as u8).into();
   (dest,src,second_arg)
}
fn get_add_reg_t1_operands(hw: &HalfWord)->Operands{
   let (dest,arg_0,arg_1) = get_9b_register_triplet(hw);
   Operands::RegisterTriplet(dest,arg_0,arg_1)
}

fn get_add_reg_t2_operands(hw: &HalfWord)->Operands{
   let dest: DestRegister = (hw[0] & 0x07).into();
   println!("rm=({:#02b} & {:#02b})= {:#02b}",hw[0],0x78,hw[0] & 0x78);
   let r = get_bitfield::<4>(hw[0] as u32,3);
   Operands::RegisterPair(dest,r.0.into())
}

fn get_add_reg_sp_imm8_operands(hw: &HalfWord)->Operands{
   let dest: u8 = hw[1] & 0x07;
   Operands::ADD_REG_SP_IMM8(dest.into(),(hw[0] as u32).into())
}

fn get_add_reg_sp_imm7_operands(hw: &HalfWord)->Operands{
   let v = hw[0] & 0x7F;
   Operands::INCR_SP_BY_IMM7((v as u32).into())
}

fn get_incr_reg_by_sp_operands(hw: &HalfWord)->Operands{
   let dest = hw[0] & 0x07;
   Operands::INCR_REG_BY_SP(dest.into())
}

fn get_incr_sp_by_reg_operands(hw: &HalfWord)->Operands{
   let dest = get_bitfield::<4>(hw[0] as u32,3);
   Operands::INCR_SP_BY_REG(dest.0.into())
}

fn get_adr_operands(hw: &HalfWord)->Operands{
   let dest: u8 = hw[1] & 0x07;
   Operands::ADR(dest.into(),(hw[0] as u32).into())
}

fn get_asr_imm5_operands(hw: &HalfWord)->Operands{
   let (dest,other) = get_def_reg_pair(hw);
   let native: u16 = from_arm_bytes_16b(*hw);
   let literal = get_bitfield::<5>(native as u32,6);
   Operands::ASRS_Imm5(dest,other.0.into(),literal)
}

fn get_cond_branch_operands(hw: &HalfWord)->Operands{
   let label: Literal<8> = hw[0].into();
   Operands::COND_BRANCH(label)
}

fn get_uncond_branch_operands(hw: &HalfWord)->Operands{
   let native: u16 = from_arm_bytes_16b(*hw);
   let label: Literal<11> = ((native & 0x07FF) as u32).into();
   Operands::B_ALWAYS(label)
}

fn get_breakpoint_operands(hw: &HalfWord)->Operands{
   let imm8: Literal<8> = hw[0].into();
   Operands::BREAKPOINT(imm8)
}

fn get_branch_and_lnk_operands(bytes: &Word)->i32{
   let left_hw: [u8;2] = [bytes[0],bytes[1]];
   let native_l: u16 = from_arm_bytes_16b(left_hw);

   let right_hw: [u8;2] = [bytes[2],bytes[3]];
   let native_r: u16 = from_arm_bytes_16b(right_hw);
   let imm10: u32 = (native_l & 0x03FF) as u32;
   let sign_bit: u32 = ((native_l & 0x0400) >> 10)as u32;

   let imm11: u32 = (native_r & 0x07FF) as u32;
   let j1 = ((native_r & 0x2000) >> 13) as u32;
   let j2 = ((native_r & 0x0800) >> 11) as u32;

   let i1: u32 = !(j1 ^ sign_bit);
   let i2: u32 = !(j2 ^ sign_bit);
   let u_total = (imm11 << 1) | (imm10 << 12) | (i2 << 23) | (i1 << 24) | (sign_bit << 25);
   let sign_extended = if sign_bit > 0 {
      0xFD000000_u32 | u_total
   }else{
      u_total
   };

   sign_extended as i32
}

fn get_br_lnk_exchange_operands(hw: &HalfWord)->Operands{
   let register: Register = ((hw[0] & 0x078) >> 3).into();
   Operands::BR_LNK_EXCHANGE(register)
}

fn get_br_exchange_operands(hw: &HalfWord)->Operands{
   let register: Register = ((hw[0] & 0x078) >> 3).into();
   Operands::BR_EXCHANGE(register)
}

fn get_cmp_neg_reg_operands(hw: &HalfWord)->Operands{
   let (dest,other) = get_pure_reg_pair(hw);
   Operands::PureRegisterPair(dest,other)
}

fn get_cmp_imm8_operands(hw: &HalfWord)->Operands{
   let imm8: Literal<8> =  (hw[0]).into();
   let register: Register = (hw[1] & 0x07).into();
   Operands::CMP_Imm8(register,imm8)
}

fn get_cmp_reg_t1_operands(hw: &HalfWord)->Operands{
   let first: Register = (hw[0] & 0x07).into();
   let second: Register = get_bitfield::<3>(hw[0] as u32,3).0.into();
   Operands::PureRegisterPair(first, second)
}

fn get_cmp_reg_t2_operands(hw: &HalfWord)->Operands{
   let first: Register = (hw[0] & 0x07).into();
   let second: Register = get_bitfield::<4>(hw[0] as u32,3).0.into();
   Operands::PureRegisterPair(first, second)
}

fn get_ldm_operands(hw: &HalfWord)->Operands{
   let list = hw[0] as u16; 
   let reg: Register = (hw[1] & 0x07).into();
   Operands::LDM(reg, list)
}

fn get_dest_src_and_imm5(hw: &HalfWord)->(DestRegister,SrcRegister,Literal<5>){
   let dest: DestRegister = (hw[0] & 0x07).into();
   let native = from_arm_bytes_16b(*hw);
   let base: SrcRegister = get_bitfield::<3>(native as u32,3).0.into();
   let imm5: Literal<5> = get_bitfield::<5>(native as u32,6);
   (dest,base,imm5)
}

fn get_8b_literal_and_dest(hw: &HalfWord)->(Literal<8>,DestRegister){
   let dest: DestRegister = (hw[1] & 0x07).into();
   let imm8: Literal<8> = hw[0].into();
   (imm8,dest)
}

fn get_ldr_imm5_operands(hw: &HalfWord)->Operands{
   let (dest,base,imm5) = get_dest_src_and_imm5(hw);
   Operands::LDR_Imm5(dest,base,imm5)
}

fn get_ldr_imm8_operands(hw: &HalfWord)->Operands{
   let (imm8,dest) = get_8b_literal_and_dest(hw);
   Operands::LDR_Imm8(dest, imm8)
}

fn get_ldr_reg_operands(hw: &HalfWord)->Operands{
   let (dest,base_reg,offset_reg) = get_9b_register_triplet(hw);
   Operands::LDR_REG(dest,base_reg,offset_reg)
}

fn get_ls_imm5_operands(hw: &HalfWord)->Operands{
   let (dest,src,offset) = get_dest_src_and_imm5(hw);
   Operands::LS_Imm5(dest,src,offset)
}

fn get_mov_imm8_operands(hw: &HalfWord)->Operands{
   let (literal,dest) = get_8b_literal_and_dest(hw);
   Operands::MOV_Imm8(dest,literal)
}

fn get_mov_reg_operands<const L: u32>(hw: &HalfWord)->Operands{
   let dest: DestRegister = (hw[0] & 0x07).into();
   let src: SrcRegister = get_bitfield::<L>(hw[0] as u32,3).0.into();
   Operands::MOV_REG(dest,src)
}

fn get_mvn_operands(hw: &HalfWord)->Operands{
   let (dest,src) = get_def_reg_pair(hw);
   Operands::MVN(dest, src.0.into())
}

fn get_pop_operands(hw: &HalfWord)->Operands{
   let pc_bit = (hw[1] & 0x01) as u16;
   let list = hw[0] as u16 | (pc_bit << 15);
   Operands::RegisterList(list)
}

fn get_push_operands(hw: &HalfWord)->Operands{
   let lr_bit = (hw[0] & 0x01) as u16;
   let list = hw[0] as u16 | (lr_bit << 14);
   Operands::RegisterList(list)
}
