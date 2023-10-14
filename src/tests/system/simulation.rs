
use crate::tests::system::run_script_on_remote_cpu;

use super::gdb_script;

#[test] #[ignore] 
pub fn hardware_linear_search(){
   let label = String::from("linear_search");
   let script = gdb_script(&label, 12);
   println!("script generated\n{}",&script);
   std::fs::write("dump_proc_state_linear_search", &script).unwrap();
   let output = run_script_on_remote_cpu(
      "dump_proc_state_linear_search".into(), 
      "elf_samples/linear_search.elf".into()
   );

   println!("{}",&output);
   std::fs::remove_file("dump_proc_state_linear_search").unwrap();
   panic!("want to see logs");
}
