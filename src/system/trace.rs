pub struct Trace{
   log: String,
   writes: u16,
   max_records: u16
}

impl Trace{
   pub fn create(max: u16)->Self{
      Self{
         log: String::new(),
         writes: 0,
         max_records: max
      }
   }

   #[inline]
   pub fn get(&mut self)->&mut String{
      self.writes += 1;
      &mut self.log
   }

   #[inline]
   pub fn push(&mut self, ch: char){
      self.log.push(ch);
   }

   #[inline]
   pub fn push_str(&mut self, line: &str){
      self.writes += 1;
      self.log.push_str(line);
   }

   #[inline]
   pub fn ends_with(&self, line: &str)->bool{
      self.log.ends_with(line)
   }

   #[inline]
   pub fn clone(&self)->String{
      self.log.clone()
   }

   pub fn trim(&mut self){
      while self.writes >= self.max_records{
         match self.log.find('\n'){
            Some(i) => {
               self.log.drain(..=i);
               self.writes -= 1;
            },
            None => break
        }
      }
   }
}


