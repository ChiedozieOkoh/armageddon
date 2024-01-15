use std::{fmt::Display, sync::{Mutex, Arc}, ops::Deref};

use iced::{widget::{pane_grid, PaneGrid, text, column, container, scrollable, row, button, vertical_slider::StyleSheet, pick_list}, Application, Theme, executor, Command, Element, futures::StreamExt};
use iced::widget::text_input;

use crate::{system::{System, ArmException, simulator::{HaltType, Simulator}, registers::Registers}, asm::interpreter::{print_assembly, disasm_text, is_segment_mapping_symbol}, binutils::from_arm_bytes_16b};

use crate::binutils::from_arm_bytes;
const TEXT_SIZE: u16 = 11;

pub struct App{
   _state: pane_grid::State<PaneType>,
   n_panes: usize,
   focus: Option<pane_grid::Pane>,
   entry_point: usize,
   pub disasm: String,
   symbols: Vec<(usize,String)>,
   mem_view: Option<MemoryView>,
   sys_view: SystemView,
   sync_sys: Arc<Mutex<System>>,
   cmd_sender: Option<iced_mpsc::Sender<Event>>,
   breakpoints: Vec<u32>,
   view_error: Option<String>,
   pending_mem_start: String,
   pending_mem_end: String,
   update_view: bool,
   bkpt_input: BkptInput
}

struct SystemView{
   pub registers: Registers,
   pub sp: u32,
   pub xpsr: u32,
   pub raw_ir: u32
}

impl From<&System> for SystemView{
   fn from(sys: &System) -> Self {
      Self{
         registers: sys.registers.clone(),
         sp: sys.get_sp(),
         xpsr: from_arm_bytes(sys.xpsr),
         raw_ir: sys.read_raw_ir()
      }
   }
}

impl Display for SystemView{
   fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
      write!(f,concat!(
             "r0: {}\n",
             "r1: {}\n",
             "r2: {}\n",
             "r3: {}\n",
             "r4: {}\n",
             "r5: {}\n", 
             "r6: {}\n",
             "r7: {}\n",
             "r8: {}\n", 
             "r9: {}\n",
             "r10: {}\n",
             "r11: {}\n",
             "r12: {}\n",
             "SP: {:#010x}\n",
             "LR: {:#x}\n",
             "PC: {:#x}\n",
             "XPSR: {:#010x}",
            ),
            self.registers.generic[0],
            self.registers.generic[1],
            self.registers.generic[2],
            self.registers.generic[3],
            self.registers.generic[4],
            self.registers.generic[5],
            self.registers.generic[6],
            self.registers.generic[7],
            self.registers.generic[8],
            self.registers.generic[9],
            self.registers.generic[10],
            self.registers.generic[11],
            self.registers.generic[12],
            self.sp,
            self.registers.lr,
            self.sp,
            self.xpsr
      )
   }
}

struct MemoryByte{
}

struct BkptInput{
   pub pending_addr_or_symbol: String,  
}

impl BkptInput{
   pub fn try_get_addr(&self, symbols: &Vec<(usize,String)>)->Option<u32>{
      match parse_hex(&self.pending_addr_or_symbol){
         Some(addr) => Some(addr),
         None => {
            let treated = self.pending_addr_or_symbol.trim();
            for (addr, symbol) in symbols{
               if symbol.eq(treated) && !is_segment_mapping_symbol(symbol){
                  return Some(*addr as u32);
               }
            }
            println!("could not identify symbol: {}",self.pending_addr_or_symbol);
            return None;
         },
      }
   }
}

#[derive(Clone,Debug,PartialEq,Eq)]
pub enum Cast{
   UWORD,
   IWORD,
   UHALF,
   IHALF,
   UBYTE,
   IBYTE
}

