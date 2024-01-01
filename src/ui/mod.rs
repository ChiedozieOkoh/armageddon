use std::fmt::Display;

use iced::{widget::{pane_grid, PaneGrid, text, column, container, scrollable, row, button, vertical_slider::StyleSheet, pick_list}, Application, Theme, executor, Command, Element};
use iced::widget::text_input;

use crate::{system::{System, ArmException, simulator::HaltType}, asm::interpreter::{print_assembly, disasm_text}};

use crate::binutils::from_arm_bytes;
const TEXT_SIZE: u16 = 11;

pub struct App{
   _state: pane_grid::State<PaneType>,
   n_panes: usize,
   focus: Option<pane_grid::Pane>,
   pub system: System,
   entry_point: usize,
   pub disasm: String,
   symbols: Vec<(usize,String)>,
   mem_view: Option<MemoryView>,
   view_error: Option<String>,
   pending_mem_start: String,
   pending_mem_end: String,
   update_view: bool
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

fn user_cmds<'a>()->Element<'a, Event>{
   row![
      button(text("step").size(TEXT_SIZE)).on_press(Event::Dbg(Debug::Step)),
      button(text("continue").size(TEXT_SIZE)).on_press(Event::Dbg(Debug::Continue))
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
         let ir = app.system.read_raw_ir();
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
         let str_state = format!(
            "r0: {}\nr1: {}\nr2: {}\nr3: {}\nr4: {}\nr5: {}\nr6: {}\nr7: {}\nr8: {}\nr9: {}\nr10: {}\nr11: {}\nr12: {}\nSP: {:#010x}\nLR: {:#x}\nPC: {:#x}\nXPSR: {:#010x}",
            app.system.registers.generic[0],
            app.system.registers.generic[1],
            app.system.registers.generic[2],
            app.system.registers.generic[3],
            app.system.registers.generic[4],
            app.system.registers.generic[5],
            app.system.registers.generic[6],
            app.system.registers.generic[7],
            app.system.registers.generic[8],
            app.system.registers.generic[9],
            app.system.registers.generic[10],
            app.system.registers.generic[11],
            app.system.registers.generic[12],
            app.system.get_sp(),
            app.system.registers.lr,
            app.system.read_raw_ir(),
            from_arm_bytes(app.system.xpsr)
         );
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
         let start = std::cmp::min(app.system.memory.len() - 2,view.start as usize);
         let end = std::cmp::min(app.system.memory.len() - 1,view.end as usize);
         let real_start = std::cmp::min(start,end);
         let real_end = std::cmp::max(start,end);

         let data = &app.system.memory[real_start ..= real_end];
         //println!("should update view {}",app.update_view);
         let text_box = if app.view_error.is_some(){
               scrollable(text(app.view_error.as_ref().unwrap()).size(TEXT_SIZE).width(iced::Length::Fill))
            }else{
               if app.update_view{
                  scrollable(text(&format!("{:?}",data)).size(TEXT_SIZE).width(iced::Length::Fill))
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
      let mut msg = String::new(); 
      for i in disasm.into_iter(){
         msg.push_str(&i);
         msg.push('\n');
      }
      (Self{
         _state: state,
         n_panes: 1,
         focus: Some(first),
         system: sys,
         disasm: msg,
         entry_point,
         symbols,
         mem_view: None,
         pending_mem_start: "start (hex)".into(),
         pending_mem_end: "end (hex)".into(),
         update_view: false,
         view_error: None
      },Command::none())
   }

   fn title(&self) -> String {
        "Armageddon Simulator".into()
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
         Event::Dbg(Debug::Step) => {
            match self.system.step(){
               Ok(offset) => {
                  self.system.offset_pc(offset).unwrap();
               },
               Err(fault) => {
                  panic!("{:?}",fault);
               }
            }
         }
         _ => todo!()
      }

      Command::none()
    }

   fn view(&self) -> iced::Element<'_, Self::Message, iced::Renderer<Self::Theme>> {
      let layout = PaneGrid::new(&self._state, |id, pane, _maximised|{
         let title_bar = pane_grid::TitleBar::new("Armageddon").controls(pane_cmds(self.n_panes,id)).padding(10).style(focused_pane);
         pane_grid::Content::new(
            column![user_cmds(),
            pane_render(&self,pane)].padding(10)
         ).title_bar(title_bar)
      }.style(focused_pane))
      .on_resize(10,|e| Event::Ui(Gui::ResizePane(e)));
      layout.into()
    }
}

/*pub enum Breakpoint{
   Create(usize),
   Delete(usize)
}*/

#[derive(Debug,Clone)]
pub enum Debug{
   Halt(HaltType),
   Continue,
   Step,
   Disconnect,
   CreateBreakpoint(u32),
   DeleteBreakpoint(u32)
}

#[derive(Debug,Clone)]
pub enum Gui{
   SplitPane(pane_grid::Pane,PaneType,pane_grid::Axis),
   ResizePane(pane_grid::ResizeEvent),
   ClosePane(pane_grid::Pane),
   Exp(Explorer),
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
               Err(_) => {println!("could not parse {}",hex);None}
            }
         },
         None => None
      }
   }else{
      match u32::from_str_radix(hex,16){
         Ok(v) => Some(v),
         Err(_) => {println!("could not parse {}",hex);None}
      }
   }
}
