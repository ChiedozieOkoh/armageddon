pub mod instructions;
pub mod memory;
pub mod simulation;

use std::fs;
use std::process::Command;

use crate::dbg_ln;

pub const PROC_VARIABLES: usize = 18;
pub const R0: usize = 0;
pub const R1: usize = 1;
pub const R2: usize = 2;
pub const R3: usize = 3;
pub const R4: usize = 4;
pub const R5: usize = 5;
pub const R6: usize = 6;
pub const R7: usize = 7;
pub const R8: usize = 8;
pub const R9: usize = 9;
pub const R10: usize = 10;
pub const R11: usize = 11;
pub const R12: usize = 12;
pub const MSP: usize = 13;
pub const PSP: usize = 14;
pub const LR: usize = 15;
pub const PC: usize = 16;
pub const XPSR: usize = 17;

fn gdb_script(start_point_label: &String)->String{
   let script = fs::read_to_string("dump_proc_state").unwrap();
   let breakpoint = format!("break {}", start_point_label);

   return script.replace("break TEST_START", &breakpoint);
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

      let r4 = get_value_of_register(state, "$r4 = ").unwrap();

      let r5 = get_value_of_register(state, "$r5 = ").unwrap();

      let r6 = get_value_of_register(state, "$r6 = ").unwrap();

      let r7 = get_value_of_register(state, "$r7 = ").unwrap();

      let r8 = get_value_of_register(state, "$r8 = ").unwrap();

      let r9 = get_value_of_register(state, "$r9 = ").unwrap();

      let r10 = get_value_of_register(state, "$r10 = ").unwrap();

      let r11 = get_value_of_register(state, "$r11 = ").unwrap();

      let r12 = get_value_of_register(state, "$r12 = ").unwrap();
      //println!("r3 -> {:?}", r3);
      let msp = get_reg_val_from_hex(state, "$msp = ").unwrap();

      let psp = get_reg_val_from_hex(state, "$psp = ").unwrap();
      //println!("sp -> {:?}", sp);
      let lr = get_value_of_register(state, "$lr = ").unwrap();
      //println!("lr -> {:?}", lr);
      let pc = get_reg_val_from_hex(state, "$pc = ").unwrap();
      //println!("pc -> {:?}", pc);
      let xpsr = get_value_of_register(state, "$xPSR = ").unwrap();
      //println!("xPSR -> {:?}", xpsr);

      states.push(r0);
      states.push(r1);
      states.push(r2);
      states.push(r3);
      states.push(r4);
      states.push(r5);
      states.push(r6);
      states.push(r7);
      states.push(r8);
      states.push(r9);
      states.push(r10);
      states.push(r11);
      states.push(r12);
      states.push(msp);
      states.push(psp);
      states.push(lr);
      states.push(pc);
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
   if str_value.contains('-'){
      return str_value.parse::<i32>().unwrap() as u32;
   }else{
      return str_value.parse::<u32>().unwrap();
   }
}

