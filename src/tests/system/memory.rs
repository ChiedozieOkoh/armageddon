use crate::system::{System,load_memory,write_memory};

#[test]
pub fn should_err_on_unaligned_read(){
   let system = System::create(100);
   println!("{}",system.memory.len());

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
