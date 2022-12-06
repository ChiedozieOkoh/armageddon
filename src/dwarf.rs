pub type Uleb128 = [u8];

pub fn from_uleb128(bytes: &Uleb128)-> u64{
   let mut res = 0u64;
   let mut shift = 0;
   const size: usize = std::mem::size_of::<u64>();
   let mut last_byte: usize = 0;
   for i in 0 .. bytes.len(){
      res |= ((bytes[i] & 0x7f) as u64) << shift;
      if bytes[i] >> 7 == 0u8 {
         break;
      }
      shift += 7;
      last_byte += 1;
   }
   if shift < size && (bytes[last_byte] >> 7 == 1){
      res | (!0 << shift)
   }else{
      res
   }

}
