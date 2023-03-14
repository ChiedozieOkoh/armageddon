use crate::binutils::{get_bit,set_bit,clear_bit,u32_to_arm_bytes,from_arm_bytes};
use crate::asm::Word;
pub type Apsr = Word;//Application Program Status Register

#[derive(Debug,PartialEq)]
pub enum SpecialRegister{
   APSR,
   IAPSR,
   EAPSR,
   XPSR,
   IPSR,
   EPSR,
   IEPSR,
   MSP,
   PSP,
   PRIMASK,
   CONTROL
}

pub struct Registers{
   pub generic: [u32;13], //R0 -> R12
   special: [u32;3] //have restrictions on how they can used
}

impl Registers{
   pub fn create()->Self{
      Self{generic: [0;13],special: [0;3]}
   }
}

pub fn get_negative_bit(apsr: &Apsr)-> u32{
   get_bit(31,from_arm_bytes(*apsr))
}

pub fn set_negative_bit(apsr: &mut Apsr){
   let new_val = set_bit(31, from_arm_bytes(*apsr));
   *apsr = u32_to_arm_bytes(new_val);
}

pub fn clear_negative_bit(apsr: &mut Apsr){
   let new_val = clear_bit(31,from_arm_bytes(*apsr));
   *apsr = u32_to_arm_bytes(new_val);
}

pub fn get_zero_bit(apsr: &Apsr)-> u32{
   get_bit(30,from_arm_bytes(*apsr))
}

pub fn set_zero_bit(apsr: &mut Apsr){
   let new_val = set_bit(30, from_arm_bytes(*apsr));
   *apsr = u32_to_arm_bytes(new_val);
}

pub fn clear_zero_bit(apsr: &mut Apsr){
   let new_val = clear_bit(30,from_arm_bytes(*apsr));
   *apsr = u32_to_arm_bytes(new_val);
}

pub fn get_carry_bit(apsr: &Apsr)-> u32{
   get_bit(29,from_arm_bytes(*apsr))
}

pub fn set_carry_bit(apsr: &mut Apsr){
   let new_val = set_bit(29, from_arm_bytes(*apsr));
   *apsr = u32_to_arm_bytes(new_val);
}

pub fn clear_carry_bit(apsr: &mut Apsr){
   let new_val = clear_bit(29,from_arm_bytes(*apsr));
   *apsr = u32_to_arm_bytes(new_val);
}

pub fn get_overflow_bit(apsr: &Apsr)-> u32{
   get_bit(28,from_arm_bytes(*apsr))
}

pub fn set_overflow_bit(apsr: &mut Apsr){
   let new_val = set_bit(28, from_arm_bytes(*apsr));
   *apsr = u32_to_arm_bytes(new_val);
}

pub fn clear_overflow_bit(apsr: &mut Apsr){
   let new_val = clear_bit(28,from_arm_bytes(*apsr));
   *apsr = u32_to_arm_bytes(new_val);
}

