use iced::widget::pane_grid::{Pane,State};
use std::collections::HashMap;
use crate::dbg_ln;

use iced::widget::scrollable::Id;

type LineNumber = usize;
use super::{MemoryView,PaneType};
use iced::Point as CursorPoint;

pub struct Window{
   focus: Option<Pane>,
   pub cursor_position: Option<CursorPoint>,
   items: HashMap<Pane,Id>,
   scroll_offset: HashMap<Id,LineNumber>
}

pub fn line_buffer(large_buffer: &String,first_line: usize,last_line_included: usize)->&str{

   let mut line_iter = large_buffer.lines();
   let mut ch_ptr: usize = 0;
   let viewport_lines = last_line_included - first_line; 
   let mut starting_line = first_line;
   while starting_line > 0 {
      let line_len = line_iter.next().unwrap().as_bytes().len();
      if line_len == 0 {
         dbg_ln!("blank line detected");
         ch_ptr += 1;
      }else{
         ch_ptr += line_len;
         while ch_ptr < large_buffer.as_bytes().len(){
            if large_buffer.as_bytes()[ch_ptr] == '\n' as u8 && ((ch_ptr + 1) < large_buffer.as_bytes().len()){
               ch_ptr += 1;
               break;
            }else{
               ch_ptr += 1;
            }
         }
      }
      starting_line -= 1;
   }

   let mut end_ptr: usize = ch_ptr; 
   dbg_ln!("end_ptr begins by pointing to: '{}'",large_buffer.as_bytes()[end_ptr] as char);
   for _ in 0 ..=viewport_lines{
      match line_iter.next(){
         Some(ln_text) => {
            /*
            println!("end_ptr now pointing to: '{}'",large_buffer.as_bytes()[end_ptr] as char);
            println!("line checked '{}' len : {}",ln_text, ln_text.as_bytes().len());
            println!("{} := {} + {}",end_ptr,end_ptr,ln_text.as_bytes().len());
            end_ptr += std::cmp::max(1,ln_text.as_bytes().len());
            */
            
            let line_len = ln_text.as_bytes().len();
            if line_len == 0{
               end_ptr += 1;
            }else{
               if large_buffer.as_bytes()[end_ptr] == '\n' as u8{
                  end_ptr += 1;
               }
               end_ptr += line_len;
            }
         },
         None => break,
      }
   }

   //(ch_ptr,end_ptr)
   &large_buffer[ch_ptr .. end_ptr]
}

impl Window{
   pub fn create()->Self{
      Self{ 
         focus: None,
         items: HashMap::new(),
         scroll_offset: HashMap::new(),
         cursor_position: None
      }
   }
   pub fn add_pane(&mut self, p: Pane)->Id{
      let id = Id::unique();
      let _ = self.items.insert(p, id.clone());
      self.focus = Some(p);
      id
   }

   pub fn id_of(&self, p: &Pane)->Option<&Id>{
      self.items.get(p)
   }

   pub fn remove_pane(&mut self, p: &Pane){
      if self.items.contains_key(p){
         self.scroll_offset.remove(self.items.get(p).unwrap());
      }
      self.items.remove(p);
   }

   pub fn focus_if_present(&mut self, p: &Pane){
      if self.items.contains_key(p){
         self.focus = Some(p.clone());
         dbg_ln!("now focused on {:?}",p);
         dbg_ln!("active window ids: {:?}",self.items);
      }
   }

   pub fn get_focused_pane(&self)->Option<& Id>{
      match self.focus{
         Some(pane) => {
            let maybe_exists = self.items.get(&pane);
            maybe_exists
         },
         None => None,
      }
   }

   pub fn record_line_offset(&mut self, id: Id, line_number: usize){
      self.scroll_offset.entry(id)
         .and_modify(|ln| *ln = line_number)
         .or_insert(line_number);
   }

   pub fn scroll_position(&self,id: &Id)->Option<&usize>{
      self.scroll_offset.get(id)
   }
}

pub struct ExplorerMap{
   mem_view: HashMap<Id,MemoryView>,
   pending_start: HashMap<Id,String>,
   pending_end: HashMap<Id,String>
}

impl ExplorerMap{
   pub fn create()->Self{
      Self{
         mem_view: HashMap::new(),
         pending_start: HashMap::new(),
         pending_end: HashMap::new(),
      }
   }

   pub fn view_of(&self, id: &Id)->Option<&MemoryView>{
      self.mem_view.get(id)
   }

   pub fn mut_view_of(&mut self, id: &Id)->Option<&mut MemoryView>{
      self.mem_view.get_mut(id)
   }

   pub fn set_view(&mut self, id: Id, val: MemoryView){
      self.mem_view.insert(id, val);
   }

   pub fn mut_view_entry(&mut self, id: Id) -> std::collections::hash_map::Entry<Id, MemoryView> {
      self.mem_view.entry(id)
   }

   pub fn get_start(&self, id: &Id)->Option<String>{
      if let Some(s) = self.pending_start.get(id){
         Some(s.clone())
      }else{
         None
      }
   }

   pub fn start_string(&mut self, id: Id,val: String){
      self.pending_start.insert(id,val);
   }

   pub fn end_string(&mut self, id: Id,val: String){
      self.pending_end.insert(id,val);
   }

   pub fn get_end(&self, id: &Id)->Option<String>{
      match self.pending_end.get(id){
         Some(s) => Some(s.clone()),
         None => None,
      }
   }

   pub fn remove(&mut self,id: &Id){
      self.mem_view.remove(id);
      self.pending_start.remove(id);
      self.pending_end.remove(id);
   }
}
