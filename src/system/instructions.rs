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
   signed_bitfield, from_arm_bytes, clear_bit_64b
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
   dbg_ln!("sum == {}, zflag = {}", sum, sum == 0);
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

pub fn adc_flags(a: u32, b: u32, carry: bool)->(u32,ConditionFlags){
   let (sum,carry_out,overflow) = add_with_carry::<32>(a.into(), b.into(), carry as u32);
   let flags = ConditionFlags{
      negative: sum & 0x80000000 > 0,
      carry: carry_out,
      zero: sum == 0,
      overflow
   };
   return (sum,flags);
}

pub fn add_with_carry<const L: u32>(a: BitField<L>, b: BitField<L>, carry: u32)-> (u32, bool, bool){
   let u_sum: u64 = (a.0 as u64) + (b.0 as u64) + (carry as u64);
   dbg_ln!("Usum= {} + {} + {} = {}({3:x})",(a.0),(b.0),(carry),u_sum);
   dbg_ln!("64sum= {} + {} + {} = {}",a.0 as u64,b.0 as u64, carry as u64,u_sum);
   let signed_sum: Wrapping<i32> = 
      Wrapping(signed_bitfield::<L>(a)) 
      + Wrapping(signed_bitfield::<L>(b)) 
      + Wrapping(carry as i32);

   let result = (u_sum & 0xFFFFFFFF) as u32;
   dbg_ln!("Ssum = {} + {} + {} = {}",signed_bitfield::<L>(a),signed_bitfield::<L>(b),carry,signed_sum);
   let carry_out = result as u64 != u_sum;
   dbg_ln!("C = {:x} != {:x} = {}",u_sum & 0xFFFFFFFF,u_sum,carry_out);
   let overflow = signed_bitfield::<L>(BitField::<L>(result as u32)) != signed_sum.0;
   dbg_ln!("R= {}",result & 0xFFFFFFFF);
   return ((result & 0xFFFFFFFF) as u32,carry_out,overflow);
}

pub fn asr(val: u32, ammount: u32, overflow: bool)->(u32,ConditionFlags){
   let signed = val & 0x80000000 > 0;
   let signed_bits = u32::MAX << (32 - ammount);
   let result = if signed{
      (val >> ammount) | signed_bits
   }else{
      val >> ammount
   };

   let carry = (val & (1 << (ammount - 1))) > 0;
   let flags = ConditionFlags{
      negative: signed,
      zero: result == 0,
      carry,
      overflow,
   };
   return (result, flags);
}

pub fn shift_right(a: u32, shift: u32, overflow: bool)->(u32,ConditionFlags){
   assert!(shift > 0);
   let last_discarded_bit = ((1 << (shift - 1)) & a) > 0;
   let result = a >> shift;
   let is_negative = (result & 0x80000000) > 0;
   let flags = ConditionFlags{
      carry: last_discarded_bit,
      negative: is_negative,
      zero: result == 0,
      overflow
   };
   return (result,flags);
}

pub fn shift_left(a: u32, shift: u32, overflow: bool)->(u32,ConditionFlags){
   assert!(shift > 0);
   let last_discarded_bit = ((1 << (32 - shift)) & a) > 0;
   let result = a << shift;
   let negative = (result & 0x80000000) > 0;
   let flags = ConditionFlags{
      carry: last_discarded_bit,
      negative,
      zero: result == 0,
      overflow
   };
   return (result,flags);
}

pub fn ror(a: u32, rotate: u32, overflow: bool)->(u32,ConditionFlags){
   let r_amt = rotate % 32;
   let left_shift = 32 - r_amt;
   let result = (a << left_shift) | (a >> r_amt);
   let flags = ConditionFlags{
      carry: (result & 0x80000000) > 0,
      negative: (result & 0x80000000) > 0 ,
      zero: result == 0,
      overflow
   };
   return (result,flags);
}
pub fn xor(a: u32, b: u32, overflow: bool)->(u32,ConditionFlags){
   let r = a^b;
   let flags = ConditionFlags{
      carry: false,
      negative: (r & 0x80000000) > 0,
      zero: r == 0,
      overflow
   };

   return (r,flags);
}
