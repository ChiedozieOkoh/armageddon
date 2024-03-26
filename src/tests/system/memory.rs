use crate::system::{System,load_memory,write_memory, BlockAllocator};

#[test]
pub fn should_err_on_unaligned_read(){
   let system = System::create(100);
   //println!("{}",system.memory.len());

   let ld_fault_32b = load_memory::<4>(&system, 33);
   let ld_fault_16b = load_memory::<2>(&system, 67);
   let ld_valid_32b = load_memory::<4>(&system, 20);
   let ld_valid_16b = load_memory::<2>(&system, 22);
   let ld_valid_8b = load_memory::<1>(&system, 33);


   assert!(ld_fault_32b.is_err());
   assert!(ld_fault_16b.is_err());
   assert!(ld_valid_32b.is_ok());
   assert!(ld_valid_16b.is_ok());
   assert!(ld_valid_8b.is_ok());
}

#[test]
fn should_err_on_unaligned_write(){
   let mut system = System::create(100);

   let w_fault_32b = write_memory::<4>(&mut system, 33, [1;4]);
   let w_fault_16b = write_memory::<2>(&mut system, 67, [2;2]);
   let w_valid_32b = write_memory::<4>(&mut system, 20, [3;4]);
   let w_valid_16b = write_memory::<2>(&mut system, 22, [5;2]);
   let w_valid_8b = write_memory::<1>(&mut system, 33, [7;1]);

   assert!(w_fault_32b.is_err());
   assert!(w_fault_16b.is_err());
   assert!(w_valid_32b.is_ok());
   assert!(w_valid_16b.is_ok());
   assert!(w_valid_8b.is_ok());
}

#[test]
fn allocator_test(){
   use crate::system::PAGE_SIZE;
   let mut allocator = BlockAllocator::create();
   
   assert_eq!(allocator.get::<4>(104),[0,0,0,0]);
   assert_eq!(allocator.get::<2>(3000),[0,0]);

   allocator.put::<4>(22, [31,2,1,8]);
   assert_eq!(allocator.get::<4>(22),[31,2,1,8]);
   assert_eq!(allocator.pages(),1);

   allocator.put::<1>(PAGE_SIZE as u32 - 1, [8]);
   assert_eq!(allocator.get::<1>(PAGE_SIZE as u32 - 1),[8]);
   assert_eq!(allocator.pages(),1);

   allocator.put::<4>(PAGE_SIZE as u32 - 200, [51,51,51,51]);
   assert_eq!(allocator.get::<4>(PAGE_SIZE as u32 - 200),[51,51,51,51]);
   assert_eq!(allocator.pages(),1);
   
   allocator.put::<4>(PAGE_SIZE as u32 - 4, [2,2,2,2]);
   assert_eq!(allocator.get::<4>(PAGE_SIZE as u32 - 4),[2,2,2,2]);
   assert_eq!(allocator.pages(),1);

   allocator.put::<2>(3000,[87,41]);
   assert_eq!(allocator.get::<2>(3000),[87,41]);
   assert_eq!(allocator.pages(),2);

   let big_address =(PAGE_SIZE * 100) as u32 + 232;
   allocator.put::<4>(big_address,[50,42,99,200]);
   assert_eq!(allocator.get::<4>(big_address),[50,42,99,200]);
   assert_eq!(allocator.pages(),3);

   let big_address_b =(PAGE_SIZE * 200) as u32 + 42;
   allocator.put::<4>(big_address_b,[11,22,33,44]);
   assert_eq!(allocator.get::<4>(big_address_b),[11,22,33,44]);
   assert_eq!(allocator.pages(),4);

   //sanity check
   assert_eq!(allocator.get::<4>(22),[31,2,1,8]);
   assert_eq!(allocator.get::<2>(3000),[87,41]);
   assert_eq!(allocator.get::<4>(big_address),[50,42,99,200]);

   //should be able to overwrite memory
   allocator.put::<2>(big_address,[66,3]);
   assert_eq!(allocator.get::<4>(big_address),[66,3,99,200]);
   assert_eq!(allocator.pages(),4);

   allocator.put::<2>(PAGE_SIZE as u32 - 2, [9,8]);
   allocator.put::<2>(PAGE_SIZE as u32,[6,7]);
   assert_eq!(allocator.get_instr_32b(PAGE_SIZE as u32 - 2),[9,8,6,7],"can fetch 32 bit instructions ");

   assert_eq!(allocator.view(PAGE_SIZE as u32 - 2, PAGE_SIZE as u32 + 1),vec![9,8,6,7]);
   assert_eq!(allocator.view(PAGE_SIZE as u32 - 2, PAGE_SIZE as u32 + 1),vec![9,8,6,7]);
}
