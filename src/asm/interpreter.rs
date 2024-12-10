use crate::{asm::decode::{Opcode,B16}, elf::decoder::{LiteralPools, SymbolDefinition, SymbolType}, binutils::{from_arm_bytes_16b, from_arm_bytes}, conditional_branches};
use crate::dbg_ln;
use super::{decode_operands::{get_operands, pretty_print, get_operands_32b, Operands, u32_to_hex, serialise_operand}, decode::{instruction_size, InstructionSize, B32, serialise_opcode}};

pub const INDENT: &str = "   ";

pub struct SymbolTable<'a>{
   symbols: &'a Vec<SymbolDefinition>,
   cursor: usize
}

pub fn is_segment_mapping_symbol(symbol: &str)->bool{
   symbol.eq("$d") || symbol.eq("$t")
}

pub fn peek(symbols: &Vec<SymbolDefinition>, address: usize)->Option<&String>{
   let thumb = address | 1;
   match symbols.binary_search_by(|sym|sym.position.cmp(&thumb)){
     Ok(i) => {
        if matches!(symbols[i]._type,SymbolType::Func){
           return Some(&symbols[i].name);
        }
     },
     Err(_) => {},
   }
   let maybe_sym = symbols.binary_search_by(|sym| sym.position.cmp(&address));
   match maybe_sym{
      Ok(s) => {
         if symbols[s].name.eq("$t") || symbols[s].name.eq("$d"){
            let mut i = s;
            while i < symbols.len() && symbols[i].position == address{
               if symbols[i].name.ne("$t") && symbols[i].name.ne("$d"){
                  return Some(&symbols[i].name);
               }
               i += 1;
            }
            return None;
         }else{
            return Some(&symbols[s].name);
         }
      },
      Err(_) => None,
   }
}

impl<'a> SymbolTable<'a>{
   pub fn create(symbols: &'a Vec<SymbolDefinition>)->Self{
      dbg_ln!("sym_arr: {:?}",symbols);
      Self{symbols, cursor: 0}
   }

   //TODO consider the case when multiple symbols have the same address value
   pub fn lookup(&mut self, address: usize)->Option<&String>{
      return self.progressive_lookup(address,true);
   }

   pub fn lookup_ignore_functions(&mut self, address: usize)->Option<&String>{
      return self.progressive_lookup(address,false);
   }
   

   fn progressive_lookup(&mut self, address: usize, search_for_thumb_funcs: bool)->Option<&String>{
      if self.cursor >= self.symbols.len(){
         return None;
      }

      if search_for_thumb_funcs && address % 2 == 0 {
         let thumb_address = address | 1;
         let mut idx = self.symbols.partition_point(|sym| sym.position <= thumb_address);
         idx -= 1;
         while self.symbols[idx].position == thumb_address && idx != 0{
            if matches!(self.symbols[idx]._type,SymbolType::Func){
               return Some(&self.symbols[idx].name);
            }
            idx -= 1;
         }
      }

      while self.cursor < self.symbols.len(){
         let sym_addr = self.symbols[self.cursor].position;
         if sym_addr > address{
            return None;
         }

         if sym_addr == address{
            let name = &self.symbols[self.cursor].name;
            if name.ne("$d") && name.ne("$t"){
               return Some(name);
            }
         }
         

         self.cursor += 1;
      }
      return None;
   }
}