fn print_states(states: Vec<u32>){
   print_state_column(&states, R0, "r0");
   print_state_column(&states, R1, "r1");
   print_state_column(&states, R2, "r2");
   print_state_column(&states, R3, "r3");
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

#[test]
fn can_parse_results_simple(){
   let results = concat!(
      "unrelated\n",
      "unrelated\n",
      "<<STARTING_PROC_LOG>>\n",
      "7	MOV r0,#4\n",
      "1: $r0 = 536873020\n",
      "2: $r1 = 268436189\n",
      "3: $r2 = 536871480\n",
      "4: $r3 = 536871668\n",
      "5: $r4 = 268436068\n",
      "6: $r5 = 537140993\n",
      "7: $r6 = 402653184\n",
      "8: $r7 = 0\n",
      "9: $r8 = -1\n",
      "10: $r9 = -1\n",
      "11: $r10 = -1\n",
      "12: $r11 = -1\n",
      "13: $r12 = 872415296\n",
      "14: $msp = (void *) 0x20041ff8\n",
      "15: $psp = (void *) 0xfffffffc\n",
      "16: $lr = 268436195\n",
      "17: $pc = (void (*)()) 0x100002e4 <fibonacci>\n",
      "18: $xPSR = 1627389952\n",
      "<<-->>\n",
      "<<FINISHED_PROC_LOG>>\n"
   );

   let states = parse_gdb_output(&results);

   assert_eq!(states, vec![
      536873020,
      268436189,
      536871480,
      536871668,
      268436068,
      537140993,
      402653184,
      0,
      u32::MAX,
      u32::MAX,
      u32::MAX,
      u32::MAX,
      872415296,
      0x20041ff8,
      0xfffffffc,
      268436195,
      0x100002e4,
      1627389952,
   ]);
}

#[test]
fn can_parse_results(){
   let results = concat!(
      "main () at /home/chiedozie/dev/src/auto_test/main.c:11\n",
      "11	   while (1){ }\n",
      "Breakpoint 1 at 0x100002e4: file /home/chiedozie/dev/src/auto_test/fib.s, line 7.\n",
      "Note: automatically using hardware breakpoints for read-only addresses.\n",
      "Loading section .boot2, size 0x100 lma 0x10000000\n",
      "Loading section .text, size 0x3f08 lma 0x10000100\n",
      "Loading section .rodata, size 0xe08 lma 0x10004008\n",
      "Loading section .binary_info, size 0x1c lma 0x10004e10\n",
      "Loading section .data, size 0x234 lma 0x10004e2c\n",
      "Start address 0x100001e8, load size 20576\n",
      "Transfer rate: 16 KB/sec, 3429 bytes/write.\n",
      "<<STARTING_PROC_LOG>>\n",
      "Breakpoint 1, fibonacci () at /home/chiedozie/dev/src/auto_test/fib.s:7\n",
      "7	MOV r0,#4\n",
      "1: $r0 = 536873020\n",
      "2: $r1 = 268436189\n",
      "3: $r2 = 536871480\n",
      "4: $r3 = 536871668\n",
      "5: $r4 = 268436068\n",
      "6: $r5 = 537140993\n",
      "7: $r6 = 402653184\n",
      "8: $r7 = 0\n",
      "9: $r8 = -1\n",
      "10: $r9 = -1\n",
      "11: $r10 = -1\n",
      "12: $r11 = -1\n",
      "13: $r12 = 872415296\n",
      "14: $msp = (void *) 0x20041ff8\n",
      "15: $psp = (void *) 0xfffffffc\n",
      "16: $lr = 268436195\n",
      "17: $pc = (void (*)()) 0x100002e4 <fibonacci>\n",
      "18: $xPSR = 1627389952\n",
      "<<-->>\n",
      "8	MOV r1,#0\n",
      "1: $r0 = 4\n",
      "2: $r1 = 268436189\n",
      "3: $r2 = 536871480\n",
      "4: $r3 = 536871668\n",
      "5: $r4 = 268436068\n",
      "6: $r5 = 537140993\n",
      "7: $r6 = 402653184\n",
      "8: $r7 = 0\n",
      "9: $r8 = -1\n",
      "10: $r9 = -1\n",
      "11: $r10 = -1\n",
      "12: $r11 = -1\n",
      "13: $r12 = 872415296\n",
      "15: $msp = (void *) 0x20041ff8\n",
      "16: $psp = (void *) 0xfffffffc\n", 
      "16: $lr = 268436195\n",
      "17: $pc = (void (*)()) 0x100002e6 <fibonacci+2>\n",
      "18: $xPSR = 553648128\n",
      "<<-->>\n",
      "11	   while (1){ }\n",
      "1: $r0 = 0\n",
      "2: $r1 = 3\n",
      "3: $r2 = 5\n",
      "4: $r3 = 8\n",
      "5: $r4 = 268436068\n",
      "6: $r5 = 537140993\n",
      "7: $r6 = 402653184\n",
      "8: $r7 = 0\n",
      "9: $r8 = -1\n",
      "10: $r9 = -1\n",
      "11: $r10 = -1\n",
      "12: $r11 = -1\n",
      "13: $r12 = 872415296\n",
      "14: $msp = (void *) 0x20041ff8\n",
      "15: $psp = (void *) 0xfffffffc\n",
      "16: $lr = 268436209\n",
      "17: $pc = (void (*)()) 0x100002e2 <main+6>\n",
      "18: $xPSR = 1627389952\n",
      "<<-->>\n",
      "<<FINISHED_PROC_LOG>>\n",
   );

   let states = parse_gdb_output(&results);

   assert_eq!(states,vec![
      536873020,
      268436189,
      536871480,
      536871668,
      268436068,
      537140993,
      402653184,
      0,
      u32::MAX,
      u32::MAX,
      u32::MAX,
      u32::MAX,
      872415296,
      0x20041ff8,
      0xfffffffc,
      268436195,
      0x100002e4,
      1627389952,

      4,
      268436189,
      536871480,
      536871668,
      268436068,
      537140993,
      402653184,
      0,
      u32::MAX,
      u32::MAX,
      u32::MAX,
      u32::MAX,
      872415296,
      0x20041ff8,
      0xfffffffc,
      268436195,
      0x100002e6,
      553648128,

      0,
      3,
      5,
      8,
      268436068,
      537140993,
      402653184,
      0,
      u32::MAX,
      u32::MAX,
      u32::MAX,
      u32::MAX,
      872415296,
      0x20041ff8,
      0xfffffffc,
      268436209,
      0x100002e2,
      1627389952,
   ]);
}
