pub mod instructions;
pub mod memory;
pub mod simulation;

use std::fs;
use std::process::Command;


fn gdb_script(start_point_label: &String, lines_of_asm: u32)->String{
   let script = fs::read_to_string("dump_proc_state").unwrap();
   let breakpoint = format!("break {}", start_point_label);
   let lines = format!("set $asm_fn_line = {}",lines_of_asm);

   return script.replace("break test_start", &breakpoint)
      .replace("set $asm_fn_line = LINE", &lines);
}

fn run_script_on_remote_cpu(script: &str, elf: &str)-> String{
   let sh = Command::new("bash")
      .arg("-C")
      .arg("run_hardware_test.sh")
      .arg(script)
      .arg(elf)
      .output()
      .expect("could not run script");
   let msg = std::str::from_utf8(&sh.stdout[..]).unwrap();

   return msg.to_string();
}

