pub mod instructions;
pub mod memory;
pub mod simulation;

use std::fs;
use std::process::Command;

use crate::dbg_ln;

const PROC_VARIABLES: usize = 8;
pub const R0: usize = 0;
pub const R1: usize = 1;
pub const R2: usize = 2;
pub const R3: usize = 3;
pub const SP: usize = 4;
pub const PC: usize = 5;
pub const LR: usize = 6;
pub const XPSR: usize = 7;

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

fn parse_gdb_output(output: &str)->Vec<u32>{
   let mut add = false;
   let mut register_display = String::new();
   for line in output.lines(){
      if line.contains("<<STARTING_PROC_LOG>>"){
         add = true;
      }

      if add {
         register_display.push_str(line);
         register_display.push('\n');
      }
      
      if line.contains("<<FINISHED_PROC_LOG>>"){
         break;
      }

   }

   
   let mut states = Vec::new();
   for state in register_display.split("<<-->>"){
      println!("Qstate: [{}]",&state);
      if state.contains("<<FINISHED_PROC_LOG>>"){
         continue;
      }

      let r0 = get_value_of_register(state, "$r0 = ").unwrap();
      //println!("r0 -> {:?}", r0);
      let r1 = get_value_of_register(state, "$r1 = ").unwrap();
      //println!("r1 -> {:?}", r1);
      let r2 = get_value_of_register(state, "$r2 = ").unwrap();
      //println!("r2 -> {:?}", r2);
      let r3 = get_value_of_register(state, "$r3 = ").unwrap();
      //println!("r3 -> {:?}", r3);
      let sp = get_reg_val_from_hex(state, "$sp = ").unwrap();
      //println!("sp -> {:?}", sp);
      let pc = get_reg_val_from_hex(state, "$pc = ").unwrap();
      //println!("pc -> {:?}", pc);
      let lr = get_value_of_register(state, "$lr = ").unwrap();
      //println!("lr -> {:?}", lr);
      let xpsr = get_value_of_register(state, "$xPSR = ").unwrap();
      //println!("xPSR -> {:?}", xpsr);

      states.push(r0);
      states.push(r1);
      states.push(r2);
      states.push(r3);
      states.push(sp);
      states.push(pc);
      states.push(lr);
      states.push(xpsr);
   }

   return states;
}

fn get_reg_val_from_hex(state: &str, prefix: &str)->Option<u32>{
   match state.find(prefix){
      Some(i) => {
         match state[i..].find('x'){
            Some(j) => {
               let mut digit = String::new();
               for ch in state[i + j + 1 ..].chars(){
                  if !ch.is_alphanumeric(){
                     break;
                  }else{
                     digit.push(ch);
                  }
               }
               let num = u32::from_str_radix(&digit, 16).unwrap();
               dbg_ln!("{} == {}_base10",&digit, num);
               return Some(num);
            }
            None => None,
         }
      },
      None => None
   }
}

fn get_value_of_register(state: &str, prefix: &str)->Option<u32>{
   let r0_position = state.find(prefix);
   match r0_position {
      Some(i) => {
         return Some(convert_assignment_to_int(state, i));
      },
      None => return None,
   }
}

fn convert_assignment_to_int(string: &str,position: usize )->u32{
   let mut add = false;
   let mut str_value = String::new();
   for ch in string.chars().skip(position){
      if add  && !ch.is_whitespace(){
         str_value.push(ch);
      }
      if ch == '='{
         add = true;
      }
      if ch == '\n'{
         break;
      }
   }
   return str_value.parse::<u32>().unwrap();
}

fn print_states(states: Vec<u32>){
   print_state_column(&states, R0, "r0");
   print_state_column(&states, R1, "r1");
   print_state_column(&states, R2, "r2");
   print_state_column(&states, R3, "r3");
   print_state_column(&states, SP, "sp");
   print_state_column(&states, PC, "pc");
   print_state_column(&states, LR, "lr");
   print_state_column(&states, XPSR, "xpsr");
}

fn print_state_column(states: &Vec<u32>, column: usize, name: &str){
   print!("{}:", name);
   for q in states.iter().skip(column).step_by(PROC_VARIABLES){
      print!(" {} ->", q);
   }
   println!("END");
}
