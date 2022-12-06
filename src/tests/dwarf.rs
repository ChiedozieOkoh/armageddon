use crate::dwarf::from_uleb128;


#[test]
pub fn should_decode_spec_examples(){
   assert_eq!(2,from_uleb128(&[2]));
   assert_eq!(127,from_uleb128(&[127]));
   assert_eq!(128,from_uleb128(&[0x80,0x1]));
   assert_eq!(129,from_uleb128(&[0x80 + 0x01,0x1]));
   assert_eq!(130,from_uleb128(&[0x80 + 0x02,0x1]));
   assert_eq!(624485,from_uleb128(&[0xe5, 0x8e, 0x26]));
}
