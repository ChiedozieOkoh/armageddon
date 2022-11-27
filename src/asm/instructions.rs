use std::num::TryFromIntError;
use super::registers::{
   Apsr,
   get_carry_bit,
   set_carry_bit,
   set_overflow_bit,
   set_zero_bit,
   clear_carry_bit, clear_overflow_bit, clear_zero_bit
};
use super::{clear_bit, get_bit};

fn add_immediate(apsr: &mut Apsr, a: u32, b: u32)->u32{
    let (sum, carry, overflow) = add_with_carry(a,b,0);
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
   let (sum, carry, overflow) = add_with_carry(a, b, get_carry_bit(apsr));
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

fn add_with_carry(a: u32, b: u32, carry: u32)-> (u32, bool, bool){
   let sum = a + b + carry;
   let result = clear_bit(31, sum);
   let carry_out = result != sum;
   let sum_result: Result<i32,TryFromIntError> = sum.try_into();
   let overflow = sum_result.is_err();
   (result,carry_out,overflow)
}