fn stringify_slice(arr: &[u8],cast: Cast)->String{
   let mut display = String::new();
   match cast{
      Cast::UWORD => {
         for word in arr.chunks_exact(4){
            let byte_pair: [u8;4] = word.try_into().expect("always 4 byte arr");
            let native = from_arm_bytes(byte_pair);
            display.push_str(&(native).to_string());
            display.push(' ');
         }
      },
      Cast::IWORD => {
         for word in arr.chunks_exact(4){
            let byte_pair: [u8;4] = word.try_into().expect("always 4 byte arr");
            let native = from_arm_bytes(byte_pair);
            display.push_str(&(native as i32).to_string());
            display.push(' ');
         }
      },
      Cast::UHALF => {
         for hw in arr.chunks_exact(2){
            let byte_pair: [u8;2] = hw.try_into().expect("always 2 byte arr");
            let native = from_arm_bytes_16b(byte_pair);
            display.push_str(&(native).to_string());
            display.push(' ');
         }
     },
      Cast::IHALF => for hw in arr.chunks_exact(2){
         let byte_pair: [u8;2] = hw.try_into().expect("always 2 byte arr");
         let native = from_arm_bytes_16b(byte_pair);
         display.push_str(&(native as i16).to_string());
         display.push(' ');
      },
      Cast::UBYTE => display = format!("{:?}",arr),
      Cast::IBYTE => {
         for byte in arr{
            display.push_str(&(*byte as i8).to_string());
            display.push(' ');
         }
      },
   }
   display
}

static CAST_OPTIONS: &[Cast] = &[Cast::UWORD, Cast::IWORD, Cast::UHALF, Cast::IHALF, Cast::UBYTE, Cast::IBYTE];

impl Display for Cast{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
       let rep = match self{
          Cast::UWORD => "u32",
          Cast::IWORD => "i32",
          Cast::UHALF => "u16",
          Cast::IHALF => "i16",
          Cast::UBYTE => "u8",
          Cast::IBYTE => "i8",
       };
       write!(f,"{}",rep)
    }
}

struct MemoryView{
   pub start: u32, 
   pub end: u32,
   pub cast: Cast
}

impl Default for MemoryView{
    fn default() -> Self {
        Self{start: 0, end: 0xFFFF, cast: Cast::UBYTE}
    }
}

fn user_cmds<'a>(bkpt: &BkptInput)->Element<'a, Event>{
   let bkpt_button = text_input("toggle breakpoint at address / symbol", &bkpt.pending_addr_or_symbol)
      .on_input(|s|Event::Ui(Gui::SetBkptInput(s)))
      .on_submit(Event::Ui(Gui::SubmitBkpt));
   row![
      button(text("step").size(TEXT_SIZE)).on_press(Event::Dbg(Debug::Step)),
      button(text("continue").size(TEXT_SIZE)).on_press(Event::Dbg(Debug::Continue)),
      bkpt_button
   ].spacing(5).into()
}

fn pane_cmds<'a>(n_panes: usize, pane: pane_grid::Pane)->Element<'a, Event>{
   row![
      button(text("D>")).on_press(Event::Ui(Gui::SplitPane(pane,PaneType::Disassembler,pane_grid::Axis::Vertical))),
      button(text("D^")).on_press(Event::Ui(Gui::SplitPane(pane,PaneType::Disassembler,pane_grid::Axis::Horizontal))),
      button(text("R>")).on_press(Event::Ui(Gui::SplitPane(pane,PaneType::SystemState,pane_grid::Axis::Vertical))),
      button(text("M>")).on_press(Event::Ui(Gui::SplitPane(pane,PaneType::MemoryExplorer,pane_grid::Axis::Vertical))),
      button(text("M^")).on_press(Event::Ui(Gui::SplitPane(pane,PaneType::MemoryExplorer,pane_grid::Axis::Horizontal))),
      if n_panes > 1{
         button(text("X")).on_press(Event::Ui(Gui::ClosePane(pane)))
      }else{
         button(text("X"))
      }
   ].spacing(5).into()
}

