use iced::{widget::{pane_grid, PaneGrid, text, column, container, scrollable, row, button}, Application, Theme, executor, Command, Element};

use crate::{system::System, asm::interpreter::{print_assembly, disasm_text}};

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
         let content = text(&app.disasm).size(TEXT_SIZE).width(iced::Length::Fill);
         container(scrollable(content))
            .width(iced::Length::Fill)
            .height(iced::Length::Fill)
            .into()
      },

      PaneType::SystemState => {
         let str_state = format!(
            "r0: {}\nr1: {}\nr2: {}\nr3: {}\nr4: {}\nr5: {}\nr6: {}\nr7: {}\nr8: {}\nr9: {}\nr10: {}\nr11: {}\nr12: {}\nSP: {:#010x}\nLR: {:#x}\nPC: {:#x}\n",
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
            app.system.read_pc_word_aligned()
         );
         text(str_state).size(TEXT_SIZE).width(iced::Length::Fill).into()
      }
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
      let (mut state,first) = pane_grid::State::new(PaneType::Disassembler);
      
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
   DefLayout,
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
