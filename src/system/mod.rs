use self::registers::{Registers, Apsr};
pub mod registers;
pub mod instructions;

pub struct System{
   pub registers: Registers,
   pub status: Apsr,
   pub memory: Vec<u8>
}

impl System{
   pub fn create(capacity: usize)->Self{
      let registers = Registers::create();
      System{registers,status: [0;4], memory: vec![0;capacity]}
   }
}

pub fn load_memory<'a, const T: usize>(sys: &'a System, v_addr: u32)->Result<&'a [u8;T],SysErr>{
   if !is_aligned(v_addr, T as u32){
      return Err(SysErr::HardFault);
   }

   let mem: &'a[u8;T] = sys.memory[v_addr as usize .. (v_addr as usize + T)]
      .try_into()
      .expect("should not access out of bounds memory");
   return Ok(mem)
}

pub fn write_memory<const T: usize>(sys: &mut System, v_addr: u32, value: [u8;T])->Result<(), SysErr>{
   if !is_aligned(v_addr, T as u32){
      return Err(SysErr::HardFault);
   }

   sys.memory[v_addr as usize ..(v_addr as usize + T )].copy_from_slice(&value);
   return Ok(());
}

fn is_aligned(v_addr: u32, size: u32)->bool{
   let mask: u32 = size - 1;
   return v_addr & mask == 0;
}

pub enum SysErr{
   HardFault,
}

//
//#[repr(u8)]
//enum AddressAttributes{
//   Normal = 0x1,
//   Device = 0x2,
//   DevSharable = 0x4,
//   DevNonShare = 0x8,
//   ExecuteNever = 0x16,
//   StronglyOrdered = 0x32
//}
//
//fn default_address_map(v_addr: u32)-> u8{
//   match v_addr{
//      0x0 ..= 0x1FFFFFFF => AddressAttributes::Normal as u8,
//      0x20000000 ..= 0x3FFFFFFF => AddressAttributes::Normal as u8,
//      0x40000000 ..= 0x5FFFFFFF => AddressAttributes::Device as u8,
//      0x60000000 ..= 0x7FFFFFFF => AddressAttributes::Normal as u8,
//      0x80000000 ..= 0x9FFFFFFF => AddressAttributes::Normal as u8,
//      0xA0000000 ..= 0xBFFFFFFF => AddressAttributes::DevSharable as u8 | AddressAttributes::ExecuteNever as u8,
//      0xC0000000 ..= 0xDFFFFFFF => AddressAttributes::DevNonShare as u8 | AddressAttributes::ExecuteNever as u8,
//      0xE0000000 ..= 0xE00FFFFF => AddressAttributes::StronglyOrdered as u8,
//      0xE0100000 ..= 0xFFFFFFFF => AddressAttributes::ExecuteNever as u8
//   }
//}
//
