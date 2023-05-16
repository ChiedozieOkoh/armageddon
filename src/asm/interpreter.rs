use std::collections::HashMap;

use crate::asm::decode::{Opcode,B16};
use super::{decode_operands::{get_operands, pretty_print, get_operands_32b}, decode::{instruction_size, InstructionSize}};

//TODO get text section symbol offset, use that to recognise symbols by relative byte offset.
pub fn print_assembly(bytes: &[u8], text_symbol_map: &HashMap<usize, String>){
   let src_code = disassemble(
      bytes,
      |byte_offset,code,encoded_16b|{
         let maybe_args = get_operands(&code, encoded_16b);
         let maybe_label = text_symbol_map.get(&byte_offset);
         let mut line = String::new();
         if let Some(label) = maybe_label{
            line.push_str("<");
            line.push_str(label);
            line.push_str(">:\n");
         }
         let instruction = match maybe_args{
            Some(args) => {
               if code == Opcode::_16Bit(B16::CPS){
                  format!("{}",pretty_print(&args))
               }else{
                  format!("{} {}",code,pretty_print(&args))
               }
            },
            None => format!("{}",code)
         };
         line.push_str(&instruction);
         line
      },
      |byte_offset,code,encoded_32b|{
         let maybe_args = get_operands_32b(&code, encoded_32b);
         let mut line = String::new();
         let maybe_label = text_symbol_map.get(&byte_offset);
         if let Some(label) = maybe_label{
            line.push_str("<");
            line.push_str(label);
            line.push_str(">:\n");
         }
         let instruction = match maybe_args{
            Some(args) => {
               format!("{} {}",code,pretty_print(&args))
            },
            None => format!("{}",code)
         };
         line.push_str(&instruction);
         line
      }
   );
   for line in src_code{
      println!("{}",line);
   }
}

fn disassemble<
T,
F: Fn(usize,Opcode,&[u8;2])->T,
G: Fn(usize,Opcode,&[u8;4])->T
> (bytes: &[u8], operation_16b: F,operation_32b: G)->Vec<T>{
   let mut i: usize = 0;
   let mut result: Vec<T> = Vec::new();
   while i < bytes.len(){
      let hw: &[u8;2] = &bytes[i..i+2].try_into().expect("should be 2byte aligned"); 
      match instruction_size(hw){
         InstructionSize::B16 => {
            let thumb_instruction = Opcode::from(hw);
            result.push(operation_16b(i,thumb_instruction,hw));
            i += 2;
         },
         InstructionSize::B32 => {
            let word: &[u8;4] = &bytes[i..i+4].try_into().expect("should be 4byte aligned");
            let instruction_32bit = Opcode::from(word);
            result.push(operation_32b(i,instruction_32bit,word));
            i += 4;
         }
      }
   }
   result 
}
