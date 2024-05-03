use iced::widget::pane_grid::Pane;
use std::collections::HashMap;
use crate::dbg_ln;

use iced::widget::scrollable::Id;
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