fn pane_render<'a>(
   app: &App,
   state: &PaneType
   )->Element<'a, Event>{
   match state{
      PaneType::Disassembler => {
         let ir = app.sys_view.raw_ir;
         let highlighted = app.disasm.lines().take(1).next().unwrap();
         let text_widget: iced::widget::Text<'a,iced::Renderer> = if highlighted.contains("<"){
            text(highlighted).size(TEXT_SIZE).style(iced::color!(100,0,0))
         }else{
            text(highlighted).size(TEXT_SIZE)
         };
         let mut text_box: iced::widget::Column<'a,Event,iced::Renderer> = column![text_widget];
         text_box = text_box.spacing(0).padding(0);
         let mut un_highlighted = String::new();
         for line in app.disasm.lines().skip(1){
            if line.contains("<"){
               //if !un_highlighted.trim().is_empty(){
                  text_box = text_box.push(
                     text(&un_highlighted)
                     .size(TEXT_SIZE)
                     .width(iced::Length::Fill)
                  );
               //}
               text_box = text_box.push(
                  text(line)
                  .size(TEXT_SIZE)
                  .width(iced::Length::Fill)
                  .style(iced::color!(100,0,0))
               );
               un_highlighted.clear();
            }else{
               if !line.trim().is_empty(){
                  let offset = line.split(":").next();
                  match offset{
                     Some(address) => {
                        let add_v = u32::from_str_radix(
                           address.trim().trim_start_matches("0x").trim(),
                           16
                        ).unwrap();
                        if add_v == ir{
                           if !un_highlighted.trim().is_empty(){
                              text_box = text_box.push(
                                 text(&un_highlighted)
                                 .size(TEXT_SIZE)
                                 .width(iced::Length::Fill)
                              );
                           }
                           text_box = text_box.push(
                              text(line)
                              .size(TEXT_SIZE)
                              .width(iced::Length::Fill)
                              .font(iced::Font{
                                 weight: iced::font::Weight::Bold,
                                 .. Default::default()
                              })
                           );
                           un_highlighted.clear();

                        }else if app.breakpoints.contains(&add_v){
                           if !un_highlighted.trim().is_empty(){
                              text_box = text_box.push(
                                 text(&un_highlighted)
                                 .size(TEXT_SIZE)
                                 .width(iced::Length::Fill)
                              );
                           }
                           un_highlighted.clear();
                           text_box = text_box.push(container(text(line).size(TEXT_SIZE)).style(brkpt_theme).padding(0));
                        }else{
                           un_highlighted.push_str(line);
                           un_highlighted.push('\n');
                        }
                     },
                     None => {
                        un_highlighted.push_str(line);
                        un_highlighted.push('\n');
                     }
                  }
               }else{
                  un_highlighted.push('\n');
               }

               //un_highlighted.push_str(line);
               //un_highlighted.push('\n');
            };
         }
         if !un_highlighted.trim().is_empty(){
            text_box = text_box.push(text(&un_highlighted).size(TEXT_SIZE).width(iced::Length::Fill));
         }
         //let content = text(&app.disasm).size(TEXT_SIZE).width(iced::Length::Fill).style(iced::color!(100,0,0));
         container(scrollable(text_box))
            .width(iced::Length::Fill)
            .height(iced::Length::Fill)
            .into()
      },

      PaneType::SystemState => {
         let str_state = format!("{}",app.sys_view);
         scrollable(text(str_state).size(TEXT_SIZE)).width(iced::Length::Fill).height(iced::Length::Fill).into()
      },

      PaneType::MemoryExplorer => {
         let view = app.mem_view.as_ref().unwrap_or_else(|| &MemoryView{start: 0,end: 0xFFFF, cast: Cast::UBYTE});
         let inputs = row(vec![
            text_input("start (hex)",&app.pending_mem_start)
               .on_input(|s| Event::Ui(Gui::Exp(Explorer::SetStart(s))))
               .on_submit(Event::Ui(Gui::Exp(Explorer::Update)))
               .into(),

            text_input("end (hex)",&app.pending_mem_end)
               .on_input(|s| Event::Ui(Gui::Exp(Explorer::SetEnd(s))))
               .on_submit(Event::Ui(Gui::Exp(Explorer::Update)))
               .into(),

            pick_list(CAST_OPTIONS, Some(view.cast.clone()), |c| Event::Ui(Gui::Exp(Explorer::SetCast(c))))
               .into()
         ]);
         //println!("should update view {}",app.update_view);
         let text_box = if app.view_error.is_some(){
               scrollable(text(app.view_error.as_ref().unwrap()).size(TEXT_SIZE).width(iced::Length::Fill))
            }else{
               if app.update_view{
                  match app.sync_sys.try_lock(){
                    Ok(sys) => {
                       let start = std::cmp::min(sys.memory.len() - 2,view.start as usize);
                       let end = std::cmp::min(sys.memory.len() - 1,view.end as usize);
                       let real_start = std::cmp::min(start,end);
                       let real_end = std::cmp::max(start,end);

                       let data = &sys.memory[real_start ..= real_end];
                       let string_data = stringify_slice(data, view.cast.clone());
                       scrollable(text(&string_data).size(TEXT_SIZE).width(iced::Length::Fill))
                    },
                    Err(_) => {
                       println!("could not acquire dbg thread lock!!!");
                       scrollable(text("...").size(TEXT_SIZE).width(iced::Length::Fill))
                    },
                }
               }else{
                  scrollable(text("... press enter to update view").size(TEXT_SIZE).width(iced::Length::Fill))
               }
         };

         container(
            column![
               inputs,
               text_box
            ]
         ).width(iced::Length::Fill).height(iced::Length::Fill).into()
      }
   }
}

