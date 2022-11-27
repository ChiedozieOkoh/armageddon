use std::path::Path;
use crate::elf::decoder::get_header;

#[test]
fn should_get_header(){
   let file = Path::new("./assembly_tests/add.o");
   let header = get_header(&file).unwrap();
   println!("header {:?}", header);
}
