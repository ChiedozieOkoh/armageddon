use std::{fmt::Display, sync::{Mutex, Arc}, ops::{Deref, DerefMut}, path::{Path, PathBuf}, borrow::BorrowMut};

use iced::{widget::{pane_grid, PaneGrid, text, column, container, scrollable, row, button, vertical_slider::StyleSheet, pick_list, image, tooltip, mouse_area, Row}, Application, Theme, executor, Command, Element, futures::StreamExt};
use iced::widget::text_input;

use crate::{system::{System, ArmException, simulator::{HaltType, Simulator}, registers::Registers, self, write_memory}, asm::interpreter::{print_assembly, disasm_text, is_segment_mapping_symbol, TextPosition}, binutils::{from_arm_bytes_16b, u32_to_arm_bytes}, elf::decoder::SymbolDefinition, to_arm_bytes};

use crate::dbg_ln;
use crate::binutils::from_arm_bytes;
const TEXT_SIZE: u16 = 11;

pub mod searchbar;
pub mod window;

pub struct App{
   _state: pane_grid::State<PaneType>,
   n_panes: usize,
   focused_pane: pane_grid::Pane,
   diasm_windows: Window,
   entry_point: usize,
   pub disasm: String,
   register_hex_display: [bool;13],
   searchbar: Option<SearchBar>,
   symbols: Vec<SymbolDefinition>,
   mem_view: Option<MemoryView>,
   sys_view: SystemView,
   sync_sys: Arc<Mutex<System>>,
   cmd_sender: Option<iced_mpsc::Sender<Event>>,
   breakpoints: Vec<u32>,
   view_error: Option<String>,
   pending_mem_start: String,
   pending_mem_end: String,
   trace_record: String,
   update_view: bool,
   bkpt_input: BkptInput
}

struct SystemView{
   pub mode: system::Mode,
   pub registers: Registers,
   pub sp: u32,
   pub xpsr: u32,
   pub raw_ir: u32
}

impl From<&System> for SystemView{
   fn from(sys: &System) -> Self {
      Self{
         mode: sys.mode.clone(),
         registers: sys.registers.clone(),
         sp: sys.get_sp(),
         xpsr: from_arm_bytes(sys.xpsr),
         raw_ir: sys.read_raw_ir()
      }
   }
}

fn inlay_button_ref(components: Element<'static,Event>,on_click: Event, highlight: bool) -> iced::widget::MouseArea<'static,Event,iced::Renderer> {
   if highlight{
      return mouse_area(
         container(components).style(brkpt_theme)
      ).on_release(on_click);
   }else{
      return mouse_area(
         container(components)
      ).on_release(on_click);
   }
}

fn inlay_button(label: String,on_click: Event, highlight: bool) -> iced::widget::MouseArea<'static,Event,iced::Renderer> {
   if highlight{
      mouse_area(
         container(
            text(label).size(TEXT_SIZE).width(iced::Length::Shrink)
         ).style(brkpt_theme)
      ).on_release(on_click)
   }else{
      mouse_area(text(label).size(TEXT_SIZE).width(iced::Length::Shrink))
      .on_release(on_click)
   }
}

fn adjustable_register(reg_num: u32,name: &str, value: u32,in_hex: bool) -> Row<Event>{
   let label = if in_hex{
      format!("  {}: {:#x}",name,value)
   }else{
      format!("  {}: {}",name,value)
   };
   row![
   inlay_button(label,Event::Ui(Gui::ToggleRegisterDisplay(reg_num)),false)
   ]
}

impl Display for SystemView{
   fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
      write!(f,concat!(
             "mode: {:?}\n",
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
             "LR: {:#010x}\n",
             "PC: {:#010x}\n",
             "XPSR: {:#010x}",
            ),
            self.mode,
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
            self.registers.pc,
            self.xpsr
      )
   }
}

struct BkptInput{
   pub pending_addr_or_symbol: String,  
}

