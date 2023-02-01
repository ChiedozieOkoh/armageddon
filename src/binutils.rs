#[inline]
pub fn get_bit(bit: u32,word: u32)-> u32{
   println!("mask={:04b} & value={:04b}",(1<<bit),word);
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
pub fn u16_to_arm_hword(v: u16)->u16{
   u16::to_le(v)
}

#[inline]
pub fn u32_to_arm_bytes(v: u32)-> [u8;4]{
   u32::to_le_bytes(v)
}

#[inline]
pub fn from_arm_bytes(word: [u8;4])-> u32{
   u32::from_le_bytes(word)
}

#[inline]
pub const fn from_arm_bytes_16b(hw: [u8;2])->u16{
   u16::from_le_bytes(hw)
}

#[derive(Copy,Clone)]
pub struct  BitField<const L: u32> (pub u32);
impl <const L: u32> From<u32> for  BitField<L>{
   fn from(value: u32) -> Self {
      Self(value)
   }
}

impl <const L: u32> From<u8> for  BitField<L>{
   fn from(value: u8) -> Self {
      Self(value as u32)
   }
}

impl <const L: u32> std::fmt::Display for  BitField<L>{
   fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
      write!(f,"#{}",self.0)
   }
}

impl <const L: u32> std::fmt::Debug for  BitField<L>{
   fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
      write!(f,"<w:{}>#{}",L,self.0)
   }
}

#[inline]
pub fn get_bitfield<const LEN: u32>(v: u32, start: u32)->BitField<LEN>{
   debug_assert!(LEN != 0);
   let last_bit = 1 << (LEN -1);
   let mask = (last_bit | last_bit - 1) << start;
   println!("v={:#02b},m={:#02b}",v,mask);
   let res = v & mask;
   (res >> start).into()
}

pub fn sign_extend<const A: u32>(x: BitField<A>)->i32{
   debug_assert!(A != 0);
   let mask = 1 << (A-1);
   if x.0 & mask > 0{
      println!("adding extra bits");
      let extra_bits = u32::MAX - umax::<A>();
      println!("msk={:#x},val={:#x}\n{:#x} | {:#x} = {:#x}",extra_bits,x.0,extra_bits,x.0,extra_bits|x.0);
      let extended = extra_bits | x.0;
      extended as i32
   }else{
      x.0 as i32
   }
}
#[inline]
fn get_bitfield_u32(v: u32, start: u32, len: u32)->u32{
   debug_assert!(len != 0);
   let last_bit = 1 << (len -1);
   let mask = (last_bit | last_bit - 1) << start;
   println!("v={:#02b},m={:#02b}",v,mask);
   let res = v & mask;
   res >> start
}

pub fn umax<const L: u32>()->u32{
   let last_bit =  1 << (L - 1);
   last_bit | last_bit - 1
}

pub fn smax<const L: u32>()->i32{
   let last_bit =  1 << (L - 1);
   last_bit - 1
}

pub fn smin<const L: u32>()->i32{
   let min: u32 = 1 << (L-1);
   signed_bitfield::<L>(min.into()) as i32
}

pub fn clear_extra<const L: u32>(a: BitField<L>)->BitField<L>{
   let mask = (1 << (L-1)) | (1 << (L-1)) -1;
   (a.0 & mask).into()
}

pub fn signed_bitfield<const L: u32>(a: BitField<L>)->i32{
   println!("making signed bitfield");
   if get_bit(L - 1,a.0)  == 0{
      println!("ret={}",a.0);
      a.0 as i32
   }else{
      println!("ret=({} - {})",a.0 as i32,1<<L);
      a.0 as i32  - (1<<L)
   }
}

pub type BitList = u16;
pub fn get_set_bits(bytes: BitList)->Vec<u8>{
   let mut bits = Vec::new();
   for shift in 0..16{
      let set = (1 << shift) & bytes > 0;
      //println!("{:#x} & {:#x} = {:?}",(1<<shift),bytes,set);
      if set{
         bits.push(shift as u8);
      }
   }
   bits
}

