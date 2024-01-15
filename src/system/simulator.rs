use std::{thread::{JoinHandle, self}, sync::mpsc::{Sender, channel, Receiver,TryRecvError}};

use super::{System, ArmException};
use crate::ui::Debug; 

#[derive(Clone,Debug)]
pub enum HaltType{
   error(ArmException),
   breakpoint
}

pub struct Simulator;

//TODO figure away to send a slice of system memory as a signal
//TODO consider having the step signal return the current ip address
impl Simulator{
   pub fn step_or_signal_halt(sys: &mut System)->Result<(),Debug>{
      match sys.step(){
         Ok(offset) => Self::halt_if_err(sys.offset_pc(offset)),
         Err(e) => Err(Debug::Halt(HaltType::error(e))),
      }
   }

   pub fn step_or_signal_halt_type(sys: &mut System)->Result<(),HaltType>{
      match sys.step(){
         Ok(offset) => match sys.offset_pc(offset){
            Ok(_) => Ok(()),
            Err(ex) => Err(HaltType::error(ex)),
         },
         Err(e) => Err(HaltType::error(e)),
      }
   }

   fn halt_if_err(cond: Result<(),ArmException>)->Result<(),Debug>{
      match cond{
         Ok(_) => Ok(()),
         Err(e) => Err(Debug::Halt(HaltType::error(e))),
      }
   }
}
