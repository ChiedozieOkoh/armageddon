use crate::binutils::BitField;
use crate::system::instructions::add_with_carry;

#[test]
pub fn adc_should_detect_overflow(){
   let a: BitField<3> = 0x7_u32.into();// there are 3 bits to represent a and b as numbers
   let b: BitField<3> = 0x1_u32.into();

   let (sum,carry,overflow) = add_with_carry::<3>(a, b, 0);

   assert_eq!(sum,0);
   assert_eq!(carry,true);
   assert_eq!(overflow,true);
}

#[test]
pub fn adc_within_bound_is_normal(){
   let a: BitField<3> = 0x5_u32.into();
   let b: BitField<3> = 0x1_u32.into();

   let (sum,carry,overflow) = add_with_carry::<3>(a, b, 0);

   assert_eq!(a.0 + b.0,sum);
   assert_eq!(carry,false);
   assert_eq!(overflow,false);
}


