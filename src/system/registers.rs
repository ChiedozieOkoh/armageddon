use crate::binutils::{get_bit,set_bit,clear_bit,u32_to_arm_bytes,from_arm_bytes};
use crate::asm::Word;

use super::Access;
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
pub enum RegAccess{
   READ,
   WRITE
}

impl SpecialRegister{
   pub fn needs_privileged_access(&self,access: RegAccess)->bool{
      match self{
         SpecialRegister::APSR => false,
         SpecialRegister::IAPSR => true,
         SpecialRegister::EAPSR => true,
         SpecialRegister::XPSR => true,
         SpecialRegister::IPSR => true,
         SpecialRegister::EPSR => true,
         SpecialRegister::IEPSR => true,
         SpecialRegister::MSP => true,
         SpecialRegister::PSP => true,
         SpecialRegister::PRIMASK => true,
         SpecialRegister::CONTROL => match access{
            RegAccess::READ => false,
            RegAccess::WRITE => true,
        },
      }
   }

   pub fn mask(&self)->u32{
      let apsr = 0xF0000000;
      let epsr = 0x01000200;
      let ipsr = 0x3F;
      match self{
        SpecialRegister::APSR => apsr,
        SpecialRegister::IAPSR => apsr | ipsr,
        SpecialRegister::EAPSR => epsr | apsr,
        SpecialRegister::XPSR => apsr | ipsr | epsr,
        SpecialRegister::IPSR => ipsr,
        SpecialRegister::EPSR => epsr,
        SpecialRegister::IEPSR => ipsr | epsr,
        SpecialRegister::MSP => u32::MAX,
        SpecialRegister::PSP => u32::MAX,
        SpecialRegister::PRIMASK => 1,
        SpecialRegister::CONTROL => 3,
      }
   }
}

#[macro_export]
macro_rules! xpsr_registers {
    () => {
       SpecialRegister::APSR
          | SpecialRegister::IPSR
          | SpecialRegister::IAPSR
          | SpecialRegister::EAPSR
          | SpecialRegister::EPSR
          | SpecialRegister::IEPSR
          | SpecialRegister::XPSR
    };
}

#[derive(Clone)]
pub struct Registers{
   pub generic: [u32;13], //R0 -> R12
   pub sp_main: u32,
   pub sp_process: u32,
   pub lr: u32,
   pub pc: usize
}

impl Registers{
   pub fn create()->Self{
      Self{
         generic: [0;13],
         sp_main: 0,
         sp_process: 0,
         lr: 0,
         pc: 0
      }
   }
}

#[inline]
pub fn get_overflow_bit(apsr: Apsr)->bool{
   let v: u32 = from_arm_bytes(apsr);
   return (0x10000000 & v) > 0;
}

#[inline]
pub fn get_carry_bit(apsr: Apsr)->bool{
   let v: u32 = from_arm_bytes(apsr);
   return (0x20000000 & v) > 0;
}

pub fn set_negative_bit(apsr: &mut Apsr){
   let new_val = set_bit(31, from_arm_bytes(*apsr));
   *apsr = u32_to_arm_bytes(new_val);
}

pub fn clear_negative_bit(apsr: &mut Apsr){
   let new_val = clear_bit(31,from_arm_bytes(*apsr));
   *apsr = u32_to_arm_bytes(new_val);
}

pub fn set_zero_bit(apsr: &mut Apsr){
   let new_val = set_bit(30, from_arm_bytes(*apsr));
   *apsr = u32_to_arm_bytes(new_val);
}

pub fn clear_zero_bit(apsr: &mut Apsr){
   let new_val = clear_bit(30,from_arm_bytes(*apsr));
   *apsr = u32_to_arm_bytes(new_val);
}

pub fn set_carry_bit(apsr: &mut Apsr){
   let new_val = set_bit(29, from_arm_bytes(*apsr));
   *apsr = u32_to_arm_bytes(new_val);
}

pub fn clear_carry_bit(apsr: &mut Apsr){
   let new_val = clear_bit(29,from_arm_bytes(*apsr));
   *apsr = u32_to_arm_bytes(new_val);
}

pub fn set_overflow_bit(apsr: &mut Apsr){
   let new_val = set_bit(28, from_arm_bytes(*apsr));
   *apsr = u32_to_arm_bytes(new_val);
}

pub fn clear_overflow_bit(apsr: &mut Apsr){
   let new_val = clear_bit(28,from_arm_bytes(*apsr));
   *apsr = u32_to_arm_bytes(new_val);
}