impl BkptInput{
   pub fn try_get_addr(&self, symbols: &Vec<SymbolDefinition>)->Option<u32>{
      match parse_hex(&self.pending_addr_or_symbol){
         Some(addr) => Some(addr),
         None => {
            let treated = self.pending_addr_or_symbol.trim();
            for symbol in symbols{
               if symbol.name.eq(treated) && !is_segment_mapping_symbol(&symbol.name){
                  return Some(symbol.position as u32);
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

macro_rules! parse_hex_or_base10 {
   ($_type:ty,$string:expr,$is_hex:expr) => {
      if $is_hex{
         match <$_type>::from_str_radix($string,16){
            Ok(v) => Some(v),
            Err(_) => {None},
         }
      }else{
         match <$_type>::from_str_radix($string,10){
            Ok(v) => Some(v),
            Err(_) => {None},
         }
      }
   }
}

fn stringify_slice(mut offset: u32, arr: &[u8], cast: Cast)->String{
   let mut display = String::new();
   match cast{
      Cast::UWORD => {
         for word in arr.chunks_exact(4){
            let byte_pair: [u8;4] = word.try_into().expect("always 4 byte arr");
            let native = from_arm_bytes(byte_pair);
            display.push_str(&format!("{:#010X}: {}\n",offset,native));
            offset += 4;
         }
      },
      Cast::IWORD => {
         for word in arr.chunks_exact(4){
            let byte_pair: [u8;4] = word.try_into().expect("always 4 byte arr");
            let native = from_arm_bytes(byte_pair);
            display.push_str(&format!("{:#010X}: {}\n",offset,native as i32));
            offset += 4;
         }
      },
      Cast::UHALF => {
         for hw in arr.chunks_exact(2){
            let byte_pair: [u8;2] = hw.try_into().expect("always 2 byte arr");
            let native = from_arm_bytes_16b(byte_pair);
            display.push_str(&format!("{:#010X}: {}\n",offset,native as u16));
            offset += 2;
         }
     },
      Cast::IHALF => for hw in arr.chunks_exact(2){
         let byte_pair: [u8;2] = hw.try_into().expect("always 2 byte arr");
         let native = from_arm_bytes_16b(byte_pair);
            display.push_str(&format!("{:#010X}: {}\n",offset,native as i16));
            offset += 2;
      },
      Cast::UBYTE => {
         for byte in arr{
            display.push_str(&format!("{:#010X}: {}\n",offset,*byte));
            offset += 1;
         }
      }
      Cast::IBYTE => {
         for byte in arr{
            display.push_str(&format!("{:#010X}: {}\n",offset,*byte as i8));
            offset += 1;
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
   pub view_cast: Cast,
   //pub entry_cast: Cast,
   pub raw_entry_data: String,
   pub sanitised_entry: Vec<u8>
}

impl Default for MemoryView{
   fn default() -> Self {
      Self{
         start: 0,
         end: 0xFFFF,
         view_cast: Cast::UBYTE,
         //entry_cast: Cast::UBYTE,
         raw_entry_data: String::new(),
         sanitised_entry: Vec::new()
      }
   }
}

fn user_cmds<'a>(bkpt: &BkptInput)->Element<'a, Event>{
   let bkpt_button = text_input("toggle breakpoint at address / symbol", &bkpt.pending_addr_or_symbol)
      .on_input(|s|Event::Ui(Gui::SetBkptInput(s)))
      .on_submit(Event::Ui(Gui::SubmitBkpt));
   row![
      button(text("step").size(TEXT_SIZE)).on_press(Event::Dbg(Debug::Step)),
      button(text("continue").size(TEXT_SIZE)).on_press(Event::Dbg(Debug::Continue)),
      button(text("halt").size(TEXT_SIZE)).on_press(Event::Ui(Gui::SubmitHalt)),
      button(text("reset").size(TEXT_SIZE)).on_press(Event::Dbg(Debug::Reset)),
      bkpt_button
   ].spacing(5).into()
}

fn img_button<'a>(msg: &str, signal: Event,alt_signal: Event, tip: &str) -> Element<'a,Event>{
   let btn = tooltip(
      button(text(msg)).on_press(signal),
      tip,
      tooltip::Position::Bottom
   ).gap(10).style(iced::theme::Container::Box);
   mouse_area(btn)
      .on_right_release(alt_signal)
      .into()
}

macro_rules! split_pane_event {
    ($a:expr,$b:path,$c:path) => {
       Event::Ui(Gui::SplitPane($a,$b,$c))
    };
}

fn pane_cmds<'a>(n_panes: usize, pane: pane_grid::Pane,n_breakpoints: usize)->Element<'a, Event>{
   use pane_grid::Axis::Vertical as Vertical;
   use pane_grid::Axis::Horizontal as Horizontal;
   let mut cmds = row![
      img_button(
         "disassembly",
         split_pane_event!(pane,PaneType::Disassembler,Vertical),
         split_pane_event!(pane,PaneType::Disassembler,Horizontal),
         "Open disassembly (right click to split horizontally)"
      ),
      img_button(
         "registers",
         split_pane_event!(pane,PaneType::SystemState,Vertical),
         split_pane_event!(pane,PaneType::SystemState,Horizontal),
         "View cpu registers (right click to split horizontally)"
      ),
      img_button(
         "memory",
         split_pane_event!(pane,PaneType::MemoryExplorer,Vertical),
         split_pane_event!(pane,PaneType::MemoryExplorer,Horizontal),
         "view a region of memory (right click to split horizontally)"
      ),
      img_button(
         "logs",
         split_pane_event!(pane,PaneType::Trace,Vertical),
         split_pane_event!(pane,PaneType::Trace,Horizontal),
         "view a log of recently executed instructions (right click to split horizontally)"
      ),
      //button(text("R>")).on_press(Event::Ui(Gui::SplitPane(pane,PaneType::SystemState,pane_grid::Axis::Vertical))),
      //button(text("M>")).on_press(Event::Ui(Gui::SplitPane(pane,PaneType::MemoryExplorer,pane_grid::Axis::Vertical))),
      //button(text("M^")).on_press(Event::Ui(Gui::SplitPane(pane,PaneType::MemoryExplorer,pane_grid::Axis::Horizontal))),
      if n_panes > 1{
         button(text("close")).on_press(Event::Ui(Gui::ClosePane(pane)))
      }else{
         button(text("close"))
      }
   ].spacing(5);

   if n_breakpoints > 0 {
      cmds = cmds.push(
         button(text("clear breakpoints")).on_press(Event::Ui(Gui::SubmitBkptClear))
      )
   }

   cmds.into()
}

/*
fn highlight_or_default<'a>(target: &String,line: &str, search_result: &Vec<TextPosition>, sr_idx: usize,_focus: bool)->(usize,Row<'a,Event,iced::Renderer>){
   let current = current_search_result(&search_results,sr_idx);
   let mut new_idx = sr_idx;
   let brkpt_text = if current.is_some_and(|sr| sr == line_number){
      let v = highlight_search_result(
         &app.searchbar.as_ref().unwrap().target,
         line,
         &search_results[sr_idx],
         false
         );
      new_idx += 1;
      v
   }else{
      row![text(line).size(TEXT_SIZE)]
   };
   return (new_idx,brkpt_text);
}
*/

fn highlight_search_result<'a>(target: &String,line: &str,current_result: &TextPosition, _focus: bool, normal_font: Option<(iced::Color,iced::Font)>)->Row<'a,Event,iced::Renderer>{
   let offset = current_result.line_offset;
   let hl_len = target.len();
   let hl_region = line.get(offset .. offset + hl_len).unwrap();
   let before = line.get(.. offset);
   let after =  line.get(offset + hl_len ..);
   let mut text_box = Row::new();
   let highlight_clr = if _focus{
      iced::color!(0,0,100)
   }else{
      iced::color!(100,0,0)
   };
   if before.is_some(){
      let mut t = text(before.unwrap()).size(TEXT_SIZE);
      if normal_font.is_some(){
         let (c,f) = normal_font.clone().unwrap();
         t = t.style(c)
              .font(f);
      }
      text_box = text_box.push(t);
   }

   text_box = text_box.push(
      text(hl_region)
         .size(TEXT_SIZE)
         .style(highlight_clr)
         .font(iced::Font{
            weight: iced::font::Weight::Bold,
            .. Default::default()
         })
   );

   if after.is_some(){
      let mut t = text(after.unwrap()).size(TEXT_SIZE);
      if normal_font.is_some(){
         let (c,f) = normal_font.clone().unwrap();
         t = t.style(c)
              .font(f);
      }
      text_box = text_box.push(t);
   }

   text_box.padding(0).height(iced::Length::Shrink)
}

fn pane_render<'a>(
   app: &App,
   id: &pane_grid::Pane,
   state: &PaneType
   )->Element<'a, Event>{
   match state{
      PaneType::Disassembler => {
         let ir = app.sys_view.raw_ir;
         let mut line_number: usize = 0; 
         let mut sr_idx: usize = 0;
         let search_results = if app.searchbar.is_some(){
            app.searchbar.as_ref().unwrap().text_occurances()
         }else{
            vec![]
         };

         let current_search_result = |r: &Vec<TextPosition>, counter: usize|{
            if r.is_empty(){
               return None
            }
            if counter >= r.len(){
               return None;
            }
            return Some(r[counter].line_number);
         };

         let highlighted = app.disasm.lines().take(1).next().unwrap();
         let c_search_result = current_search_result(&search_results,sr_idx);
         let text_widget: Row<Event>= if highlighted.contains("<"){
            row![text(highlighted).size(TEXT_SIZE).style(iced::color!(100,0,0))]
         }else if c_search_result.is_some_and(|sr| sr == line_number){
            println!("for search term '{}'",app.searchbar.as_ref().unwrap().target);
            println!("current search result ptr {:?}",c_search_result);
            let current = sr_idx;
            let focused = app.searchbar.as_ref().unwrap().is_nth_term_focused(current);
            sr_idx += 1;
            highlight_search_result(&app.searchbar.as_ref().unwrap().target, highlighted, &search_results[current], focused, None)
         }else{
            row![text(highlighted).size(TEXT_SIZE)]
         };

         line_number += 1;
         let mut text_box: iced::widget::Column<'a,Event,iced::Renderer> = column![text_widget];
         text_box = text_box.spacing(0).padding(0).height(iced::Length::Shrink);

         for line in app.disasm.lines().skip(1){
            if !line.trim().is_empty(){
               let offset = line.split(":").next().unwrap();
               let addr = u32::from_str_radix(
                  offset.trim().trim_start_matches("0x").trim(),
                  16
                  ).unwrap();

               let on_symbol = line.contains("<");
               let on_exec_intr = addr == ir;
               let on_breakpoint = app.breakpoints.contains(&addr);

               let normal_font = if on_exec_intr & !on_symbol{
                  iced::Font{ weight: iced::font::Weight::Bold, .. Default::default()}
               }else{
                  iced::Font{ .. Default::default()}
               };

               let normal_colour = if on_symbol{
                  iced::color!(100,0,0)
               }else{
                  iced::color!(0,0,0)
               };
               let current = current_search_result(&search_results,sr_idx);
               let inner_text  = if current.is_some_and(|sr| sr == line_number){
                  let focused = app.searchbar.as_ref()
                     .unwrap()
                     .is_nth_term_focused(sr_idx);

                  let v = highlight_search_result(
                     &app.searchbar.as_ref().unwrap().target,
                     line,
                     &search_results[sr_idx],
                     focused,
                     Some((normal_colour,normal_font))
                     );
                  sr_idx += 1;
                  v
               }else{
                  let inter = text(line)
                     .size(TEXT_SIZE)
                     .style(normal_colour)
                     .width(iced::Length::Fill)
                     .font(normal_font);
                  row![
                     inter
                  ]
               };

               let msg = Event::Ui(Gui::SubmitGuiBkpt(addr));
               let rendered_line = inlay_button_ref(inner_text.into(), msg, on_breakpoint );
               text_box = text_box.push(rendered_line);
            }else{
               text_box = text_box.push(text("\n"));
            }
            line_number += 1;
         }
         //let content = text(&app.disasm).size(TEXT_SIZE).width(iced::Length::Fill).style(iced::color!(100,0,0));
         container(scrollable(text_box.spacing(0)).id(app.diasm_windows.id_of(id).unwrap().clone()))
            .width(iced::Length::Fill)
            .height(iced::Length::Fill)
            .into()
      },

      PaneType::SystemState => {
         let sview = &app.sys_view;
         column![
            text(format!("  mode: {:?}",sview.mode))
               .size(TEXT_SIZE)
               .width(iced::Length::Fill),
            adjustable_register(
               0,
               "r0",
               sview.registers.generic[0],
               app.register_hex_display[0]
            ),
            adjustable_register(
               1,
               "r1",
               sview.registers.generic[1],
               app.register_hex_display[1]
            ),
            adjustable_register(
               2,
               "r2",
               sview.registers.generic[2],
               app.register_hex_display[2]
            ),
            adjustable_register(
               3,
               "r3",
               sview.registers.generic[3],
               app.register_hex_display[3]
            ),
            adjustable_register(
               4,
               "r4",
               sview.registers.generic[4],
               app.register_hex_display[4]
            ),
            adjustable_register(
               5,
               "r5",
               sview.registers.generic[5],
               app.register_hex_display[5]
            ),
            adjustable_register(
               6,
               "r6",
               sview.registers.generic[6],
               app.register_hex_display[6]
            ),
            adjustable_register(
               7,
               "r7",
               sview.registers.generic[7],
               app.register_hex_display[7]
            ),
            adjustable_register(
               8,
               "r8",
               sview.registers.generic[8],
               app.register_hex_display[8]
            ),
            adjustable_register(
               9,
               "r9",
               sview.registers.generic[9],
               app.register_hex_display[9]
            ),
            adjustable_register(
               10,
               "r10",
               sview.registers.generic[10],
               app.register_hex_display[10]
            ),
            adjustable_register(
               11,
               "r11",
               sview.registers.generic[11],
               app.register_hex_display[11]
            ),
            adjustable_register(
               12,
               "r12",
               sview.registers.generic[12],
               app.register_hex_display[12]
            ),
            text(format!("  SP: {:#010x}",sview.sp))
               .size(TEXT_SIZE)
               .width(iced::Length::Fill),

            text(format!("  LR: {:#010x}",sview.registers.lr))
               .size(TEXT_SIZE)
               .width(iced::Length::Fill),

            text(format!("  PC: {:#010x}",sview.registers.pc))
               .size(TEXT_SIZE)
               .width(iced::Length::Fill),

            text(format!("  XPSR: {:#010x}",sview.xpsr))
               .size(TEXT_SIZE)
               .width(iced::Length::Fill)
         ].into()
      },

      PaneType::MemoryExplorer => {
         let def= MemoryView{
            start: 0,
            end: 32,
            view_cast: Cast::UBYTE,
            raw_entry_data: String::new(),
            sanitised_entry: Vec::new()
         };
         let view = app.mem_view.as_ref().unwrap_or_else(|| {&def});
         let inputs = row(vec![
            text_input("start (hex)",&app.pending_mem_start)
               .on_input(|s| Event::Ui(Gui::Exp(Explorer::SetStart(s))))
               .on_submit(Event::Ui(Gui::Exp(Explorer::Update)))
               .into(),

            text_input("end (hex)",&app.pending_mem_end)
               .on_input(|s| Event::Ui(Gui::Exp(Explorer::SetEnd(s))))
               .on_submit(Event::Ui(Gui::Exp(Explorer::Update)))
               .into(),

            pick_list(CAST_OPTIONS, Some(view.view_cast.clone()), |c| Event::Ui(Gui::Exp(Explorer::SetViewCast(c))))
               .into()
         ]);
         //println!("should update view {}",app.update_view);
         let text_box = if app.view_error.is_some(){
               scrollable(text(app.view_error.as_ref().unwrap()).size(TEXT_SIZE).width(iced::Length::Fill))
            }else{
               if app.update_view{
                  match app.sync_sys.try_lock(){
                     Ok(sys) => {
                        //let start = std::cmp::min(sys.memory.len() - 2,view.start as usize);
                        //let end = std::cmp::min(sys.memory.len() - 1,view.end as usize);
                        let real_start = std::cmp::min(view.start,view.end);
                        let real_end = std::cmp::max(view.start,view.end);

                        //let data = &sys.memory[real_start ..= real_end];
                        let data = sys.alloc.view(real_start,real_end);
                        let string_data = stringify_slice(real_start,&data, view.view_cast.clone());
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

         /*let data_entry = row![
            text_input(
               "write data at this memory address",
               &view.raw_entry_data
            )
         ];*/
         container(
            column![
               inputs,
               text_box,
               //data_entry
            ]
         ).width(iced::Length::Fill).height(iced::Length::Fill).into()
      }

      PaneType::Trace=>{
         let content = scrollable(text(&app.trace_record).size(TEXT_SIZE).width(iced::Length::Fill));
         container(
            column![
               content
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

fn normal_pane(theme: &Theme)->container::Appearance{
   let palette = theme.extended_palette();
   container::Appearance{
      background: Some(palette.background.weak.color.into()),
      border_width: 2.0,
      border_color: palette.background.strong.color,
      ..Default::default()
   }
}

fn focused_pane(theme: &Theme)->container::Appearance{
   let palette = theme.extended_palette();
   container::Appearance{
      background: Some(palette.background.weak.color.into()),
      border_width: 2.0,
      border_color: iced::Color{r: 1.0, g: 0.0, b: 0.0, a: 1.0},
      ..Default::default()
   }
}

impl Application for App{
   type Flags = (System, usize, Vec<SymbolDefinition>,String);
   type Message = Event;
   type Theme = Theme;
   type Executor = executor::Default;

   fn new(args: Self::Flags)->(Self,Command<Event>){
      let (state,def) = pane_grid::State::new(PaneType::Disassembler);
      
      let (mut sys,entry_point, symbols, disassembly) = args;
      sys.trace_enabled = true;
      let starting_view: SystemView = (&sys).into();
      let sync_sys_arc = Arc::new(Mutex::new(sys));
      let mut windows = Window::create();
      windows.add_pane(def.clone());
      (Self{
         _state: state,
         focused_pane: def,
         n_panes: 1,
         diasm_windows: windows,
         sync_sys: sync_sys_arc,
         disasm: disassembly,
         entry_point,
         symbols,
         register_hex_display: [false;13],
         sys_view: starting_view,
         cmd_sender: None,
         mem_view: None,
         searchbar: None,
         breakpoints: Vec::new(),
         pending_mem_start: "start (hex)".into(),
         pending_mem_end: "end (hex)".into(),
         update_view: false,
         view_error: None,
         trace_record: String::new(),
         bkpt_input: BkptInput { pending_addr_or_symbol: String::new() }
      },Command::none())
   }

   fn title(&self) -> String {
        "Armageddon Simulator".into()
    }

   fn subscription(&self) -> iced::Subscription<Self::Message> {
      use iced::keyboard::Event as KeyEvent;
      let shortcuts = iced::subscription::events_with(|event, status|{
         match event{
            iced::Event::Keyboard(KeyEvent::KeyReleased { key_code, modifiers }) => match key_code{
                iced::keyboard::KeyCode::Enter => {
                   if modifiers.alt() && matches!(status,iced::event::Status::Ignored){
                      Some(Event::Ui(Gui::CentreDisassembler))
                   }else{
                      None
                   }
                },
                iced::keyboard::KeyCode::D=>{
                   if modifiers.control() && matches!(status,iced::event::Status::Ignored){
                      Some(Event::Ui(Gui::SubmitBkptClear))
                   }else{
                      None
                   }
                },
                iced::keyboard::KeyCode::F =>{
                   if modifiers.control(){
                      Some(Event::Ui(Gui::OpenSearchBar))
                   }else{
                      None
                   }
                },
                iced::keyboard::KeyCode::Backspace=>{
                   if matches!(status,iced::event::Status::Ignored){
                      Some(Event::Ui(Gui::CloseFocusedPane))
                   }else{
                      None
                   }
                },
                _ => None,
            },
            _ => None
         }
      });
      let async_copy = self.sync_sys.clone();
      //assert_eq!(2,Arc::strong_count(&async_copy),"only one instance runs in continue mode, only one instance runs in step mode");
      let sim_runtime = iced::subscription::channel(0, 1, |mut output| async move {
         let (sndr, mut rcvr)  = iced_mpsc::channel(10);
         output.send(Event::Dbg(Debug::Connect(sndr))).await;
         let mut halt = None;
         loop{
            match rcvr.select_next_some().await{
               Event::Ui(e) => panic!("invalid cmd sent to simulator loop {:?}",e),
               Event::Dbg(Debug::Reset)=>{
                  let mut sys = async_copy.lock().unwrap();
                  sys.reset();
                  halt = Some(HaltType::usercmd);
               }

               Event::Dbg(Debug::Halt(HaltType::usercmd)) => {
                  halt = Some(HaltType::usercmd);
               },

               Event::Dbg(Debug::CreateBreakpoint(addr))=>{
                  let mut sys = async_copy.lock().unwrap();
                  sys.add_breakpoint(addr);
               },

               Event::Dbg(Debug::DeleteBreakpoint(addr))=>{
                  let mut sys = async_copy.lock().unwrap();
                  sys.remove_breakpoint(addr);
               },

               Event::Dbg(Debug::ClearBreakpoints)=>{
                  let mut sys = async_copy.lock().unwrap();
                  sys.clear_breakpoints();
               }

               Event::Dbg(Debug::Disconnect) => {
                  if !output.is_closed(){
                     output.close_channel();
                  }
               },
               Event::Dbg(Debug::Continue) => {
                  let mut continue_mode = true;
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
                              Event::Dbg(Debug::Halt(e)) => {
                                 continue_mode = false;
                                 halt = Some(e);
                              },
                              Event::Dbg(Debug::Disconnect) => {
                                 continue_mode = false;
                                 output.close_channel();
                              },
                              Event::Dbg(Debug::Reset)=>{
                                 continue_mode = false;
                                 sys.reset();
                                 halt = Some(HaltType::usercmd);
                              },
                              Event::Dbg(Debug::CreateBreakpoint(addr))=>{
                                 sys.add_breakpoint(addr);
                              },
                              Event::Dbg(Debug::DeleteBreakpoint(addr))=>{
                                 sys.remove_breakpoint(addr);
                              },
                              Event::Dbg(Debug::ClearBreakpoints)=>{
                                 sys.clear_breakpoints();
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
                   let _ = output.send(Event::Dbg(Debug::Halt(h))).await;
                   halt = None;
                },
                None => {},
            }
         }
      });
      iced::subscription::Subscription::batch(vec![shortcuts,sim_runtime])
   }

   fn update(&mut self, message: Event) -> Command<Self::Message> {
      let mut cmd = Command::none();
      match message{
         Event::Ui(Gui::SplitPane(pane,kind, axis)) => {
            if let Some(p) = self._state.split(axis, &pane, kind.clone()){
               let (new_pane,_) = p;
               match kind{
                  PaneType::Disassembler => {
                     let _  = self.diasm_windows.add_pane(new_pane.clone());
                     self.diasm_windows.focus_if_present(&new_pane);
                  },
                  _ => {}
               }
               self.focused_pane = new_pane;
            }
            self.n_panes += 1;
         },

         Event::Ui(Gui::ResizePane(pane_grid::ResizeEvent{split, ratio})) => {
            self._state.resize(&split,ratio);
         },

         Event::Ui(Gui::CloseFocusedPane)=>{
            if self.n_panes >= 2{
               let old_focus = self.focused_pane.clone();
               if let Some(other_pane) = self._state.adjacent(&self.focused_pane, pane_grid::Direction::Up){
                  self.focused_pane = other_pane;
               }else if let Some(other_pane) = self._state.adjacent(&self.focused_pane, pane_grid::Direction::Left){
                  self.focused_pane = other_pane;
               }else if let Some(other_pane) = self._state.adjacent(&self.focused_pane, pane_grid::Direction::Down){
                  self.focused_pane = other_pane;
               }else if let Some(other_pane) = self._state.adjacent(&self.focused_pane, pane_grid::Direction::Right){
                  self.focused_pane = other_pane;
               }
               self.diasm_windows.focus_if_present(&self.focused_pane);
               self.diasm_windows.remove_pane(&old_focus);
               self._state.close(&old_focus);
               self.n_panes -=1;
            }
         }

         Event::Ui(Gui::ClosePane(pane)) => {
            if pane.eq(&self.focused_pane){
               if let Some(other_pane) = self._state.adjacent(&pane, pane_grid::Direction::Up){
                  self.focused_pane = other_pane;
               }else if let Some(other_pane) = self._state.adjacent(&pane, pane_grid::Direction::Left){
                  self.focused_pane = other_pane;
               }else if let Some(other_pane) = self._state.adjacent(&pane, pane_grid::Direction::Down){
                  self.focused_pane = other_pane;
               }else if let Some(other_pane) = self._state.adjacent(&pane, pane_grid::Direction::Right){
                  self.focused_pane = other_pane;
               }
            }
            self.diasm_windows.focus_if_present(&self.focused_pane);
            self.diasm_windows.remove_pane(&pane);
            self._state.close(&pane);
            self.n_panes -= 1;
         },

         Event::Ui(Gui::FocusPane(pane))=>{
            self.diasm_windows.focus_if_present(&pane);
            self.focused_pane = pane;
         },

         Event::Ui(Gui::OpenSearchBar)=>{ self.searchbar = Some(SearchBar::create()); },

         Event::Ui(Gui::SetSearchInput(input))=> {
            self.searchbar.as_mut().unwrap().pending = input;
         },

         Event::Ui(Gui::CloseSearchBar)=>{ self.searchbar = None; },

         Event::Ui(Gui::FocusNextSearchResult)=>{
            let _ = self.searchbar.as_mut().unwrap().focus_next();
            let maybe_position = self.searchbar.as_ref()
               .unwrap()
               .get_focused_search_result();

            match maybe_position{
                Some(position) => {
                   let total_lines = self.disasm.lines().count();
                   let ratio = position.line_number as f32 / total_lines as f32;
                   dbg_ln!("estimated ratio {} / {} =  {}",
                           position.line_number,
                           total_lines,
                           ratio
                   );
                   if let Some(id) = self.diasm_windows.get_focused_pane(){
                      cmd = iced::widget::scrollable::snap_to(
                         id.clone(),
                         scrollable::RelativeOffset { x: 0.0, y: ratio }
                      );
                   }
                },
                None => println!("no matches found"),
            }
         },

         Event::Ui(Gui::SubmitSearch)=>{
            let sb = self.searchbar.as_mut().unwrap();
            sb.target = sb.pending.clone();
            //let sys = self.sync_sys.try_lock().unwrap();
            match sb.find(&self.disasm){
                Ok(_) => {},
                Err(e) => println!("error occured during search {:?}",e),
            }
         },

         Event::Ui(Gui::CentreDisassembler)=>{
            let ir = self.sys_view.raw_ir;
            let mut ir_ln = 0_u32;
            let mut line_number = 0_u32;
            for line in self.disasm.lines(){
               if !line.trim().is_empty(){
                  let offset = line.split(":").next();

                  match offset{
                     Some(address) => {
                        dbg_ln!("parsing {}",address);
                        let add_v = u32::from_str_radix(
                           address.trim().trim_start_matches("0x").trim(),
                           16
                        ).unwrap();
                        if add_v == ir{
                           ir_ln = line_number;
                        }
                     },
                     _ => {}
                  }
               }
               line_number += 1;
            }
            let y_ratio = (ir_ln as f32) / line_number as f32;
            dbg_ln!("estimated ratio {} / {} =  {}",ir_ln,line_number,y_ratio);
            if let Some(id) = self.diasm_windows.get_focused_pane(){
               println!("snapping to {:?}",id);
               cmd = iced::widget::scrollable::snap_to(
                  id.clone(),
                  scrollable::RelativeOffset { x: 0.0, y: y_ratio }
               );
            }
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

            match self.mem_view{
                Some(ref mut v) => {
                   let trimmed = v.raw_entry_data.trim();
                   if !trimmed.is_empty(){
                      match self.sync_sys.try_lock(){
                         Ok(mut sys) => {
                            let mut address = v.start;
                            if trimmed.contains(char::is_whitespace){
                               for values in trimmed.split_whitespace(){
                                  match v.view_cast{
                                     Cast::UWORD => {
                                        let maybe_word = parse_hex_or_base10!(u32, values, values.starts_with("0x"));
                                        match maybe_word{
                                           Some(word)=> {
                                              let bytes = to_arm_bytes!(u32,word);
                                              let r = write_memory(sys.deref_mut().into(), address, bytes);
                                              match r{
                                                 Ok(_) => address += 4,
                                                 Err(e) => {
                                                    Simulator::register_exceptions(sys.deref_mut().into(), e.clone());
                                                    if e.is_fault(){
                                                       match e{
                                                          ArmException::HardFault(msg) => {
                                                             let mut err_msg = String::from("Could not complete memory write due to hardfault\n");
                                                             err_msg.push_str(&msg);
                                                             self.view_error = Some(err_msg)
                                                          },
                                                          _ => {self.view_error = Some(String::from("could not complete memory write due to fault"));}
                                                       }
                                                       break;
                                                    }
                                                 },
                                              }
                                           },
                                           None => {}
                                        }

                                        address += 4;
                                     },
                                     Cast::IWORD => {
                                     },
                                     Cast::UHALF => {
                                     },
                                     Cast::IHALF => {
                                     },
                                     Cast::UBYTE => {
                                     },
                                     Cast::IBYTE => {
                                     },
                                  }
                               }
                            }else{
                            }
                         },
                         Err(_) => println!("cannot edit memory in continue mode"),
                      }
                   }
                },
                None => {},
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

         Event::Ui(Gui::Exp(Explorer::SetViewCast(c))) => {
            match self.mem_view{
               Some(ref mut current) => current.view_cast = c,
               None => {
                  let mut new_view = MemoryView::default();
                  new_view.view_cast = c;
                  self.mem_view = Some(new_view);
               },
            }
         },

         Event::Ui(Gui::Exp(Explorer::SetEntry(e)))=> {
            match self.mem_view{
               Some(ref mut current) => current.raw_entry_data = e,
               None => {
                  let mut new_view = MemoryView::default();
                  new_view.raw_entry_data = e;
                  self.mem_view = Some(new_view);
               },
            }
         },

         Event::Ui(Gui::ToggleRegisterDisplay(i))=>{
            self.register_hex_display[i as usize] = !self.register_hex_display[i as usize];
         },

         Event::Ui(Gui::SubmitBkptClear)=>{
            match self.cmd_sender{
               Some(ref mut sndr)=>{
                  let _ = sndr.try_send(Event::Dbg(Debug::ClearBreakpoints));
                  self.breakpoints.clear();
               },
               None => {panic!("cannot interact with dbg session")}
            }
         },

         Event::Ui(Gui::SubmitHalt)=>{
            match self.cmd_sender{
               Some(ref mut sndr)=>{
                  println!("sending halt to dbg session");
                  let _ = sndr.try_send(Event::Dbg(Debug::Halt(HaltType::usercmd)));
               },
               None => {panic!("cannot interact with dbg session")}
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

         Event::Ui(Gui::SubmitGuiBkpt(addr))=>{
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

         Event::Dbg(Debug::Step) => {
            let mut sys = self.sync_sys.try_lock().unwrap();
            Simulator::step_or_signal_halt(&mut sys).unwrap();
            self.trace_record = sys.trace.clone();
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
         Event::Dbg(Debug::Reset)=>{
            match self.cmd_sender.as_mut(){
               Some(sndr)=>{
                  sndr.try_send(Event::Dbg(Debug::Reset)).unwrap();
               },
               None =>{
                  println!("WARN: cannot interact with debug session");
                  let mut sys = self.sync_sys.try_lock().unwrap();
                  sys.reset();
                  self.sys_view = sys.deref().into();
               }
            }
         },
         Event::Dbg(Debug::Halt(_type))=>{
            println!("dbg session halted due to {:?}",_type);
            let sys = self.sync_sys.try_lock().unwrap();
            self.trace_record = sys.trace.clone();
            self.sys_view = sys.deref().into();
         }
         _ => todo!()
      }

      cmd
    }


   fn view(&self) -> iced::Element<'_, Self::Message, iced::Renderer> {
      let pane_buttons = pane_cmds(self.n_panes,self.focused_pane.clone(),self.breakpoints.len());
      let layout = PaneGrid::new(&self._state, |id, pane, _maximised|{
         let is_focused = id == self.focused_pane;
         let title_bar = pane_titles(pane,is_focused);
         pane_grid::Content::new(
            pane_render(&self,&id,pane)
         ).title_bar(title_bar)
      }.style(normal_pane))
      .on_resize(10,|e| Event::Ui(Gui::ResizePane(e)))
      .on_click(|p| Event::Ui(Gui::FocusPane(p)));

      if self.searchbar.is_some(){
         column![
            user_cmds(&self.bkpt_input),
            pane_buttons,
            searchbar(&self.searchbar.as_ref().unwrap()),
            layout
         ].into()
      }else{
         column![
            user_cmds(&self.bkpt_input),
            pane_buttons,
            layout
         ].into()
      }
      //layout.into()
    }
}

fn searchbar<'a>(bar: &'a SearchBar)->iced::Element<'a,Event>{
   let close = button("close").on_press(Event::Ui(Gui::CloseSearchBar));
   let next = button("next").on_press(Event::Ui(Gui::FocusNextSearchResult));
   let input = text_input(&bar.help(), &bar.pending)
      .on_input(|s|Event::Ui(Gui::SetSearchInput(s)))
      .on_submit(Event::Ui(Gui::SubmitSearch));
   row![next,input,close].into()
}

fn pane_titles(kind: &PaneType, focused: bool) -> pane_grid::TitleBar<Event> {
   match &kind{
      PaneType::Disassembler =>{
         pane_grid::TitleBar::new("Armageddon (disassembly)")
            .padding(10).style(if focused{focused_pane}else{normal_pane})
      } ,
      PaneType::SystemState => {
         pane_grid::TitleBar::new("Armageddon (registers)")
            .padding(10).style(if focused{focused_pane}else{normal_pane})
      },
      PaneType::MemoryExplorer => {
         pane_grid::TitleBar::new("Armageddon (memory viewer)")
            .padding(10).style(if focused{focused_pane}else{normal_pane})
      },
      PaneType::Trace => {
         pane_grid::TitleBar::new("Armageddon (execution trace)")
            .padding(10).style(if focused{focused_pane}else{normal_pane})
      },
   }
}
/*pub enum Breakpoint{
   Create(usize),
   Delete(usize)
}*/

use iced::futures::channel::mpsc as iced_mpsc;
use iced::futures::sink::SinkExt;

use self::{searchbar::SearchBar, window::Window}; 
#[derive(Debug,Clone)]
pub enum Debug{
   Halt(HaltType),
   Continue,
   Step,
   Disconnect,
   Reset,
   CreateBreakpoint(u32),
   DeleteBreakpoint(u32),
   ClearBreakpoints,
   Connect(iced_mpsc::Sender<Event>)
}

#[derive(Debug,Clone)]
pub enum Gui{
   SplitPane(pane_grid::Pane,PaneType,pane_grid::Axis),
   ResizePane(pane_grid::ResizeEvent),
   FocusPane(pane_grid::Pane),
   ClosePane(pane_grid::Pane),
   CloseFocusedPane,
   Exp(Explorer),
   SetBkptInput(String),
   SubmitBkpt,
   SubmitGuiBkpt(u32),
   SubmitHalt,
   SubmitBkptClear,
   OpenSearchBar,
   SubmitSearch,
   FocusNextSearchResult,
   CloseSearchBar,
   SetSearchInput(String),
   CentreDisassembler,
   ToggleRegisterDisplay(u32)
}

#[derive(Debug,Clone)]
pub enum Explorer{
   SetStart(String),
   SetEnd(String),
   SetViewCast(Cast),
   SetEntry(String),
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
   MemoryExplorer,
   Trace
}

pub fn parse_hex(hex: &str)->Option<u32>{
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