fn brkpt_theme(theme: &Theme)->container::Appearance{
   let palette = theme.extended_palette();
   container::Appearance{
      background: Some(iced::Background::Color(iced::color!(30,100,200))),
      text_color: Some(iced::color!(0,0,0)),
      border_width: 10.0,
      border_color: palette.background.strong.color,
      ..Default::default()
   }
}

fn focused_pane(theme: &Theme)->container::Appearance{
   let palette = theme.extended_palette();
   container::Appearance{
      background: Some(palette.background.weak.color.into()),
      border_width: 2.0,
      border_color: palette.background.strong.color,
      ..Default::default()
   }
}

impl Application for App{
   type Flags = (System, usize, Vec<(usize,String)>);
   type Message = Event;
   type Theme = Theme;
   type Executor = executor::Default;

   fn new(args: Self::Flags)->(Self,Command<Event>){
      let (state,first) = pane_grid::State::new(PaneType::Disassembler);
      
      let (sys,entry_point, symbols) = args;
      let disasm = disasm_text(&sys.memory, entry_point, &symbols);
      let starting_view: SystemView = (&sys).into();
      let sync_sys_arc = Arc::new(Mutex::new(sys));
      let mut msg = String::new(); 
      for i in disasm.into_iter(){
         msg.push_str(&i);
         msg.push('\n');
      }
      (Self{
         _state: state,
         n_panes: 1,
         focus: Some(first),
         sync_sys: sync_sys_arc,
         disasm: msg,
         entry_point,
         symbols,
         sys_view: starting_view,
         cmd_sender: None,
         mem_view: None,
         breakpoints: Vec::new(),
         pending_mem_start: "start (hex)".into(),
         pending_mem_end: "end (hex)".into(),
         update_view: false,
         view_error: None,
         bkpt_input: BkptInput { pending_addr_or_symbol: String::new() }
      },Command::none())
   }

   fn title(&self) -> String {
        "Armageddon Simulator".into()
    }

