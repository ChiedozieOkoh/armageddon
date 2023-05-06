use crate::asm::decode::{Opcode,B16};
use super::decode_operands::{get_operands, pretty_print, get_operands_32b};

pub fn print_assembly(bytes: &[u8]){
   let src_code = disassemble(
      bytes, 
      |code,encoded_16b|{
         let maybe_args = get_operands(&code, encoded_16b);
         match maybe_args{
            Some(args) => {
               if code == Opcode::_16Bit(B16::CPS){
                  format!("{}",pretty_print(&args))
               }else{
                  format!("{} {}",code,pretty_print(&args))
               }
            },
            None => format!("{}",code)
         }
      },
      |code,encoded_32b|{
         let maybe_args = get_operands_32b(&code, encoded_32b);
         match maybe_args{
            Some(args) => {
               format!("{} {}",code,pretty_print(&args))
            },
            None => format!("{}",code)
         }
      }
   );
   for line in src_code{
      println!("{}",line);
   }
}

fn disassemble<T,F: Fn(Opcode,&[u8;2])->T, G: Fn(Opcode,&[u8;4])->T>(bytes: &[u8],operation: F,operation_32b: G)->Vec<T>{
   let mut i: usize = 0;
   let mut result: Vec<T> = Vec::new();
   while i < bytes.len(){
      let hw: &[u8;2] = &bytes[i..i+2].try_into().expect("should be 2byte aligned"); 
      let thumb_instruction = Opcode::from(hw);
      //TODO use the ISA specified way of checking whether a instruction is 32b or 16b
      // UNDEFINED is a valid instruction and should not be used this way.
      if thumb_instruction == Opcode::_16Bit(B16::UNDEFINED){
         if i + 4 > bytes.len(){
         println!("dont know wtf this is and its not 32b");
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
