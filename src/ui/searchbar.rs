use crate::{asm::interpreter::{find_string, find_bin, TextPosition, find_string_position}, to_arm_bytes};

use super::Cast;
use std::collections::BTreeMap;

pub struct SearchBar{
   pub kind: Kind,
   pub target: String,
   pub pending: String,
   occurances: Occurance,
   cursor: usize,
   focus: Option<usize>
}

pub enum Occurance{
   Text(Vec<TextPosition>),
   Bin(Vec<usize>)
}

#[derive(Debug)]
pub enum SearchError{
   InvalidTarget,
   ElementExistsOutsideDisassembly
}

impl SearchBar{
   pub fn create()->Self{
      Self { 
         kind: Kind::Code,
         target: String::new(),
         pending: String::new(),
         occurances: Occurance::Text(Vec::new()), 
         cursor: 0,
         focus: None 
      }
   }
   pub fn help(&self)->String{
      match &self.kind{
         Kind::Code => "(Code search): Search the disassembled text".into(),
         Kind::Ascii => "(Ascii search): Search the binary for an ascii sequence".into(),
         Kind::Num(t) => format!("({} search): Search the binary for a {} sequence",t,t),
      }
   }

   pub fn next_text_occurance(&mut self)->Option<TextPosition>{
      match &self.occurances{
         Occurance::Text(ref positions) => {
            let current = self.current_text_occurance();
            if current.is_some(){
               self.cursor += 1;
            }
            return current;
         },
         Occurance::Bin(_) => None,
      }
   }

   pub fn is_focused(&self, position: &TextPosition)->bool{
      match &self.focus{
         Some(f) => {
            match &self.occurances{
               Occurance::Text(ref v) => {
                  return v[*f].line_number == position.line_number && v[*f].line_offset == position.line_offset; 
               },
               Occurance::Bin(_) => todo!(),
            }
         },
         None => {false}
      }
   }

   pub fn get_focused_search_result(&self)->Option<TextPosition>{
      match &self.focus{
         Some(f) => {
            match &self.occurances{
               Occurance::Text(ref v) => {
                  Some(v[*f].clone())
               },
               Occurance::Bin(_) => todo!(),
            }
         },
         None => {None}
      }
   }

   pub fn text_occurances(&self)->BTreeMap<usize,Vec<TextPosition>>{
      match &self.occurances{
        Occurance::Text(ref v) => {
           let mut occ = BTreeMap::new();
           for pos in v{
              occ.entry(pos.line_number)
                 .and_modify(|pl: &mut Vec<TextPosition>| pl.push(pos.clone()))
                 .or_insert(vec![pos.clone()]);

           }
           return occ;
        },
        Occurance::Bin(_) => BTreeMap::new() ,
      }
   }

   #[inline]
   pub fn current_text_occurance(&self)->Option<TextPosition>{
      match &self.occurances{
         Occurance::Text(ref positions) => {
            if self.cursor == positions.len(){ return None; }
            if positions.is_empty(){ return None; } 

            let next = positions[self.cursor].clone();
            return Some(next);
         },
         _ => None,
      }
   }

   pub fn focus_next(&mut self)->Option<usize>{
      let len = match &self.occurances{
        Occurance::Text(v) => v.len(),
        Occurance::Bin(v) => v.len(),
      };

      if len == 0 {
         return None;
      }
      match &mut self.focus{
         Some(i) => {
            let c = *i;
            *i = (c + 1) % len;
            return Some(c);
         },
         None => {
            if len > 0{
               self.focus = Some(0);
               return Some(0);
            }else{
               return None
            }
         }
      }
   }

   pub fn find(&mut self,disasm: &str)->Result<(),SearchError>{
      self.focus = None;
      match &self.kind{
         Kind::Code => {
            let places = find_string_position(disasm, &self.target);
            self.occurances = Occurance::Text(places);
            Ok(())
         },
         Kind::Ascii =>{
            todo!()
         },
         Kind::Num(n)=>{
            todo!()
         }
      }
   }
}

pub enum Kind{
   Code,
   Ascii,
   Num(Cast)
}

pub static KIND_OPTIONS: &[Kind] = &[
   Kind::Code,
   //Kind::Ascii,
   //Kind::Num(Cast::UWORD),
   //Kind::Num(Cast::IWORD),
   //Kind::Num(Cast::UHALF),
   //Kind::Num(Cast::IHALF),
   //Kind::Num(Cast::UBYTE),
   //Kind::Num(Cast::IBYTE)
];