   fn subscription(&self) -> iced::Subscription<Self::Message> {
      let async_copy = self.sync_sys.clone();
      //assert_eq!(2,Arc::strong_count(&async_copy),"only one instance runs in continue mode, only one instance runs in step mode");
      iced::subscription::channel(0, 1, |mut output| async move {
         let (sndr, mut rcvr)  = iced_mpsc::channel(10);
         output.send(Event::Dbg(Debug::Connect(sndr))).await;
         let mut continue_mode = false;
         let mut halt = None;
         let mut exit = false;
         loop{
            match rcvr.select_next_some().await{
                Event::Ui(e) => panic!("invalid cmd sent to simulator loop {:?}",e),
                Event::Dbg(Debug::Halt(_)) => {
                   continue_mode = false;
                },

                Event::Dbg(Debug::CreateBreakpoint(addr))=>{
                   let mut sys = async_copy.lock().unwrap();
                   sys.add_breakpoint(addr);
                },

                Event::Dbg(Debug::DeleteBreakpoint(addr))=>{
                   let mut sys = async_copy.lock().unwrap();
                   sys.remove_breakpoint(addr);
                },

                Event::Dbg(Debug::Disconnect) => {
                   if !exit{
                      exit = true;
                   }
                   if !output.is_closed(){
                      output.close_channel();
                   }
                },
                Event::Dbg(Debug::Continue) => {
                   continue_mode = true;
                   while continue_mode{
                      let mut sys = async_copy.lock().unwrap();
                      if sys.on_breakpoint(){
                         continue_mode = false;
                         halt = Some(HaltType::breakpoint);
                      }else{
                         match Simulator::step_or_signal_halt_type(&mut sys){
                            Ok(_)=> {},
                            Err(e) => {continue_mode = false; halt = Some(e);}
                         }
                      }
                      match rcvr.try_next(){
                         Ok(event) => match event{
                            Some(eve) => match eve{
                               Event::Dbg(Debug::Halt(_)) => {
                                  continue_mode = false;
                               },
                               Event::Dbg(Debug::Disconnect) => {
                                  continue_mode = false;
                                  exit = true;
                                  output.close_channel();
                               },
                               Event::Dbg(e) => {
                                  panic!("invalid cmd {:?} sent to simulator loop", e)
                               },
                               Event::Ui(e) => {panic!("invalid cmd {:?} sent to sim loop", e)}
                            },
                            None => todo!(),
                         },
                         Err(_) => { },
                      }
                   }
                },
                Event::Dbg(e) => panic!("invalid cmd sent to simulator loop {:?}", e),
            }
            match halt{
                Some(h) => {
                   output.send(Event::Dbg(Debug::Halt(h))).await;
                   halt = None;
                },
                None => {},
            }
         }
      })
   }

   fn update(&mut self, message: Event) -> Command<Self::Message> {
      match message{
         Event::Ui(Gui::SplitPane(pane,kind, axis)) => {
            self._state.split(axis, &pane, kind);
            self.n_panes += 1;
         },
         Event::Ui(Gui::ResizePane(pane_grid::ResizeEvent{split, ratio})) => {
            self._state.resize(&split,ratio);
         },
         Event::Ui(Gui::ClosePane(pane)) => {
            self._state.close(&pane);
            self.n_panes -= 1;
         },

         Event::Ui(Gui::Exp(Explorer::Update)) => { 
            match parse_hex(&self.pending_mem_start){
               Some(v) => {
                  match self.mem_view{
                    Some(ref mut current) => current.start = v,
                    None => {
                       let mut new_view = MemoryView::default();
                       new_view.start = v;
                       new_view.end = if v == u32::MAX{ u32::MAX }else{ v + 1 };
                       self.mem_view = Some(new_view);
                    },
                  }
                  self.update_view = true;
                  self.view_error = None;
               },
               None => {
                  println!("should report error");
                  self.view_error = Some(format!("could not parse '{}' as a hexadecimal",&self.pending_mem_start));
               }
            }

            match parse_hex(&self.pending_mem_end){
               Some(v) => {
                  match self.mem_view{
                    Some(ref mut current) => current.end = v,
                    None => {
                       let mut new_view = MemoryView::default();
                       new_view.end = v;
                       new_view.start = if v == 0 {0}else{ v - 1 };
                       self.mem_view = Some(new_view);
                    },
                  }
                  self.update_view = true;
                  self.view_error = None;
               },
               None => {
                  println!("should report error");
                  self.view_error = Some(format!("could not parse '{}' as a hexadecimal",&self.pending_mem_end));
               }
            }
         },

         Event::Ui(Gui::Exp(Explorer::SetStart(s))) => {
            self.pending_mem_start = s;
            self.update_view = false;
         },

         Event::Ui(Gui::Exp(Explorer::SetEnd(e))) => {
            self.pending_mem_end = e;
            self.update_view = false;
         },

         Event::Ui(Gui::Exp(Explorer::SetCast(c))) => {
            match self.mem_view{
               Some(ref mut current) => current.cast = c,
               None => {
                  let mut new_view = MemoryView::default();
                  new_view.cast = c;
                  self.mem_view = Some(new_view);
               },
            }
         },

         Event::Ui(Gui::SetBkptInput(input)) => {
            self.bkpt_input.pending_addr_or_symbol = input;
         },

         Event::Ui(Gui::SubmitBkpt) => {
            match self.bkpt_input.try_get_addr(&self.symbols){
                Some(addr) => {
                   if self.breakpoints.contains(&addr){
                      match self.cmd_sender{
                         Some(ref mut sndr) =>{
                            sndr.try_send(Event::Dbg(Debug::DeleteBreakpoint(addr)));
                            self.breakpoints.retain(|x| *x != addr);
                         },
                         None => {panic!("cannot interact with dbg session")}
                      }
                   }else{
                      match self.cmd_sender{
                         Some(ref mut sndr)=>{
                            sndr.try_send(Event::Dbg(Debug::CreateBreakpoint(addr)));
                            self.breakpoints.push(addr);
                         },
                         None => {panic!("cannot interact with dbg session")}
                      }
                   }
                },
                None => {println!("could not parse {} as address",&self.bkpt_input.pending_addr_or_symbol)},
            }
         },

         Event::Dbg(Debug::Step) => {
            let mut sys = self.sync_sys.try_lock().unwrap();
            Simulator::step_or_signal_halt(&mut sys).unwrap();
            self.sys_view = sys.deref().into();
         },

         Event::Dbg(Debug::Connect(sender)) => {
            self.cmd_sender = Some(sender);
            println!("connected with dbg thread");
         },
         Event::Dbg(Debug::Continue) => {
            assert!(self.cmd_sender.is_some(),"cannot use continue dbg thread not connected");
            match self.cmd_sender.as_mut(){
               Some(sndr) => {
                  sndr.try_send(Event::Dbg(Debug::Continue)).unwrap();
               },
               None => {},
            }
         },
         Event::Dbg(Debug::Halt(_type))=>{
            println!("dbg session halted due to {:?}",_type);
            let sys = self.sync_sys.try_lock().unwrap();
            self.sys_view = sys.deref().into();
         }
         _ => todo!()
      }

      Command::none()
    }

