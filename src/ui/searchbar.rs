use crate::{asm::interpreter::{find_string, find_bin, TextPosition, find_string_position}, to_arm_bytes};

use super::Cast;

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

macro_rules! parse_hex_or_base10 {
   ($_type:ty,$string:expr,$is_hex:expr) => {
      if $is_hex{
         match <$_type>::from_str_radix($string,16){
            Ok(v) => Some(v),
            Err(_) => {None},
         }
      }else{
         match <$_type>::from_str_radix($string,10){
            Ok(v) => Some(v),
            Err(_) => {None},
         }
      }
   }
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

   pub fn text_occurances(&self)->Vec<TextPosition>{
      match &self.occurances{
        Occurance::Text(ref v) => v.clone(),
        Occurance::Bin(_) => vec![],
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

   pub fn on_focused_text(&self)->bool{
      match &self.occurances{
         Occurance::Text(_) => match self.focus{
            Some(f) =>{f == self.cursor },
            None => false,
         },
         Occurance::Bin(_) => false,
      }
   }

   pub fn find(&mut self,disasm: &str, _binary: &[u8])->Result<(),SearchError>{
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
            /*let probably_hex = self.target.starts_with("0x");
            match n{
               Cast::UWORD => {
                  /*
                  if self.target.starts_with("0x"){
                     match u8::from_str_radix(&self.target,16){
                        Ok(v) => Some(v),
                        Err(_) => {println!("could not parse {}",&self.target);None},
                     }
                  }else{
                     match u8::from_str_radix(&self.target,10){
                        Ok(v) => Some(v),
                        Err(_) => {println!("could not parse {}",&self.target);None},
                     }
                  }*/
                  let v = parse_hex_or_base10!(u32,&self.target,probably_hex);
                  match v{
                    Some(a) => {
                       let b = to_arm_bytes!(u32,a);
                       let ocurrances = find_bin(_binary,&b[..]);
                       self.ocurrances.clear();
                       for position in occurances{
                       }
                       Ok(())
                    },
                    None => Err(SearchError::InvalidTarget),
                }
               },
               Cast::IWORD => {
                  let v = parse_hex_or_base10!(i32,&self.target,probably_hex);
                  &[0,1]
               },
               Cast::UHALF => {
                  parse_hex_or_base10!(u16,&self.target,probably_hex);
                  &[0,1]
               },
               Cast::IHALF => {
                  parse_hex_or_base10!(i16,&self.target,probably_hex);
                  &[0,1]
               },
               Cast::UBYTE => {
                  parse_hex_or_base10!(u8,&self.target,probably_hex);
                  &[0,1]
               },
               Cast::IBYTE => {
                  parse_hex_or_base10!(i8,&self.target,probably_hex);
                  &[0,1]
               },
            };
            */
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
