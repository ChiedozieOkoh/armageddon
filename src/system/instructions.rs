use crate::asm::decode::{Opcode,B16};
use crate::system::registers::{
   Apsr,
   set_carry_bit,
   set_overflow_bit,
   set_zero_bit,
   clear_carry_bit, 
   clear_overflow_bit,
   clear_zero_bit
};
use crate::binutils::{
   BitField,
   clear_bit,
   get_bit,
   signed_bitfield, from_arm_bytes
};

use crate::dbg_ln;
use std::num::Wrapping;

pub fn add_immediate(a: u32, b: u32)->(u32, ConditionFlags){
   let (sum, carry, overflow) = add_with_carry::<32>(a.into(),b.into(),0);
   let zero = sum == 0;
   let negative = (sum & 0x80000000) > 0;
   return (sum, ConditionFlags{carry,negative,zero,overflow});
}

fn do_adc(apsr: &mut Apsr, a: u32, b: u32)->u32{
   let (sum, carry, overflow) = add_with_carry::<32>(a.into(), b.into(), carry_flag(*apsr).into());
   if carry{
      set_carry_bit(apsr);
   }else{
      clear_carry_bit(apsr);
   }

   if overflow{
      set_overflow_bit(apsr);
   }else{
      clear_overflow_bit(apsr);
   }

   if sum == 0{
      set_zero_bit(apsr);
   }else{
      clear_zero_bit(apsr);
   }
   sum
}

pub fn negative_flag(apsr: Apsr)->bool{
   return (from_arm_bytes(apsr) & 0x80000000) > 0;
}

pub fn zero_flag(apsr: Apsr)-> bool{
   return (from_arm_bytes(apsr) & 0x40000000) > 0;
}

pub fn carry_flag(apsr: Apsr)-> bool{
   return (from_arm_bytes(apsr) & 0x20000000) > 0;
}

pub fn overflow_flag(apsr: Apsr)->bool{
   return (from_arm_bytes(apsr) & 0x10000000) > 0;
}

pub fn cond_passed(apsr: Apsr, b_cond: &Opcode)->bool{
   match b_cond{
       Opcode::_16Bit(B16::BEQ) => zero_flag(apsr),
       Opcode::_16Bit(B16::BNEQ) => !zero_flag(apsr),
       Opcode::_16Bit(B16::B_CARRY_IS_SET) => carry_flag(apsr),
       Opcode::_16Bit(B16::B_CARRY_IS_CLEAR) => !carry_flag(apsr),
       Opcode::_16Bit(B16::B_IF_NEGATIVE) => negative_flag(apsr),
       Opcode::_16Bit(B16::B_IF_POSITIVE) => !negative_flag(apsr),
       Opcode::_16Bit(B16::B_IF_OVERFLOW) => overflow_flag(apsr),
       Opcode::_16Bit(B16::B_IF_NO_OVERFLOW) => !overflow_flag(apsr),
       Opcode::_16Bit(B16::B_UNSIGNED_HIGHER) => carry_flag(apsr) && !zero_flag(apsr),
       Opcode::_16Bit(B16::B_UNSIGNED_LOWER_OR_SAME) => !carry_flag(apsr) && zero_flag(apsr),
       Opcode::_16Bit(B16::B_GTE) => negative_flag(apsr) == overflow_flag(apsr),
       Opcode::_16Bit(B16::B_LT) => negative_flag(apsr) != overflow_flag(apsr),
       Opcode::_16Bit(B16::B_GT) => !zero_flag(apsr) && (negative_flag(apsr) == overflow_flag(apsr)),
       Opcode::_16Bit(B16::B_LTE) => zero_flag(apsr) && (negative_flag(apsr) != overflow_flag(apsr)),
       Opcode::_16Bit(B16::B_ALWAYS) => true,
      _ => unreachable!()
   }
}

pub fn compare(a: u32, b: u32)->ConditionFlags{
   let (_, flags) = subtract(a,b);
   dbg_ln!("{:?}",flags);
   return flags;
}

#[derive(Debug)]
pub struct ConditionFlags{
   pub carry: bool,
   pub negative: bool,
   pub zero: bool,
   pub overflow: bool
}

pub fn subtract(a: u32, b: u32)->(u32, ConditionFlags){
   let (sum, carry, overflow) = add_with_carry::<32>(a.into(), (!b).into(), 1);
   println!("sum == {}, zflag = {}", sum, sum == 0);
   let negative = (sum & 0x80000000) > 0;
   let flags = ConditionFlags{carry, negative, zero: sum == 0, overflow};
   return (sum,flags);
}

pub fn multiply(a: u32, b: u32)->(u32, bool, bool){
   let sum = a.wrapping_mul(b); // so rust runtime doesnt panic if this wraps
   let negative = (sum & 0x80000000) > 0;
   let zero = sum == 0;
   return (sum,negative,zero);
}

fn left_shift_left_with_carry(a: u32,shift: u32, carry: u32)->(u32,u32){
   let extended = a << shift;
   let result = clear_bit(31, extended);
   let carry = get_bit(31, extended);
   (result,carry)
}

pub fn add_with_carry<const L: u32>(a: BitField<L>, b: BitField<L>, carry: u32)-> (u32, bool, bool){
   let sum: Wrapping<u32> = Wrapping(a.0) + Wrapping(b.0) + Wrapping(carry);
   dbg_ln!("Usum= {} + {} + {} = {}",Wrapping(a.0),Wrapping(b.0),Wrapping(carry),sum);
   let signed_sum: Wrapping<i32> = 
      Wrapping(signed_bitfield::<L>(a)) 
      + Wrapping(signed_bitfield::<L>(b)) 
      + Wrapping(carry as i32);
   let result = if L == 32 { clear_bit(31, sum.0) }else{clear_bit(L, sum.0)};
   dbg_ln!("Ssum = {} + {} + {}",signed_bitfield::<L>(a),signed_bitfield::<L>(b),carry);
   dbg_ln!("signed sum={}",signed_sum);
   let carry_out = result != sum.0;
   dbg_ln!("res({}) != Usum({}) = {}",result,sum,carry_out);
   let overflow = signed_bitfield::<L>(BitField::<L>(result)) != signed_sum.0;
   dbg_ln!("Sres({}) != Ssum({}) = {}",signed_bitfield::<L>(sum.0.into()),signed_sum,overflow);
   (result,carry_out,overflow)
}

