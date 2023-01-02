use crate::asm::decode::{Opcode,B16};
use crate::system::{Fault,System,SysErr};

use super::decode_operands::{get_operands, pretty_print, Operands};
//use crate::asm::{DestRegister,SrcRegister,Literal,Register};

pub struct Disassembler;
//type Operation<T> = Fn(Opcode,&[u8;2])->T;

pub fn print_assembly(bytes: &[u8]){
   let src_code = disassemble(
      bytes, 
      |code,encoded_16b|{
         let maybe_args = get_operands(&code, encoded_16b);

         match maybe_args{
            Some(args) => format!("{:?} {}",code,pretty_print(&args)),
            None => format!("{:?}",code)
         }
      },
      |_,_|{
         String::from("<decoding 32b instruction operands is not implemented yet >")
      }
   );
   for line in src_code{
      println!("{}",line);
   }
}

pub fn execute(bytes: &[u8])->Result<(),SysErr>{
   Ok(())
}

fn disassemble<T,F: Fn(Opcode,&[u8;2])->T, G: Fn(Opcode,&[u8;4])->T>(bytes: &[u8],operation: F,operation_32b: G)->Vec<T>{
   let mut i: usize = 0;
   let mut result: Vec<T> = Vec::new();
   while i < bytes.len(){
      let hw: &[u8;2] = &bytes[i..i+2].try_into().expect("should be 2byte aligned"); 
      let thumb_instruction = Opcode::from(hw);
      if thumb_instruction == Opcode::_16Bit(B16::UNDEFINED){
         if i + 4 > bytes.len(){
            break;
         }
         let word: &[u8;4] = &bytes[i..i+4].try_into().expect("should be 4byte aligned");
         let instruction_32bit = Opcode::from(word);
         result.push(operation_32b(instruction_32bit,word));
         i += 4;
      }else{
         result.push(operation(thumb_instruction,hw));
         i += 2;
      }
   }
   result 
}

fn decode_and_exec<F,G>(bytes: &[u8],mut operation_16b: F,mut operation_32b: G)-> Result<(),SysErr>
where
   F: FnMut(&mut System,Opcode,&[u8;2]) -> Result<(),SysErr>,
   G: FnMut(&mut System,Opcode,&[u8;4]) -> Result<(),SysErr>{
   let mut i: usize = 0;
   let mut result = Err(SysErr::Fault(Fault::UnknownInstruction));
   let mut sys = System::create();
   while i < bytes.len(){
      let hw: &[u8;2] = &bytes[i..i+2].try_into().expect("should be 2byte aligned"); 
      let thumb_instruction = Opcode::from(hw);
      if thumb_instruction == Opcode::_16Bit(B16::UNDEFINED){
         if i + 4 > bytes.len(){
            break;
         }
         let word: &[u8;4] = &bytes[i..i+4].try_into().expect("should be 4byte aligned");
         let instruction_32bit = Opcode::from(word);
         result = operation_32b(&mut sys,instruction_32bit,word);
         i += 4;
      }else{
         result = operation_16b(&mut sys,thumb_instruction,hw);
         i += 2;
      }

      if result.is_err(){
         return result;
      }
   }
   result 
}