pub fn serialise_instruction(bfr: &mut String, addr: u32, code:&Opcode, operand: &Option<Operands>){
   bfr.push_str(INDENT);
   u32_to_hex(bfr, addr);
   bfr.push(':');
   bfr.push_str(INDENT);
   match operand{
      Some(args)=>{
         if *code != Opcode::_16Bit(B16::CPS){
            serialise_opcode(bfr, code);
            bfr.push(' ');
         }
         serialise_operand(bfr, &args, addr);
      },
      None=>{
         serialise_opcode(bfr, code);
      }
   }
}
pub fn print_instruction(addr: u32,code: &Opcode, operands: &Option<Operands>)->String{
   let instruction = match operands{
      Some(args) => {
         if *code == Opcode::_16Bit(B16::CPS){
            format!("{}{:#010x}:{}{}",INDENT,addr,INDENT,pretty_print(addr,&args))
         }else{
            format!("{}{:#010x}:{}{} {}",INDENT,addr,INDENT,code,pretty_print(addr,&args))
         }
      },
      None => format!("{}{:#010x}:{}{}",INDENT,addr,INDENT,code)
   };

   instruction
}

fn offset_addr(addr: u32, offset: i32)->u32{
   if offset.is_negative(){
      addr - (offset.wrapping_abs() as u32)
   }else{
      addr + (offset as u32)
   }
}

macro_rules! branch_offset {
   ($operand:path,$variant:expr) => {
      match $variant{
         $operand(v) => v,
         _=> {panic!("could not decode {} operands",stringify!($operand))}
      }
   }
}

fn symbol_aware_disassemble(
   byte_offset :usize,
   code: Opcode,
   operands: Option<Operands>,
   maybe_label: Option<&String>,
   symbols: &Vec<SymbolDefinition>
   )->String{
         let mut line = String::new();
         if let Some(label) = maybe_label{
            line.push_str(&format!("\n{:#010x}: <",byte_offset));
            line.push_str(label);
            line.push_str(">:\n");
         }
         let instruction = match operands{
            Some(args) => {
               match code{
                  Opcode::_16Bit(B16::CPS) =>{
                     format!("{}{:#010x}:{}{}",INDENT,byte_offset,INDENT,pretty_print(byte_offset as u32,&args))
                  },
                  Opcode::_16Bit(B16::B_ALWAYS)=>{
                     let offset: i32 = branch_offset!(Operands::B_ALWAYS,args);
                     if offset == 0{
                        format!("{}{:#010x}:{}BAL .",INDENT,byte_offset,INDENT)
                     }else{
                        let target = offset_addr(byte_offset as u32, offset);
                        let target_label = peek(symbols,target as usize);
                        match target_label{
                           Some(ref_label) => {
                              format!("{}{:#010x}:{}BAL ({})",INDENT,byte_offset,INDENT,ref_label)
                           },
                           None => {
                              format!("{}{:#010x}:{}{}{}    //@{:#010x}",INDENT,byte_offset,INDENT,code,pretty_print(byte_offset as u32,&args),target)
                           }
                        }
                     }
                  },

                  conditional_branches!()=>{
                     let offset: i32 = branch_offset!(Operands::COND_BRANCH,args);
                     if offset == 0{
                           format!("{}{:#010x}:{}{} .",INDENT,byte_offset,INDENT,code)
                     }else{
                        let target = offset_addr(byte_offset as u32, offset);
                        let target_label = peek(symbols,target as usize);
                        match target_label{
                           Some(ref_label) => {
                              format!("{}{:#010x}:{}{} ({})",INDENT,byte_offset,INDENT,code,ref_label)
                           },
                           None => {
                              format!("{}{:#010x}:{}{}{}    //@{:#010x}",INDENT,byte_offset,INDENT,code,pretty_print(byte_offset as u32,&args),target)
                           }
                        }
                     }
                  },
                  
                  Opcode::_32Bit(B32::BR_AND_LNK)=>{
                     let offset: i32 = branch_offset!(Operands::BR_LNK,args);
                     let target = offset_addr(byte_offset as u32, offset);
                     let target_label = peek(symbols,target as usize);
                     match target_label{
                        Some(ref_label) => {
                           format!("{}{:#010x}:{}BL ({})",INDENT,byte_offset,INDENT,ref_label)
                        },
                        None => {
                           format!("{}{:#010x}:{}{}{}    //@{:#010x}",INDENT,byte_offset,INDENT,code,pretty_print(byte_offset as u32,&args),target)
                        },
                     }
                  },

                  _ => {
                     format!("{}{:#010x}:{}{} {}",INDENT,byte_offset,INDENT,code,pretty_print(byte_offset as u32,&args))
                  }
               }
            },
            None => format!("{}{:#010x}:{}{}",INDENT,byte_offset,INDENT,code)
         };
         line.push_str(&instruction);
         line

}

