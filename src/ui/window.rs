use iced::widget::pane_grid::Pane;
use std::collections::HashMap;
use crate::dbg_ln;

use iced::widget::scrollable::Id;

use super::MemoryView;
pub struct Window{
   focus: Option<Pane>,
   items: HashMap<Pane,Id>
}

impl Window{
   pub fn create()->Self{
      Self{ focus: None, items: HashMap::new() }
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
