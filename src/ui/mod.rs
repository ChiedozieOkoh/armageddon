use iced::{widget::{pane_grid, PaneGrid, text, column, container, scrollable, row, button, vertical_slider::StyleSheet}, Application, Theme, executor, Command, Element};

use crate::{system::System, asm::interpreter::{print_assembly, disasm_text}};

use crate::binutils::from_arm_bytes;
const TEXT_SIZE: u16 = 11;

pub struct App{
   _state: pane_grid::State<PaneType>,
   n_panes: usize,
   focus: Option<pane_grid::Pane>,
   pub system: System,
   entry_point: usize,
   pub disasm: String,
   symbols: Vec<(usize,String)>
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
         text(str_state).size(TEXT_SIZE).width(iced::Length::Fill).into()
      },

      _ => todo!()
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
         symbols
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
   Halt,
   Continue,
   Step,
   CreateBreakpoint(usize),
   DeleteBreakpoint(usize)
}

#[derive(Debug,Clone)]
pub enum Gui{
   SplitPane(pane_grid::Pane,PaneType,pane_grid::Axis),
   ResizePane(pane_grid::ResizeEvent),
   ClosePane(pane_grid::Pane),
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
