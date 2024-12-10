use crate::binutils::BitField;
use crate::system::instructions::{add_with_carry, asr, ConditionFlags};

#[test] 
pub fn adc_should_detect_unsigned_overflow_with_carry_bit(){

   let (sum,carry,overflow) = add_with_carry::<32>(
      BitField::<32>(u32::MAX.into()), BitField::<32>(0_u32.into()), 1);

   assert_eq!(sum,0);
   assert_eq!(carry,true);
   assert_eq!(overflow,false);
}

#[test]
pub fn adc_should_detect_signed_overflow(){
   //max a three bit signed int can represent is 3 thus 3 + 1 should trigger an overflow
   let a: BitField<32> = 0x7FFFFFFF_u32.into();
   let b: BitField<32> = 0x0_u32.into();

   let (sum,carry,overflow) = add_with_carry(a, b, 1);
   assert_eq!(sum, a.0 + 1);
   assert_eq!(carry,false);
   assert_eq!(overflow, true);
}

#[test]
pub fn adc_within_bound_is_normal(){
   let a: BitField<3> = 0x2_u32.into();
   let b: BitField<3> = 0x1_u32.into();

   let (sum,carry,overflow) = add_with_carry::<3>(a, b, 0);

   //assert_eq!(a.0 + b.0,sum);
   assert_eq!(sum,3);
   assert_eq!(carry,false);
   assert_eq!(overflow,false);
}

#[test]
pub fn  asr_test(){
   let (sum,flags) = asr(0b11000, 4, true);

   assert_eq!(sum,1);
   assert_eq!(flags.negative,false);
   assert_eq!(flags.zero,false);
   assert_eq!(flags.carry,true);
   assert_eq!(flags.overflow,true);
}

