use crate::system::registers::{
   Apsr,
   get_carry_bit,
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
   signed_bitfield
};

use crate::dbg_ln;

fn add_immediate(apsr: &mut Apsr, a: u32, b: u32)->u32{
    let (sum, carry, overflow) = add_with_carry::<31>(a.into(),b.into(),0);
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

fn adc(apsr: &mut Apsr, a: u32, b: u32)->u32{
   let (sum, carry, overflow) = add_with_carry::<31>(a.into(), b.into(), get_carry_bit(apsr));
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

fn adc_narrow(){}

fn left_shift_left_with_carry(a: u32,shift: u32, carry: u32)->(u32,u32){
   let extended = a << shift;
   let result = clear_bit(31, extended);
   let carry = get_bit(31, extended);
   (result,carry)
}
pub fn add_with_carry<const L: u32>(a: BitField<L>, b: BitField<L>, carry: u32)-> (u32, bool, bool){
   let sum: u32 = a.0 + b.0 + carry;
   dbg_ln!("sum= {}",sum);
   let signed_sum: i32 = signed_bitfield::<L>(a) + signed_bitfield::<L>(b) + carry as i32;
   let result = clear_bit(L, sum);
   dbg_ln!("Ssum = {} + {} + {}",signed_bitfield::<L>(a),signed_bitfield::<L>(b),carry);
   dbg_ln!("signed sum={}",signed_sum);
   let carry_out = result != sum;
   dbg_ln!("res({}) != Usum({}) = {}",result,sum,carry_out);
   let overflow = signed_bitfield::<L>(sum.into()) != signed_sum;
   dbg_ln!("Sres({}) != Ssum({}) = {}",signed_bitfield::<L>(sum.into()),signed_sum,overflow);
   (result,carry_out,overflow)
}

