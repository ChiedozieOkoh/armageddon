use std::{thread::{JoinHandle, self}, sync::mpsc::{Sender, channel, Receiver,TryRecvError}};

use super::{System, ArmException};
use crate::ui::Debug; 

#[derive(Clone,Debug)]
pub enum HaltType{
   error(ArmException),
   breakpoint,
   usercmd
}

pub struct Simulator;

//TODO figure away to send a slice of system memory as a signal
//TODO consider having the step signal return the current ip address
impl Simulator{
   pub fn step_or_signal_halt(sys: &mut System)->Result<(),Debug>{
      match sys.step(){
         Ok(offset) => {
            if sys.check_for_exceptions().is_none(){
               Self::halt_if_err(sys.offset_pc(offset))
            }else{
               return Ok(());
            }
         },
         Err(e) => {
            sys.set_exc_pending(e);
            sys.check_for_exceptions();
            return Ok(());
         },
      }
   }

   pub fn register_exceptions(sys: &mut System, err: ArmException){
      sys.set_exc_pending(err);
      sys.check_for_exceptions();
   }

   pub fn step_or_signal_halt_type(sys: &mut System)->Result<(),HaltType>{
      match sys.step(){
         Ok(offset) => {
            if sys.check_for_exceptions().is_none(){
               match sys.offset_pc(offset){
                  Ok(_) => Ok(()),
                  Err(ex) => Err(HaltType::error(ex)),
               }
            }else{
               return Ok(())
            }
         },
         Err(e) => {
            sys.set_exc_pending(e);
            sys.check_for_exceptions();
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
