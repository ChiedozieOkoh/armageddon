pub mod interpreter;
pub mod instructions;
pub mod registers;
pub mod decode;

// are little endian
//---
pub type Byte = u8;
pub type HalfWord = [u8;2];
pub type Word = [u8;4];
pub type DoubleWord = [u8;8];
pub type Pointer = Word;
//---
#[inline]
pub fn get_bit(bit: u32,word: u32)-> u32{
   println!("{:04b} & {:04b}",(1<<bit),word);
   ((1 << bit) & word) >> bit
}

#[inline]
pub fn clear_bit(bit: u32,word: u32)-> u32{
   let mask = !(1 << bit);
   println!("{:04b} & {:04b}",word,mask);
   word & mask
}

#[inline]
pub fn set_bit(bit: u32,word: u32)-> u32{
   word | (1 << bit)
}

#[inline]
pub fn from_arm_bytes(word: Word)-> u32{
   u32::from_le_bytes(word)
}

#[inline]
pub fn u32_to_arm_bytes(v: u32)-> [u8;4]{
   u32::to_le_bytes(v)
}
