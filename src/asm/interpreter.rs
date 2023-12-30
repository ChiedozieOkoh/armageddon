use crate::{asm::decode::{Opcode,B16}, elf::decoder::LiteralPools, binutils::{from_arm_bytes_16b, from_arm_bytes}};
use super::{decode_operands::{get_operands, pretty_print, get_operands_32b, Operands}, decode::{instruction_size, InstructionSize}};

const INDENT: &str = "   ";

pub struct SymbolTable<'a>{
   symbols: &'a Vec<(usize,String)>,
   cursor: usize
}

impl<'a> SymbolTable<'a>{
   pub fn create(symbols: &'a Vec<(usize, String)>)->Self{
      Self{symbols, cursor: 0}
   }

   pub fn progressive_lookup(&mut self, address: usize)->Option<&String>{
      if self.cursor >= self.symbols.len(){
         return None;
      }
      while self.cursor < self.symbols.len(){
         let sym_addr = self.symbols[self.cursor].0;
         if sym_addr > address{
            return None;
         }
         if sym_addr == address{
            let name = &self.symbols[self.cursor].1;
            if name.ne("$d") && name.ne("$t"){
               return Some(name);
            }
         }

         self.cursor += 1;
      }
      return None;
   }
}

fn symbol_aware_disassemble(
   byte_offset :usize,
   code: Opcode,
   operands: Option<Operands>,
   maybe_label: Option<&String>
   )->String{
         let mut line = String::new();
         if let Some(label) = maybe_label{
            line.push_str(&format!("\n{:#010x}: <",byte_offset));
            line.push_str(label);
            line.push_str(">:\n");
         }
         let instruction = match operands{
            Some(args) => {
               if code == Opcode::_16Bit(B16::CPS){
                  format!("{}{:#010x}:{}{}",INDENT,byte_offset,INDENT,pretty_print(&args))
               }else{
                  format!("{}{:#010x}:{}{} {}",INDENT,byte_offset,INDENT,code,pretty_print(&args))
               }
            },
            None => format!("{}{}",INDENT,code)
         };
         line.push_str(&instruction);
         line

}

pub fn disasm_text(bytes: &[u8], entry_point: usize, symbols: &Vec<(usize,String)>)->Vec<String>{
   let src_code = disassemble(
      bytes,
      symbols,
      |byte_offset,code,encoded_16b,label|{
         let maybe_args = get_operands(&code, encoded_16b);
         symbol_aware_disassemble(byte_offset, code, maybe_args, label)
      },
      |byte_offset,code,encoded_32b,label|{
         let maybe_args = get_operands_32b(&code, encoded_32b);
         symbol_aware_disassemble(byte_offset, code, maybe_args, label)
      },
      |byte_offset,pool,sym_table|{
         let symbol = sym_table.progressive_lookup(byte_offset);
         let mut line = String::new(); 
         if symbol.is_some(){
            line.push_str(&format!("\n{:#010x} <",byte_offset));
            line.push_str(symbol.unwrap());
            line.push_str(">:\n");
         }
         match pool.len(){
            2 => {
               let short: [u8;2] = [pool[0],pool[1]];
               let hw = from_arm_bytes_16b(short);
               line.push_str(&format!("{}{:#010x}: .2byte {:#x}",INDENT,byte_offset,hw));
               line
            },
            4 => {
               let word: [u8;4] = [pool[0],pool[1],pool[2],pool[3]];
               let wrd = from_arm_bytes(word);
               line.push_str(&format!("{}{:#010x}: .4byte {:#x}",INDENT,byte_offset,wrd));
               line
            }
            _ => {
               let mut i = 0;
               for b in pool{
                  if i != 0 {
                     let inner_symbol = sym_table.progressive_lookup(byte_offset + i);
                     if inner_symbol.is_some(){
                        line.push_str(&format!("\n{:#010x}: <",byte_offset + i));
                        line.push_str(inner_symbol.unwrap());
                        line.push_str(">:");
                     }
                  }

                  if i % 4 == 0{
                     if i != 0{
                        line.push('\n');
                     }
                     line.push_str(INDENT);
                     line.push_str(&format!("{:#010x}: ",byte_offset + i));
                     line.push_str(".byte".into());
                  }
                  line.push_str(&format!(" {:#04x} ",b));
                  i +=1;
               }
               line
            }
         }
      }
   );
   src_code
}

