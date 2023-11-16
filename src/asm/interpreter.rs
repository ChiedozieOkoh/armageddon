use crate::{asm::decode::{Opcode,B16}, system::System, elf::decoder::LiteralPools, binutils::{from_arm_bytes_16b, from_arm_bytes}};
use super::{decode_operands::{get_operands, pretty_print, get_operands_32b, Operands}, decode::{instruction_size, InstructionSize}};

const INDENT: &str = "   ";

fn find_symbol_ignore_gnu_as_marks(symbols: &Vec<(usize, String)>, trget: usize)->Option<usize>{
   println!("symbols {:?}",symbols);
   println!("searching for symbol at {}",trget);

   let mut i = 0;
   let mut pos = None;
   for (addr,name) in symbols.iter(){
      println!("{} == {} ? :: {}",*addr,trget,name);
      if *addr == trget && name.ne("$t") && name.ne("$d"){
         pos = Some(i);
         break;
      }
      i += 1;
   }

   println!("found {:?}",pos);
   pos
}

fn symbol_aware_disassemble(
   byte_offset :usize,
   code: Opcode,
   operands: Option<Operands>,
   symbols: &Vec<(usize, String)>
   )->String{
         let maybe_label = find_symbol_ignore_gnu_as_marks(symbols, byte_offset);
         let mut line = String::new();
         if let Some(i) = maybe_label{
            line.push_str("\n<");
            line.push_str(&symbols[i].1);
            line.push_str(">:\n");
         }
         let instruction = match operands{
            Some(args) => {
               if code == Opcode::_16Bit(B16::CPS){
                  format!("{}{}",INDENT,pretty_print(&args))
               }else{
                  format!("{}{} {}",INDENT,code,pretty_print(&args))
               }
            },
            None => format!("{}{}",INDENT,code)
         };
         line.push_str(&instruction);
         line

}

pub fn print_assembly(bytes: &[u8],entry_point: usize, symbols: &Vec<(usize, String)>){
   let lit_pools = LiteralPools::create_from_list(symbols);
   println!("pools {:?}",lit_pools);
   println!("pool @ 34 {:?}",lit_pools.get_pool_at(34));
   let src_code = disassemble(
      bytes,
      &lit_pools,
      |byte_offset,code,encoded_16b|{
         let maybe_args = get_operands(&code, encoded_16b);
         symbol_aware_disassemble(byte_offset, code, maybe_args, symbols)
      },
      |byte_offset,code,encoded_32b|{
         let maybe_args = get_operands_32b(&code, encoded_32b);
         symbol_aware_disassemble(byte_offset, code, maybe_args, symbols)
      },
      |byte_offset,pool|{
         let symbol = find_symbol_ignore_gnu_as_marks(symbols, byte_offset);
         let mut line = String::new(); 
         if symbol.is_some(){
            line.push_str("\n<");
            line.push_str(&symbols[symbol.unwrap()].1);
            line.push_str(">:\n");
         }
         match pool.len(){
            2 => {
               let short: [u8;2] = [pool[0],pool[1]];
               let hw = from_arm_bytes_16b(short);
               line.push_str(&format!("{}.2byte {:#x}",INDENT,hw));
               line
            },
            4 => {
               let word: [u8;4] = [pool[0],pool[1],pool[2],pool[3]];
               let wrd = from_arm_bytes(word);
               line.push_str(&format!("{}.4byte {:#x}",INDENT,wrd));
               line
            }
            _ => {
               let mut i = 0;
               for b in pool{
                  if i != 0 {
                     let inner_symbol = find_symbol_ignore_gnu_as_marks(symbols, byte_offset + i);
                     if inner_symbol.is_some(){
                        line.push_str("\n<");
                        line.push_str(&symbols[inner_symbol.unwrap()].1);
                        line.push_str(">:");
                     }
                  }

                  if i % 4 == 0{
                     if i != 0{
                        line.push('\n');
                     }
                     line.push_str(INDENT);
                     line.push_str(".byte".into());
                  }
                  line.push_str(&format!(" {:#x} ",b));
                  i +=1;
               }
               line
            }
         }
      }
   );
   for line in src_code{
      println!("{}",line);
   }
}

use crate::system::TRACED_VARIABLES;
pub fn interpret_with_trace(sys: &mut System, code: &[u8],entry_point: usize)->Vec<[u32;TRACED_VARIABLES]>{
   let mut states = Vec::new();
   let instruction: &[u8;2] = code[entry_point..entry_point + 1].try_into().unwrap();
   return states;
}

fn disassemble<
T,
F: Fn(usize,Opcode,[u8;2])->T,
G: Fn(usize,Opcode,[u8;4])->T,
P: Fn(usize,&[u8])->T,
> (bytes: &[u8], pools: &LiteralPools, operation_16b: F,operation_32b: G, pool_handler: P)->Vec<T>{
   let mut i: usize = 0;
   let mut result: Vec<T> = Vec::new();
   while i < bytes.len(){
      match pools.get_pool_at(i){
         Some(pool) => {
            let pl_bin = match pool.end {
                Some(end) => { 
                   let last = std::cmp::min(end, bytes.len());
                   &bytes[pool.start .. last] 
                },
                None => {&bytes[pool.start ..]},
            };
            let out = pool_handler(i,pl_bin);
            result.push(out);
            i += pl_bin.len();
         },
         None => {
            let hw: [u8;2] = bytes[i..i+2].try_into().expect("should be 2byte aligned"); 
            match instruction_size(hw){
               InstructionSize::B16 => {
                  let thumb_instruction = Opcode::from(hw);
                  result.push(operation_16b(i,thumb_instruction,hw));
                  i += 2;
               },
               InstructionSize::B32 => {
                  let word: [u8;4] = bytes[i..i+4].try_into().expect("should be 4byte aligned");
                  let instruction_32bit = Opcode::from(word);
                  result.push(operation_32b(i,instruction_32bit,word));
                  i += 4;
               }
            }
        },
      }
   }
   result 
}
