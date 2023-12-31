use std::{thread::{JoinHandle, self}, sync::mpsc::{Sender, channel, Receiver,TryRecvError}};

use super::{System, ArmException};
use crate::ui::Debug; 

#[derive(Clone,Debug)]
pub enum HaltType{
   error(ArmException),
   breakpoint
}

pub struct Simulator{
   sys_loop_handle: JoinHandle<()>,
   cmd_sender: Sender<Debug>,
   status_receiver: Receiver<Debug>,
}

//TODO figure away to send a slice of system memory as a signal
//TODO consider having the step signal return the current ip address
impl Simulator{
   fn step_or_signal_halt(sys: &mut System)->Result<(),Debug>{
      match sys.step(){
         Ok(offset) => Self::halt_if_err(sys.offset_pc(offset)),
         Err(e) => Err(Debug::Halt(HaltType::error(e))),
      }
   }

   fn halt_if_err(cond: Result<(),ArmException>)->Result<(),Debug>{
      match cond{
         Ok(_) => Ok(()),
         Err(e) => Err(Debug::Halt(HaltType::error(e))),
      }
   }

   pub fn start_async_fde_cycle(mut sys: System)->Self{
      let (cmd_sender,cmd_receiver): (Sender<Debug>,Receiver<Debug>) = channel();
      let (status_sndr,status_rcvr): (Sender<Debug>,Receiver<Debug>) = channel();
      let handle = thread::spawn(move ||{
         let mut should_disconnect = false;
         let mut should_continue = false;
         loop{
            match cmd_receiver.try_recv(){
                Ok(dbg_sig) => match dbg_sig{
                   Debug::Disconnect => should_disconnect = true,
                   Debug::Step => {
                      match Self::step_or_signal_halt(&mut sys){
                         Ok(_) => {},
                         Err(e) => {status_sndr.send(e);},
                      }
                   },
                   Debug::CreateBreakpoint(address) => {
                      sys.add_breakpoint(address);
                   },
                   Debug::DeleteBreakpoint(address) => {
                      sys.remove_breakpoint(address);
                   },
                   Debug::Continue => {
                      should_continue = true;
                   },
                   Debug::Halt(_) =>{
                      should_continue = false;
                   }
                },
                Err(e) => match e {
                   TryRecvError::Disconnected => should_disconnect = true,
                   _ => {}
                },
            }
            if should_disconnect {break};
            if should_continue{
               if sys.on_breakpoint(){
                  should_continue = false;
                  status_sndr.send(Debug::Halt(HaltType::breakpoint));
               }else{
                  match Self::step_or_signal_halt(&mut sys){
                     Ok(_) => {},
                     Err(e) => {status_sndr.send(e);},
                  }
               }
            }
         }
      });

      Self{
         sys_loop_handle: handle,
         cmd_sender,
         status_receiver: status_rcvr
      }
   }
}