pub fn print_assembly(bytes: &[u8],entry_point: usize, symbols: &Vec<(usize, String)>){
   let src_code = disassemble(
      bytes,
      symbols,
      |byte_offset,code,encoded_16b,label|{
         let maybe_args = get_operands(&code, encoded_16b);
         symbol_aware_disassemble(byte_offset, code, maybe_args, label)
      },
      |byte_offset,code,encoded_32b,label|{
         let maybe_args = get_operands_32b(&code, encoded_32b);
         symbol_aware_disassemble(byte_offset, code, maybe_args, label)
      },
      |byte_offset,pool,sym_table|{
         let symbol = sym_table.progressive_lookup(byte_offset);
         let mut line = String::new(); 
         if symbol.is_some(){
            line.push_str(&format!("\n{:#010x} <",byte_offset));
            line.push_str(symbol.unwrap());
            line.push_str(">:\n");
         }
         match pool.len(){
            2 => {
               let short: [u8;2] = [pool[0],pool[1]];
               let hw = from_arm_bytes_16b(short);
               line.push_str(&format!("{}{:#010x}:{}.2byte {:#x}",INDENT,byte_offset,INDENT,hw));
               line
            },
            4 => {
               let word: [u8;4] = [pool[0],pool[1],pool[2],pool[3]];
               let wrd = from_arm_bytes(word);
               line.push_str(&format!("{}{:#010x}:{}.4byte {:#x}",INDENT,byte_offset,INDENT,wrd));
               line
            }
            _ => {
               let mut i = 0;
               for b in pool{
                  if i != 0 {
                     let inner_symbol = sym_table.progressive_lookup(byte_offset + i);
                     if inner_symbol.is_some(){
                        line.push_str(&format!("\n{:#010x}: <",byte_offset + i));
                        line.push_str(inner_symbol.unwrap());
                        line.push_str(">:");
                     }
                  }

                  if i % 4 == 0{
                     if i != 0{
                        line.push('\n');
                     }
                     line.push_str(INDENT);
                     line.push_str(&format!("{:#010x}:{}",byte_offset + i,INDENT));
                     line.push_str(".byte".into());
                  }
                  line.push_str(&format!(" {:#04x} ",b));
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

fn disassemble<
T,
F: Fn(usize,Opcode,[u8;2],Option<&String>)->T,
G: Fn(usize,Opcode,[u8;4],Option<&String>)->T,
P: FnMut(usize,&[u8],&mut SymbolTable)->T,
> (bytes: &[u8], symbols: &Vec<(usize,String)>, operation_16b: F,operation_32b: G, mut pool_handler: P)->Vec<T>{
   let mut i: usize = 0;
   let mut result: Vec<T> = Vec::new();
   let pools = LiteralPools::create_from_list(symbols);
   let mut sym_table = SymbolTable::create(symbols);
   println!("pools {:?}",pools);
   println!("pool @ 34 {:?}",pools.get_pool_at(34));

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
            let out = pool_handler(i,pl_bin,&mut sym_table);
            result.push(out);
            i += pl_bin.len();
         },
         None => {
            let hw: [u8;2] = bytes[i..i+2].try_into().expect("should be 2byte aligned"); 
            let maybe_label = sym_table.progressive_lookup(i);
            match instruction_size(hw){
               InstructionSize::B16 => {
                  let thumb_instruction = Opcode::from(hw);
                  result.push(operation_16b(i,thumb_instruction,hw,maybe_label));
                  i += 2;
               },
               InstructionSize::B32 => {
                  let word: [u8;4] = bytes[i..i+4].try_into().expect("should be 4byte aligned");
                  let instruction_32bit = Opcode::from(word);
                  result.push(operation_32b(i,instruction_32bit,word,maybe_label));
                  i += 4;
               }
            }
        },
      }
   }
   result 
}

/*fn find_symbol_ignore_gnu_as_marks(symbols: &Vec<(usize, String)>, trget: usize)->Option<usize>{
   println!("symbols {:?}",symbols);
   println!("searching for symbol at {}",trget);
   let bound = symbols.partition_point(|(addr,_)| *addr == trget);
   println!("bound -> {:?}",bound);

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
}*/