   fn view(&self) -> iced::Element<'_, Self::Message, iced::Renderer<Self::Theme>> {
      let layout = PaneGrid::new(&self._state, |id, pane, _maximised|{
         let title_bar = pane_grid::TitleBar::new("Armageddon").controls(pane_cmds(self.n_panes,id)).padding(10).style(focused_pane);
         pane_grid::Content::new(
            pane_render(&self,pane)
         ).title_bar(title_bar)
      }.style(focused_pane))
      .on_resize(10,|e| Event::Ui(Gui::ResizePane(e)));
      column![user_cmds(&self.bkpt_input),layout].into()
      //layout.into()
    }
}

/*pub enum Breakpoint{
   Create(usize),
   Delete(usize)
}*/

use iced::futures::channel::mpsc as iced_mpsc;
use iced::futures::sink::SinkExt; 
#[derive(Debug,Clone)]
pub enum Debug{
   Halt(HaltType),
   Continue,
   Step,
   Disconnect,
   CreateBreakpoint(u32),
   DeleteBreakpoint(u32),
   Connect(iced_mpsc::Sender<Event>)
}

#[derive(Debug,Clone)]
pub enum Gui{
   SplitPane(pane_grid::Pane,PaneType,pane_grid::Axis),
   ResizePane(pane_grid::ResizeEvent),
   ClosePane(pane_grid::Pane),
   Exp(Explorer),
   SetBkptInput(String),
   SubmitBkpt
}

#[derive(Debug,Clone)]
pub enum Explorer{
   SetStart(String),
   SetEnd(String),
   SetCast(Cast),
   Update,
}

#[derive(Debug,Clone)]
pub enum Event{
   Ui(Gui),
   Dbg(Debug)
}

#[derive(Debug,Clone)]
pub enum PaneType{
   Disassembler,
   SystemState,
   MemoryExplorer
}

fn parse_hex(hex: &str)->Option<u32>{
   if hex.starts_with("0x"){
      match hex.trim().strip_prefix("0x"){
         Some(h) => {
            match u32::from_str_radix(h,16){
               Ok(v) => Some(v),
               Err(_) => {println!("could not parse {} as hex",hex);None}
            }
         },
         None => None
      }
   }else{
      match u32::from_str_radix(hex,16){
         Ok(v) => Some(v),
         Err(_) => {println!("could not parse {} as hex",hex);None}
      }
   }
}
