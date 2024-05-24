use std::{thread::{JoinHandle, self}, sync::mpsc::{Sender, channel, Receiver,TryRecvError}};

use super::{System, ArmException};
use crate::ui::Debug; 

#[derive(Clone,Debug)]
pub enum HaltType{
   error(ArmException),
   lockup,
   breakpoint,
   usercmd
}

pub struct Simulator;

//TODO consider having the step signal return the current ip address
impl Simulator{
   pub fn step_or_signal_halt(sys: &mut System)->Result<(),Debug>{
      if sys.is_locked_up(){
         return Err(Debug::Halt(HaltType::lockup));
      }
      match sys.step(){
         Ok(offset) => {
            if sys.check_for_exceptions(offset).is_none(){
               Self::halt_if_err(sys.offset_pc(offset))
            }else{
               return Ok(());
            }
         },
         Err(e) => {
            let offset = match e{
                ArmException::HardFault(_) => 0,
                _ => panic!("simulator error: {:?} caused simualator to abandon instruction execution, but that should only occur when a hardfault occurs",e)
            };
            sys.set_exc_pending(e);
            sys.check_for_exceptions(offset);
            return Ok(());
         },
      }
   }

   pub fn step_or_signal_halt_type(sys: &mut System)->Result<(),HaltType>{
      if sys.is_locked_up(){
         return Err(HaltType::lockup);
      }
      match sys.step(){
         Ok(offset) => {
            if sys.check_for_exceptions(offset).is_none(){
               match sys.offset_pc(offset){
                  Ok(_) => return Ok(()),
                  Err(ex) => return Err(HaltType::error(ex)),
               }
            }else{
               return Ok(())
            }
         },
         Err(e) => {
            let offset = match e{
                ArmException::HardFault(_) => 0,
                _ => panic!("simulator error: {:?} caused simualator to abandon instruction execution, but that should only occur when a hardfault occurs",e)
            };
            sys.set_exc_pending(e);
            sys.check_for_exceptions(offset);
            return Ok(());
         },
      }
   }

   fn halt_if_err(cond: Result<(),ArmException>)->Result<(),Debug>{
      match cond{
         Ok(_) => Ok(()),
         Err(e) => Err(Debug::Halt(HaltType::error(e))),
      }
   }
}