pub struct AsmLine{
   pub address: usize,
   pub token: Token
}

pub enum Token{
   instr(Instruction),
   sym(Symbol),
   data(RawData)
}

pub struct Instruction{
   pub representation: String,
}

pub struct Symbol{
   pub name: String,
}

pub struct RawData{
   pub representation: String,
}

//TODO refactor so that lambdas don't needlessly allocate new strings
pub fn disasm_text(bytes: &[u8], section_offset: usize, symbols: &Vec<SymbolDefinition>)->Vec<String>{
   let src_code = disassemble(
      bytes,
      section_offset,
      symbols,
      |byte_offset,code,encoded_16b,label|{
         let maybe_args = get_operands(&code, encoded_16b);
         symbol_aware_disassemble(byte_offset, code, maybe_args, label,symbols)
      },
      |byte_offset,code,encoded_32b,label|{
         let maybe_args = get_operands_32b(&code, encoded_32b);
         symbol_aware_disassemble(byte_offset, code, maybe_args, label,symbols)
      },
      |byte_offset,pool,sym_table|{
         let symbol = sym_table.lookup(byte_offset);
         let mut line = String::new(); 
         if symbol.is_some(){
            line.push_str(&format!("\n{:#010x}: <",byte_offset));
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
                     let inner_symbol = sym_table.lookup(byte_offset + i);
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

pub fn print_assembly(bytes: &[u8],section_offset: usize, symbols: &Vec<SymbolDefinition>){
   let src_code = disassemble(
      bytes,
      section_offset,
      symbols,
      |byte_offset,code,encoded_16b,label|{
         let maybe_args = get_operands(&code, encoded_16b);
         symbol_aware_disassemble(byte_offset, code, maybe_args, label,symbols)
      },
      |byte_offset,code,encoded_32b,label|{
         let maybe_args = get_operands_32b(&code, encoded_32b);
         symbol_aware_disassemble(byte_offset, code, maybe_args, label,symbols)
      },
      |byte_offset,pool,sym_table|{
         let symbol = sym_table.lookup(byte_offset);
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
                     let inner_symbol = sym_table.lookup(byte_offset + i);
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
> (bytes: &[u8],section_offset: usize, symbols: &Vec<SymbolDefinition>, operation_16b: F,operation_32b: G, mut pool_handler: P)->Vec<T>{
   let mut i: usize = 0;
   let mut result: Vec<T> = Vec::new();
   let pools = LiteralPools::create_from_list(symbols);
   let mut sym_table = SymbolTable::create(symbols);
   //println!("pools {:?}",pools);
   //println!("pool @ 0x10000264 ({})_base10 {:?}",0x10000264,pools.get_pool_at(0x10000264));
   //println!("pool @ 0x10000266 ({})_base10 {:?}",0x10000266,pools.get_pool_at(0x10000266));
   //println!("pool @ 0x10000268 ({})_base10 {:?}",0x10000268,pools.get_pool_at(0x10000268));

   println!("symbols: {:?}",sym_table.symbols);
   while i < bytes.len(){
      let abs_position = i + section_offset;
      match pools.get_pool_at(abs_position){
         Some(pool) => {
            let relative_start = pool.start - section_offset; 
            let pl_bin = match pool.end {
                Some(end) => { 
                   let relative_end = end - section_offset;
                   let last = std::cmp::min(relative_end, bytes.len());
                   &bytes[relative_start .. last] 
                },
                None => {&bytes[relative_start ..]},
            };
            let out = pool_handler(abs_position,pl_bin,&mut sym_table);
            result.push(out);
            i += pl_bin.len();
         },
         None => {
            let hw: [u8;2] = bytes[i..i+2].try_into().expect("should be 2byte aligned"); 
            let maybe_label = sym_table.lookup(abs_position);
            dbg_ln!("decoding address = {:#x} value = {:#x} {:#x}",abs_position,hw[0],hw[1]);
            match instruction_size(hw){
               InstructionSize::B16 => {
                  let thumb_instruction = Opcode::from(hw);
                  result.push(operation_16b(abs_position,thumb_instruction,hw,maybe_label));
                  i += 2;
               },
               InstructionSize::B32 => {
                  let word: [u8;4] = bytes[i..i+4].try_into().expect("should be 4byte aligned");
                  let instruction_32bit = Opcode::from(word);
                  result.push(operation_32b(abs_position,instruction_32bit,word,maybe_label));
                  i += 4;
               }
            }
        },
      }
   }
   result 
}

#[derive(Clone,Debug)]
pub struct TextPosition{
   pub line_number: usize,
   pub line_offset: usize
}

pub fn find_string_position(string: &str, substring: &str)->Vec<TextPosition>{
   if substring.is_empty(){
      return Vec::new();
   }

   let mut occurances = Vec::new();
   for (i,line) in string.lines().enumerate(){
      match line.find(substring){
         Some(p) => {occurances.push(TextPosition { line_number: i, line_offset: p });},
         None => {}
      }
   }
   occurances
}

pub fn find_string(string: &str, substring: &str)->Vec<usize>{
   let mut h = 0; 
   let mut n = 0; 
   let mut occurances = Vec::new();
   let lps = build_lps_list(substring);
   while h < string.len(){
      if string.chars().nth(h) == substring.chars().nth(n){
         h += 1;
         n += 1;
      }else{
         if n == 0{
            h += 1;
         }else{
            n = lps[n - 1];
         }
      }
      if n == substring.len(){
         occurances.push(h - substring.len());
         n = 0;
      }
   }

   return occurances;
}

pub fn build_lps_list(needle: &str)->Vec<usize>{
   if needle.len() == 0{
      return vec![0];
   }

   let mut lps = vec![0;needle.len()];
   let mut prev_lps = 0;
   let mut lps_i = 1; 
   while lps_i < needle.len(){
      if needle.chars().nth(lps_i) == needle.chars().nth(prev_lps){
         lps[lps_i] = prev_lps + 1;
         prev_lps += 1;
         lps_i += 1;
      }else{
         if prev_lps == 0{
            lps[lps_i] = 0; 
            lps_i += 1;
         }else{
            prev_lps = lps[prev_lps - 1];
         }
      }
   }
   lps
}


pub fn find_bin(haystack: &[u8], needle: &[u8])->Vec<usize>{
   let mut h = 0; 
   let mut n = 0; 
   let mut occurances = Vec::new();
   let lps = build_bin_lps_list(needle);
   while h < haystack.len(){
      if haystack[h] == needle[n]{
         h += 1;
         n += 1;
      }else{
         if n == 0{
            h += 1;
         }else{
            n = lps[n - 1];
         }
      }
      if n == needle.len(){
         occurances.push(h - needle.len());
         n = 0;
      }
   }

   return occurances;

}

pub fn build_bin_lps_list(needle: &[u8])->Vec<usize>{
   if needle.len() == 0{
      return vec![0];
   }

   let mut lps = vec![0;needle.len()];
   let mut prev_lps = 0;
   let mut lps_i = 1; 
   while lps_i < needle.len(){
      if needle[lps_i] == needle[prev_lps]{
         lps[lps_i] = prev_lps + 1;
         prev_lps += 1;
         lps_i += 1;
      }else{
         if prev_lps == 0{
            lps[lps_i] = 0; 
            lps_i += 1;
         }else{
            prev_lps = lps[prev_lps - 1];
         }
      }
   }
   lps
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

