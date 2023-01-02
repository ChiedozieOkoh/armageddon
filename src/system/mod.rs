use self::registers::{Registers, Apsr};
pub mod registers;
pub mod instructions;

pub struct System{
   pub main_memory: Registers,
   pub status: Apsr,
}
impl System{
   pub fn create()->Self{
      let main_memory = Registers::create();
      System{main_memory,status: [0;4]}
   }
}
pub enum Fault{
   Alignment,
   UnknownInstruction, //not part of the ARMv6 spec but makes sense for an Interpreter
}

pub enum SysErr{
   Fault(Fault),
}

