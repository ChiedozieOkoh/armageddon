use crate::binutils::{get_bitfield,get_set_bits,smax,smin,umax};
#[test]
fn should_get_bitfield(){
   let xmpl_1 = get_bitfield::<4>(0x0F00,8);
   let xmpl_2 = get_bitfield::<4>(0b00101100, 2);
   let xmpl_3 = get_bitfield::<3>(0b011111,0);
   let xmpl_4 = get_bitfield::<3>(0xE0,5);
   println!("returned {:#02b}",xmpl_3.0);
   assert_eq!(xmpl_1.0,0x000F);
   assert_eq!(xmpl_2.0,0b1011);
   assert_eq!(xmpl_3.0,7);
   assert_eq!(xmpl_4.0,7);
}

#[test]
fn bitfields_bounds_are_correct(){
   assert_eq!(3,smax::<3>());
   assert_eq!(-4,smin::<3>());
   assert_eq!(7,umax::<3>());
}

#[test]
fn should_get_the_set_bits(){
   let bits = get_set_bits(0x73);
   assert_eq!(vec![0,1,4,5,6],bits);
}
