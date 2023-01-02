pub mod interpreter;
pub mod decode;
pub mod decode_operands;

use std::fmt;
// are little endian
//---
pub type HalfWord = [u8;2];
pub type Word = [u8;4];
//---

#[derive(PartialEq)]
pub struct Register(pub u8);
impl From<u8> for Register{
   fn from(a: u8) -> Self {
      Self(a)
   }
}

impl From<u32> for Register{
   fn from(a: u32) -> Self {
      Self(a as u8)
   }
}

impl fmt::Display for Register{
   fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
      write!(f,"r{}",self.0)
   }
}

impl fmt::Debug for Register{
   fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
      write!(f,"r{}",self.0)
   }
}

#[derive(PartialEq)]
pub struct SrcRegister(pub u8);
impl From<u8> for SrcRegister{
   fn from(a: u8) -> Self {
      Self(a)
   }
}

impl From<u32> for SrcRegister{
   fn from(a: u32) -> Self {
      Self(a as u8)
   }
}

impl fmt::Debug for SrcRegister{
   fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
      write!(f,"Rs{}",self.0)
   }
}

impl fmt::Display for SrcRegister{
   fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
      write!(f,"r{}",self.0)
   }
}

#[derive(PartialEq)]
pub struct DestRegister(pub u8);
impl From<u8> for DestRegister{
   fn from(a: u8) -> Self {
      Self(a)
   }
}

impl From<u32> for DestRegister{
   fn from(a: u32) -> Self {
      Self(a as u8)
   }
}

impl fmt::Debug for DestRegister{
   fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
      write!(f,"Rd{}",self.0)
   }
}

impl fmt::Display for DestRegister{
   fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
      write!(f,"r{}",self.0)
   }
}
use crate::{binutils::BitField, system::SysErr};

pub type Literal<const L: u32> = BitField<L>;

pub trait Intruction{
   type IOperand;
   fn has_opcode(hw: &HalfWord)->bool;
   fn get_operands(hw: &HalfWord)->Self::IOperand;
   fn execute(args: Self::IOperand)->Result<(),SysErr>;
}
