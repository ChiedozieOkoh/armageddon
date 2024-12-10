use core::fmt;
use std::fmt::Debug;

use crate::binutils::{get_set_bits, signed_bitfield,umax,smin,smax};
use crate::system::registers::SpecialRegister;
use crate::dbg_ln;

use super::{
   DestRegister,
   HalfWord,
   Literal,
   Register,
   SrcRegister,
   Word, STACK_POINTER, PROGRAM_COUNTER
};

use crate::asm::decode::{Opcode,B16,B32};
use crate::binutils::{from_arm_bytes_16b, get_bitfield, BitList,sign_extend};

#[allow(non_camel_case_types)]
#[derive(Debug)]
pub enum Operands{
   ADD_REG_SP_IMM8(DestRegister,Literal<8>),
   INCR_SP_BY_IMM7(Literal<7>),
   INCR_SP_BY_REG(Register),
   ADR(DestRegister,Literal<8>),
   ASRS_Imm5(DestRegister,SrcRegister,Literal<5>),
   COND_BRANCH(i32),
   B_ALWAYS(i32),
   BREAKPOINT(Literal<8>),
   BR_LNK(i32),
   BR_LNK_EXCHANGE(Register),
   BR_EXCHANGE(Register),
   CMP_Imm8(Register,Literal<8>),
   LDR_Imm5(DestRegister,SrcRegister,Literal<5>),
   LDR_Imm8(DestRegister,SrcRegister,Literal<8>),
   LDR_REG(DestRegister,SrcRegister,Register),
   LS_Imm5(DestRegister,SrcRegister,Literal<5>),
   MOV_REG(DestRegister,SrcRegister),
   DestImm8(DestRegister,Literal<8>),
   LoadableList(Register,BitList),
   RegisterPair(DestRegister,Register),
   RegPairImm3(DestRegister,SrcRegister,Literal<3>),
   RegisterTriplet(DestRegister,SrcRegister,Register),
   PureRegisterPair(Register,Register),
   RegisterList(BitList),
   STR_Imm5(SrcRegister,Register,Literal<5>),
   STR_Imm8(SrcRegister,Literal<8>),
   STR_REG(SrcRegister,Register,Register),
   SP_SUB(Literal<7>),
   Byte(Literal<8>),
   HalfWord(Literal<16>),
   Primask(bool),
   MSR(SpecialRegister,SrcRegister),
   MRS(DestRegister,SpecialRegister),
   Nibble(Literal<4>)
}

fn fmt_branch_offset(f: &mut std::fmt::Formatter<'_>,offset: i32)->fmt::Result{
   if offset == 0{
      return write!(f,".");
   }
   if offset.is_positive(){
      write!(f,". + {}",offset)
   }else{
      write!(f,".{}",offset)
   }
}

fn serialise_branch_offset(buffer: &mut String,offset: i32){
   if offset == 0{
      buffer.push('.');
   }else{
      if offset.is_positive(){
         buffer.push_str(". + ");
         u32_to_b10(buffer,offset as u32);
      }else{
         buffer.push_str(". - ");
         u32_to_b10(buffer,offset.unsigned_abs());
      }
   }
}

//TODO dont hardcode special register names
impl fmt::Display for Operands{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
       match self{
          Operands::ADD_REG_SP_IMM8(r, imm8) => write!(f,"{}, SP, {}", r, imm8),
          Operands::INCR_SP_BY_IMM7(imm7) => write!(f,"SP, {}",imm7),
          Operands::INCR_SP_BY_REG(r) => write!(f,"SP, {}",r),
          Operands::ADR(r, imm8) => write!(f,"{}, {}", r, imm8),
          Operands::ASRS_Imm5(d, r, imm5) => write!(f,"{}, {}, {}", d, r, imm5),
          Operands::COND_BRANCH(off) => fmt_branch_offset(f, *off),
          Operands::B_ALWAYS(off) => fmt_branch_offset(f, *off),
          Operands::BREAKPOINT(imm8) => write!(f,"{}",imm8),
          Operands::BR_LNK(off) => fmt_branch_offset(f, *off),
          Operands::BR_LNK_EXCHANGE(r) => write!(f,"{}",r),
          Operands::BR_EXCHANGE(r) => write!(f,"{}",r),
          Operands::CMP_Imm8(r, imm8) => write!(f,"{}, {}",r, imm8),
          Operands::LDR_Imm5(d, s, imm5) => write!(f,"{}, [{}, {}]", d, s, imm5),
          Operands::LDR_Imm8(d,s,imm8) => write!(f,"{}, [{}, {}]",d,s, imm8),
          Operands::LDR_REG(d, s, r) => write!(f,"{}, [{}, {}]",d, s, r),
          Operands::LS_Imm5(d, s, imm5) => write!(f,"{}, {}, {}", d, s, imm5),
          Operands::MOV_REG(d, s) => write!(f,"{}, {}", d, s),
          Operands::DestImm8(d, imm8) => write!(f,"{}, {}", d, imm8),
          Operands::LoadableList(base, list) => {
             let registers = get_set_bits(*list);
             let mut list_str = String::new();
             if (1 << base.0) & list > 0 {
                list_str.push_str(&format!("r{},",base.0));
             }else{
                list_str.push_str(&format!("r{}!,",base.0));
             }
             list_str.push_str(&fmt_register_list(registers));
             write!(f, "{}", list_str)
          },
          Operands::RegisterPair(d, r) => write!(f, "{}, {}", d, r),
          Operands::RegPairImm3(d, s, imm3) => write!(f, "{}, {}, {}", d, s, imm3),
          Operands::RegisterTriplet(d, s, a) => write!(f, "{}, {}, {}", d, s, a),
          Operands::PureRegisterPair(a, b) => write!(f, "{}, {}", a, b),
          Operands::RegisterList(list) => {
             let registers = get_set_bits(*list);
             write!(f,"{}",fmt_register_list(registers))
          },
          Operands::STR_Imm5(s, base, imm5) => write!(f, "{}, [{}, {}]", s, base, imm5),
          Operands::STR_Imm8(s, imm8) => write!(f, "{}, [SP, {}]", s, imm8),
          Operands::STR_REG(s, base, offset_reg) => write!(f, "{}, [{}, {}]", s, base, offset_reg),
          Operands::SP_SUB(imm7) => write!(f, "SP, SP, {}", imm7),
          Operands::Byte(imm8) => write!(f, "{}", imm8),
          Operands::HalfWord(imm16) => write!(f, "{}", imm16),
          Operands::Primask(flag) => {
             if *flag{
                write!(f, "CPSID i")
             }else{
                write!(f, "CPSIE i")
             }
          },
          Operands::MSR(meta, src) => write!(f, "{:?}, {}", meta, src),
          Operands::MRS(src, meta) => write!(f, "{}, {:?}", src, meta),
          Operands::Nibble(imm4) => write!(f, "{}", imm4),
       }
    }
}

pub fn hint_address(address: u32, offset: u32)->u32{
   (address + 4 + offset) & !0x3
}

pub fn pretty_print(_address: u32, operands: &Operands)->String{
   let mut line = if cfg!(debug_assertions){
      dbg_print(operands)
   }else{
      format!("{}", operands)
   };
   match operands{
      Operands::LDR_Imm8(_,src,imm8) =>{
         if src.0 == super::PROGRAM_COUNTER{
            let hint = (_address + 4 + imm8.0) & !0x3 ;
            line.push_str(&format!("    //@{:#010x}",hint));
         }
      },
      Operands::LDR_Imm5(_,src,imm5) =>{
         if src.0 == super::PROGRAM_COUNTER{
            let hint = (_address + 4 + imm5.0) & !0x3 ;
            line.push_str(&format!("    //@{:#010x}",hint));
         }
      },
      _ => {}
   } 
   return line;
}

pub fn u32_to_hex(buffer: &mut String , num: u32){
   let table = ['0','1','2','3','4','5','6','7','8','9','A','B','C','D','E','F'];
   buffer.push_str("0x");
   let mut mask = 0xF0000000;
   for i in 0 .. 8{
      let k: usize = ( (num & mask) >> (28 - (i * 4))) as usize;
      buffer.push(table[k]);
      mask = mask >> 4;
   }
}

pub fn u32_to_b10(buffer: &mut String, mut num: u32){
   let dict = ['0','1','2','3','4','5','6','7','8','9'];
   let mut number:  [u32;10] = [0,0,0,0,0,0,0,0,0,0];
   let mut i = 0;
   loop{
      if num < 10 {
         //println!("{}th digit = {}",i,num);
         number[i] = num;
         break;
      }else{
         //println!("{}th digit = {}",i,num % 10);
         //println!("next test = {}",num / 10);
         number[i] = num % 10;
         num = num / 10;
      } 

      i += 1;
   }
   for x in (0 ..=i).rev(){
      buffer.push(dict[number[x] as usize]);
   }
}

#[inline]
pub fn serialise_register(buffer: &mut String, number: u8){
   let table = ["r0","r1","r2","r3","r4","r5","r6","r7","r8","r9","r10","r11","r12"];
   match number{
      0 ..=12 => buffer.push_str(table[number as usize]),
      13 => buffer.push_str("SP"),
      14 => buffer.push_str("LR"),
      15 => buffer.push_str("PC"),
      _ => unreachable!("invalid register number")
   }
}

//TODO implement this without write! macro

pub fn serialise_operand(buffer: &mut String, operands: &Operands, address: u32){
   match operands{
      Operands::ADD_REG_SP_IMM8(r, imm8) => {
         serialise_register(buffer,r.0);
         buffer.push_str(", SP, #");
         u32_to_b10(buffer,imm8.0);
      },
      Operands::INCR_SP_BY_IMM7(imm7) => {
         buffer.push_str("SP, #");
         u32_to_b10(buffer,imm7.0);
      },
      Operands::INCR_SP_BY_REG(r) =>{
         buffer.push_str("SP, ");
         serialise_register(buffer,r.0);
      }, 
      Operands::ADR(r, imm8) => {
         serialise_register(buffer,r.0);
         buffer.push_str(", #");
         u32_to_b10(buffer,imm8.0);
      },
      Operands::ASRS_Imm5(d, r, imm5) => {
         serialise_register(buffer,d.0);
         buffer.push_str(", ");
         serialise_register(buffer,r.0);
         buffer.push_str(", #");
         u32_to_b10(buffer,imm5.0);
      },
      Operands::COND_BRANCH(off) => serialise_branch_offset(buffer, *off),
      Operands::B_ALWAYS(off) => serialise_branch_offset(buffer, *off),
      Operands::BREAKPOINT(imm8) => {
         buffer.push('#');
         u32_to_b10(buffer,imm8.0);
      },
      Operands::BR_LNK(off) => serialise_branch_offset(buffer, *off),
      Operands::BR_LNK_EXCHANGE(r) => {
         serialise_register(buffer,r.0);
      },
      Operands::BR_EXCHANGE(r) => {
         serialise_register(buffer,r.0);
      },
      Operands::CMP_Imm8(r, imm8) => {
         serialise_register(buffer,r.0);
         buffer.push_str(", #");
         u32_to_b10(buffer,imm8.0);
      },
      Operands::LDR_Imm5(d, s, imm5) => {
         serialise_register(buffer,d.0);
         buffer.push_str(", [");
         serialise_register(buffer, s.0);
         buffer.push_str(", #");
         u32_to_b10(buffer,imm5.0);
         buffer.push_str("]    //@");
         let hint = (address + 4 + imm5.0) & !0x3;
         u32_to_hex(buffer,hint);
      },
      Operands::LDR_Imm8(d,s,imm8) => {
         serialise_register(buffer,d.0);
         buffer.push_str(", [");
         serialise_register(buffer,s.0);
         buffer.push_str(", #");
         u32_to_b10(buffer,imm8.0);
         buffer.push_str("]    //@");
         let hint = (address + 4 + imm8.0) & !0x3;
         u32_to_hex(buffer,hint);
      },
      Operands::LDR_REG(d, s, r) => {
         serialise_register(buffer,d.0);
         buffer.push_str(", [");
         serialise_register(buffer,s.0);
         buffer.push_str(", ");
         serialise_register(buffer,r.0);
         buffer.push(']');
      },
      Operands::LS_Imm5(d, s, imm5) => {
         serialise_register(buffer,d.0);
         buffer.push_str(", ");
         serialise_register(buffer,s.0);
         buffer.push_str(", #");
         u32_to_b10(buffer, imm5.0);
      },
      Operands::MOV_REG(d, s) => {
         serialise_register(buffer,d.0);
         buffer.push_str(", ");
         serialise_register(buffer,s.0);
      },
      Operands::DestImm8(d, imm8) => {
         serialise_register(buffer,d.0);
         buffer.push_str(", #");
         u32_to_b10(buffer,imm8.0);
      },
      Operands::LoadableList(base, list) => {
       let registers = get_set_bits(*list);
       //let mut list_str = String::new();
       serialise_register(buffer,base.0);
       if (1 << base.0) & *list > 0 {
          //list_str.push_str(&format!("r{},",base.0));
          buffer.push(',');
       }else{
          //list_str.push_str(&format!("r{}!,",base.0));
          buffer.push_str("!,");
       }
       //list_str.push_str(&fmt_register_list(registers));
       serialise_register_list(buffer,registers);
       //write!(f, "{}", list_str)
      },
      Operands::RegisterPair(d, r) =>{
         serialise_register(buffer,d.0);
         buffer.push_str(", ");
         serialise_register(buffer,r.0);
      },
      Operands::RegPairImm3(d, s, imm3) => {
         serialise_register(buffer,d.0);
         buffer.push_str(", ");
         serialise_register(buffer,s.0);
         buffer.push_str(", #");
         u32_to_b10(buffer,imm3.0);
      },
      Operands::RegisterTriplet(d, s, a) => {
         serialise_register(buffer,d.0);
         buffer.push_str(", ");
         serialise_register(buffer,s.0);
         buffer.push_str(", ");
         serialise_register(buffer,a.0);
      },
      Operands::PureRegisterPair(a, b) => {
         serialise_register(buffer,a.0);
         buffer.push_str(", ");
         serialise_register(buffer,b.0);
      },
      Operands::RegisterList(list) => {
       let registers = get_set_bits(*list);
       serialise_register_list(buffer,registers);
       //write!(f,"{}",fmt_register_list(registers))
      },
      Operands::STR_Imm5(s, base, imm5) => {
         serialise_register(buffer,s.0);
         buffer.push_str(", [");
         serialise_register(buffer,base.0);
         buffer.push_str(", #");
         u32_to_b10(buffer,imm5.0);
         buffer.push(']');
      },
      Operands::STR_Imm8(s, imm8) => {
         serialise_register(buffer,s.0);
         buffer.push_str(", [SP, #");
         u32_to_b10(buffer,imm8.0);
         buffer.push(']');
      },
      Operands::STR_REG(s, base, offset_reg) => {
         serialise_register(buffer,s.0);
         buffer.push_str(", [");
         serialise_register(buffer,base.0);
         buffer.push_str(", ");
         serialise_register(buffer,offset_reg.0);
      },
      Operands::SP_SUB(imm7) => {
         buffer.push_str("SP, SP, #");
         u32_to_b10(buffer,imm7.0);
      },
      Operands::Byte(imm8) => {
         u32_to_b10(buffer,imm8.0);
      },
      Operands::HalfWord(imm16) => {
         u32_to_b10(buffer,imm16.0);
      },
      Operands::Primask(flag) => {
         if *flag{
            buffer.push_str("CPSID i");
            //write!(f, "CPSID i")
         }else{
            buffer.push_str("CPSIE i");
            //write!(f, "CPSIE i")
         }
      },
      Operands::MSR(meta, src) => {
         serialise_special_register(buffer,meta);
         buffer.push_str(", ");
         serialise_register(buffer,src.0);
      },
      Operands::MRS(dest, meta) => {
         serialise_register(buffer,dest.0);
         buffer.push_str(", ");
         serialise_special_register(buffer,meta);
      },
      Operands::Nibble(imm4) => {
         buffer.push('#');
         u32_to_b10(buffer,imm4.0);
      },
   }
}

fn serialise_special_register(buffer: &mut String, reg: &SpecialRegister){
   match reg{
      SpecialRegister::APSR => buffer.push_str("APSR"),
      SpecialRegister::IAPSR => buffer.push_str("IAPSR"),
      SpecialRegister::EAPSR => buffer.push_str("EAPSR"),
      SpecialRegister::XPSR => buffer.push_str("XPSR"),
      SpecialRegister::IPSR => buffer.push_str("IPSR"),
      SpecialRegister::EPSR => buffer.push_str("EPSR"),
      SpecialRegister::IEPSR => buffer.push_str("IEPSR"),
      SpecialRegister::MSP => buffer.push_str("MSP"),
      SpecialRegister::PSP => buffer.push_str("PSP"),
      SpecialRegister::PRIMASK => buffer.push_str("PRIMASK"),
      SpecialRegister::CONTROL => buffer.push_str("CONTROL")
   }
}


//TODO  finish whatever this thing was
/*pub fn has_symbol_operation(operands: &Operands)->bool{
   match operands{
      Operands::LDR_Imm8(_, base, offset)=>{
         base.0 == PROGRAM_COUNTER
      },
      Operand::
   }
}*/

fn dbg_print(operands: &Operands)->String{
   println!("{:?}\n\n", operands);
   match operands{
      Operands::LoadableList(base_register,register_list) => {
         let registers = get_set_bits(*register_list);
         let mut list_str = String::new();
         if (1 << base_register.0) & register_list > 0 {
            list_str.push_str(&format!("r{},",base_register.0));
         }else{
            list_str.push_str(&format!("r{}!,",base_register.0));
         }
         list_str.push_str(&fmt_register_list(registers));
         list_str
      },
      Operands::LDR_Imm5(dest,base ,imm5) => {
         format!("{},[{},{}]",dest,base,imm5)
      },
      Operands::LDR_Imm8(dest,src, imm8) => {
         format!("{},[{},{}]",dest,src,imm8)
      },
      Operands::LDR_REG(dest,src,offset) => {
         format!("{},[{},{}]",dest,src,offset)
      },
      Operands::RegisterList(list) => {
         let registers = get_set_bits(*list);
         fmt_register_list(registers)
      },
      Operands::Primask(flag) => if *flag {String::from("CPSID i")} else{String::from("CPSIE i")},
      _ => {
         let dbg_operands = format!("{:?}",operands);
         remove_everything_outside_brackets(&dbg_operands)
      }
   }

}

fn fmt_register_list(registers: Vec<u8>)->String{
   let list = registers.iter()
      .map(|n| format!("{}",Register::from(*n)))
      .reduce(|acc,i| acc + "," + &i)
      .unwrap();

   let mut fin = String::new();
   fin.push('{');
   fin.push_str(&list);
   fin.push('}');
   fin
}

fn serialise_register_list(buffer: &mut String, registers: Vec<u8>){
   buffer.push('{');
   for r in 0 .. (registers.len() - 1){
      serialise_register(buffer,registers[r]);
      buffer.push(',');
   }
   serialise_register(buffer,registers[registers.len() -1]);
   buffer.push('}');
}

fn remove_everything_outside_brackets(text: &str)->String{
   let mut in_brackets = false;
   let mut new_str = String::new();
   for ch in text.chars(){
      if ch == '(' || ch == ')'{
         in_brackets = !in_brackets;
         continue;
      }
      if in_brackets{
         new_str.push(ch);
      }
   }
   new_str
}

pub fn get_operands(code: &Opcode, hw: HalfWord)-> Option<Operands>{
   match code{
      Opcode::_16Bit(opcode) => {
         match opcode{
            B16::ADCS => Some(get_def_reg_pair_as_operands(hw)),
            B16::ADD_Imm3 =>  Some(get_def_reg_pair_with_imm3(hw)),
            B16::ADD_Imm8 => Some(get_dest_and_imm8(hw)),
            B16::ADDS_REG => Some(get_9b_register_triplet(hw)),
            B16::ADDS_REG_T2 => Some(get_add_reg_t2_operands(hw)),
            B16::ADD_REG_SP_IMM8 => Some(get_add_reg_sp_imm8_operands(hw)),
            B16::INCR_SP_BY_IMM7 => Some(get_add_reg_sp_imm7_operands(hw)),
            B16::INCR_SP_BY_REG => Some(get_incr_sp_by_reg_operands(hw)),
            B16::ADR => Some(get_adr_operands(hw)),
            B16::ANDS => Some(get_def_reg_pair_as_operands(hw)),
            B16::ASRS_Imm5 => Some(get_asr_imm5_operands(hw)),
            B16::ASRS_REG => Some(get_def_reg_pair_as_operands(hw)),
            B16::BEQ => Some(get_cond_branch_operands(hw)), 
            B16::BNEQ => Some(get_cond_branch_operands(hw)), 
            B16::B_CARRY_IS_SET => Some(get_cond_branch_operands(hw)), 
            B16::B_CARRY_IS_CLEAR => Some(get_cond_branch_operands(hw)), 
            B16::B_IF_NEGATIVE => Some(get_cond_branch_operands(hw)), 
            B16::B_IF_POSITIVE => Some(get_cond_branch_operands(hw)), 
            B16::B_IF_OVERFLOW => Some(get_cond_branch_operands(hw)), 
            B16::B_IF_NO_OVERFLOW => Some(get_cond_branch_operands(hw)), 
            B16::B_UNSIGNED_HIGHER => Some(get_cond_branch_operands(hw)), 
            B16::B_UNSIGNED_LOWER_OR_SAME => Some(get_cond_branch_operands(hw)), 
            B16::B_GTE => Some(get_cond_branch_operands(hw)), 
            B16::B_LTE => Some(get_cond_branch_operands(hw)), 
            B16::B_GT => Some(get_cond_branch_operands(hw)), 
            B16::B_LT => Some(get_cond_branch_operands(hw)), 
            B16::B_ALWAYS => Some(get_uncond_branch_operands(hw)), 
            B16::BIT_CLEAR_REGISTER => Some(get_def_reg_pair_as_operands(hw)),
            B16::BREAKPOINT => Some(get_breakpoint_operands(hw)),
            B16::BR_LNK_EXCHANGE => Some(get_br_lnk_exchange_operands(hw)),
            B16::BR_EXCHANGE => Some(get_br_exchange_operands(hw)),
            B16::CMP_NEG_REG => Some(get_pure_reg_pair(hw)),
            B16::CMP_Imm8 => Some(get_cmp_imm8_operands(hw)),
            B16::CMP_REG_T1 => Some(get_cmp_reg_operands::<3>(hw)),
            B16::CMP_REG_T2 => Some(get_cmp_reg_operands::<4>(hw)),
            B16::CPS => Some(get_cps_operands(hw)),
            B16::XOR_REG => Some(get_def_reg_pair_as_operands(hw)),
            B16::LDM => Some(get_load_list_operands(hw)),
            B16::LDR_Imm5 => Some(get_ldr_imm5_operands(hw,4)),
            B16::LDR_SP_Imm8 => Some(get_ldr_imm8_operands(hw)),
            B16::LDR_PC_Imm8 => Some(get_ldr_imm8_operands(hw)),
            B16::LDR_REGS => Some(get_ldr_reg_operands(hw)),
            B16::LDRB_Imm5 => Some(get_ldr_imm5_operands(hw,1)),
            B16::LDRB_REGS => Some(get_ldr_reg_operands(hw)),
            B16::LDRH_Imm5 => Some(get_ldr_imm5_operands(hw,2)),
            B16::LDRH_REGS => Some(get_ldr_reg_operands(hw)),
            B16::LDRSB_REGS => Some(get_ldr_reg_operands(hw)),
            B16::LDRSH_REGS => Some(get_ldr_reg_operands(hw)),
            B16::LSL_Imm5 => Some(get_ls_imm5_operands(hw)),
            B16::LSL_REGS => Some(get_def_reg_pair_as_operands(hw)),
            B16::LSR_Imm5 => Some(get_ls_imm5_operands(hw)),
            B16::LSR_REGS => Some(get_def_reg_pair_as_operands(hw)),
            B16::MOV_Imm8 => Some(get_dest_and_imm8(hw)),
            B16::MOV_REGS_T1 => Some(get_mov_reg_operands::<4>(hw)),
            B16::MOV_REGS_T2 => Some(get_mov_reg_operands::<3>(hw)),
            B16::MUL => Some(get_def_reg_pair_as_operands(hw)),
            B16::MVN => Some(get_mov_reg_operands::<3>(hw)),
            B16::NOP => None,
            B16::POP => Some(get_pop_operands(hw)),
            B16::PUSH => Some(get_push_operands(hw)),
            B16::ORR => Some(get_def_reg_pair_as_operands(hw)),
            B16::REV => Some(get_def_reg_pair_as_operands(hw)),
            B16::REV_16 => Some(get_def_reg_pair_as_operands(hw)),
            B16::REVSH => Some(get_def_reg_pair_as_operands(hw)),
            B16::ROR => Some(get_def_reg_pair_as_operands(hw)),
            B16::RSB => Some(get_def_reg_pair_as_operands(hw)),
            B16::SBC => Some(get_def_reg_pair_as_operands(hw)),
            B16::SEV => None,
            B16::STM => Some(get_load_list_operands(hw)),
            B16::STR_Imm5 => Some(get_str_imm5_operands(hw,2)),
            B16::STR_Imm8 => Some(get_str_imm8_operands(hw,2)),
            B16::STR_REG => Some(get_str_reg_operands(hw)),
            B16::STRB_Imm5 => Some(get_str_imm5_operands(hw,0)),
            B16::STRB_REG => Some(get_str_reg_operands(hw)),
            B16::STRH_Imm5 => Some(get_str_imm5_operands(hw,1)),
            B16::STRH_REG => Some(get_str_reg_operands(hw)),
            B16::SUB_Imm3 => Some(get_def_reg_pair_with_imm3(hw)),
            B16::SUB_Imm8 => Some(get_dest_and_imm8(hw)),
            B16::SUB_REG => Some(get_9b_register_triplet(hw)),
            B16::SUB_SP_Imm7 => Some(get_sub_sp_operands(hw)),
            B16::SVC => Some(get_low_byte(hw)),
            B16::SXTB => Some(get_def_reg_pair_as_operands(hw)),
            B16::SXTH => Some(get_def_reg_pair_as_operands(hw)),
            B16::TST => Some(get_pure_reg_pair(hw)),
            B16::UNDEFINED => Some(get_low_byte(hw)),
            B16::UXTB => Some(get_def_reg_pair_as_operands(hw)),
            B16::UXTH => Some(get_def_reg_pair_as_operands(hw)),
            B16::WFE => None,
            B16::WFI => None,
            B16::YIELD => None,
         }
      }
      Opcode::_32Bit(_) => panic!("cannot parse 16b operands from 32b instruction {:?}",code)
   }
}

pub fn get_operands_32b(code: &Opcode, bytes: Word)->Option<Operands>{
   match code {
      Opcode::_16Bit(_) => {
         panic!("16b {:?} operands shouldn't be parsed from 32b input",code)
      },
      Opcode::_32Bit(instruction) => {
         match instruction{
            B32::BR_AND_LNK => Some(get_branch_and_lnk_operands(bytes)),
            B32::MRS => Some(get_mrs_operands(bytes)),
            B32::MSR => Some(get_msr_operands(bytes)),
            B32::DSB => Some(get_barrier_option(bytes)),
            B32::ISB => Some(get_barrier_option(bytes)),
            B32::DMB => Some(get_barrier_option(bytes)),
            B32::UNDEFINED => Some(get_undefined_32b(bytes)),
         }
      }
   }
}

fn get_def_reg_pair(hw: HalfWord)->(DestRegister,Register){
   let (dest,other) = get_def_reg_pair_u8(hw);
   (dest.into(),other.into())
}

fn get_def_reg_pair_as_operands(hw: HalfWord)->Operands{
   let (dest,other) = get_def_reg_pair(hw);
   Operands::RegisterPair(dest, other)
}

fn get_pure_reg_pair(hw: HalfWord)->Operands{
   let (dest,other) = get_def_reg_pair_u8(hw);
   Operands::PureRegisterPair(dest.into(),other.into())
}

#[inline]
fn get_def_reg_pair_u8(hw: HalfWord)->(u8,u8){
   let dest: u8 = hw[0] & 0x07;
   let other: u8 = (hw[0] & 0x38) >> 3;
   (dest,other)
}

#[inline]
fn get_def_reg_pair_with_imm3(hw: HalfWord)->Operands{
   let (dest,other) = get_def_reg_pair(hw);
   let native: u16 = from_arm_bytes_16b(hw);
   let imm3: Literal<3> = get_bitfield::<3>(native as u32, 6);
   Operands::RegPairImm3(dest,other.0.into(),imm3)
}

fn get_dest_and_imm8(hw: HalfWord)->Operands{
   let dest: u8 = hw[1] & 0x07;
   let imm8: Literal<8> = (hw[0] as u32).into();
   Operands::DestImm8(dest.into(), imm8)
}

fn get_9b_register_triplet_u8(hw: HalfWord)->(u8,u8,u8){
   let dest = (hw[0] & 0x07).into();
   let src = (hw[0] & 0x38) >> 3;
   let native = from_arm_bytes_16b(hw);
   let second_arg = ((native & 0x01C0) >> 6) as u8;
   (dest,src,second_arg)
}
fn get_9b_register_triplet(hw: HalfWord)->Operands{
   let (dest,src,second_arg) = get_9b_register_triplet_u8(hw);
   Operands::RegisterTriplet(dest.into(),src.into(),second_arg.into())
}

fn get_add_reg_t2_operands(hw: HalfWord)->Operands{
   let opt_dest_bit = (hw[0] & 0x80) >> 4;
   let dest: DestRegister = ((hw[0] & 0x07) | opt_dest_bit).into();
   dbg_ln!("rm=({:#02b} & {:#02b})= {:#02b}",hw[0],0x78,hw[0] & 0x78);
   let r = get_bitfield::<4>(hw[0] as u32,3);
   Operands::RegisterPair(dest,r.0.into())
}

fn get_add_reg_sp_imm8_operands(hw: HalfWord)->Operands{
   let dest: u8 = hw[1] & 0x07;
   Operands::ADD_REG_SP_IMM8(dest.into(),((hw[0] as u32) << 2).into())
}

fn get_add_reg_sp_imm7_operands(hw: HalfWord)->Operands{
   let v = hw[0] & 0x7F;
   Operands::INCR_SP_BY_IMM7(((v as u32) << 2).into())
}


fn get_incr_sp_by_reg_operands(hw: HalfWord)->Operands{
   let dest = get_bitfield::<4>(hw[0] as u32,3);
   Operands::INCR_SP_BY_REG(dest.0.into())
}

fn get_adr_operands(hw: HalfWord)->Operands{
   let dest: u8 = hw[1] & 0x07;
   let literal = (hw[0] as u32) << 2;
   Operands::ADR(dest.into(),(literal).into())
}

fn get_asr_imm5_operands(hw: HalfWord)->Operands{
   let (dest,other) = get_def_reg_pair(hw);
   let native: u16 = from_arm_bytes_16b(hw);
   let literal = get_bitfield::<5>(native as u32,6);
   Operands::ASRS_Imm5(dest,other.0.into(),literal)
}

fn get_cond_branch_operands(hw: HalfWord)->Operands{
   //dbg_ln!("raw: {:#x},{:#x}",hw[0],hw[1]);
   //dbg_ln!("enc: {}_base10 {:#x}",hw[0],hw[0]);
   //dbg_ln!("native: {}, {:#x}",hw[0] as i8,hw[0] as i8);
   let shifted: Literal<9> = ((((hw[0]) as u32) << 1)).into();
   //dbg_ln!("shifted: {}, {:#x}",shifted,shifted.0);
   if shifted.0 & 0x100 > 0 {
      dbg_ln!("is signed");
   }else {
      dbg_ln!("is unsigned");
   }
   let extended = sign_extend(shifted) + 4;
   dbg_ln!("ext: {}",extended);
   debug_assert!(extended >= -256);
   debug_assert!(extended <= 254);
   debug_assert_eq!(extended.abs() % 2,0);
   Operands::COND_BRANCH(extended)
}

fn get_uncond_branch_operands(hw: HalfWord)->Operands{
   let native: u16 = from_arm_bytes_16b(hw);
   let label: Literal<11> = ((native & 0x07FF) as u32).into();
   let adjusted: Literal<12> = (label.0 << 1).into();
   let literal: i32 = sign_extend(adjusted) + 4;
   debug_assert!(literal >= -2048);
   debug_assert!(literal <= 2046);
   debug_assert_eq!(literal.abs() % 2,0);
   Operands::B_ALWAYS(literal)
}

fn get_breakpoint_operands(hw: HalfWord)->Operands{
   let imm8: Literal<8> = hw[0].into();
   Operands::BREAKPOINT(imm8)
}

fn get_branch_and_lnk_operands(bytes: Word)->Operands{
   //dbg_ln!("instr: [{:#x},{:#x},{:#x},{:#x}]",bytes[0],bytes[1],bytes[2],bytes[3]);
   let left_hw: [u8;2] = [bytes[0],bytes[1]];
   let native_l: u16 = from_arm_bytes_16b(left_hw);

   let right_hw: [u8;2] = [bytes[2],bytes[3]];
   let native_r: u16 = from_arm_bytes_16b(right_hw);
   //dbg_ln!("native bin: {:#x},{:#x}",native_l,native_r);
   let imm10: u32 = (native_l & 0x03FF) as u32;
   let sign_bit: u32 = ((native_l & 0x0400) >> 10)as u32;

   let imm11: u32 = (native_r & 0x07FF) as u32;
   let j1 = ((native_r & 0x2000) >> 13) as u32;
   let j2 = ((native_r & 0x0800) >> 11) as u32;

   let i1: u32 = !(j1 ^ sign_bit) & 0x1;
   let i2: u32 = !(j2 ^ sign_bit) & 0x1;
   //dbg_ln!("j1={}, j2={}, i1={}, i2={} s={}",j1,j2,i1,i2,sign_bit);
   //dbg_ln!("s:{:x}\ni1:{:x}\n:i2:{:x}\nimm10:{:x}\nimm11:{:x}\n",sign_bit,i1,i2,imm10,imm11);
   //dbg_ln!("1mm10:imm11:0 = {:x}",(imm11<<1) | (imm10<<12));
   //dbg_ln!("i1:i2:1mm10:imm11:0 = {:x}",(imm11<<1) | (imm10<<12) | (i2<<22)| (i1<<23));
   //dbg_ln!("s:i1:i2:1mm10:imm11:0 = {:x}",(imm11<<1) | (imm10<<12) | (i2<<22)| (i1<<23) | (sign_bit) << 24);
   let u_total: u32 = (imm11 << 1) | (imm10 << 12) | (i2 << 22) | (i1 << 23) | (sign_bit << 24);
   let sign_extended: u32 = if sign_bit > 0 {
      0xFE000000_u32 | u_total
   }else{
      u_total
   };

   let result = sign_extended as i32 + 4_i32;
   debug_assert_eq!(result % 2,0);
   debug_assert!(result >= -16777216," within limit specified in ARMv6 ISA");
   debug_assert!(result <= 16777214," within limit specified in ARMv6 ISA");
   Operands::BR_LNK(result)
}

fn get_br_lnk_exchange_operands(hw: HalfWord)->Operands{
   let register: Register = ((hw[0] & 0x078) >> 3).into();
   Operands::BR_LNK_EXCHANGE(register)
}

fn get_br_exchange_operands(hw: HalfWord)->Operands{
   let register: Register = ((hw[0] & 0x078) >> 3).into();
   Operands::BR_EXCHANGE(register)
}

fn get_cmp_imm8_operands(hw: HalfWord)->Operands{
   let (register,imm8) = offset_addressing_imm8(hw);
   Operands::CMP_Imm8(register.into(),imm8)
}

fn get_cmp_reg_operands<const L:  u32>(hw: HalfWord)->Operands{
   let first: Register = (hw[0] & 0x07).into();
   let second: Register = get_bitfield::<L>(hw[0] as u32,3).0.into();
   Operands::PureRegisterPair(first, second)
}

fn get_load_list_operands(hw: HalfWord)->Operands{
   let list = hw[0] as u16; 
   let reg: Register = (hw[1] & 0x07).into();
   Operands::LoadableList(reg, list)
}

fn offset_addressing_imm5(hw: HalfWord)->(u8,u8,Literal<5>){
   let (dest,base) = get_def_reg_pair_u8(hw);
   let native = from_arm_bytes_16b(hw);
   let imm5: Literal<5> = get_bitfield::<5>(native as u32,6);
   (dest,base.into(),imm5)
}

fn offset_addressing_imm8(hw: HalfWord)->(u8,Literal<8>){
   let dest = (hw[1] & 0x07).into();
   let imm8: Literal<8> = hw[0].into();
   (dest,imm8)
}

fn offset_addressing_regs(hw: HalfWord)->(u8,Register,Register){
   let (flex,base,offset) = get_9b_register_triplet_u8(hw);
   (flex,base.into(),offset.into())
}

fn get_dest_src_and_imm5(hw: HalfWord)->(DestRegister,SrcRegister,Literal<5>){
   let (dest,base,imm5) = offset_addressing_imm5(hw);
   (dest.into(),base.into(),imm5)
}


fn get_ldr_imm5_operands(hw: HalfWord, multiple: u8)->Operands{
   let (dest,base,imm5) = get_dest_src_and_imm5(hw);
   let adjusted: Literal<5> = (imm5.0 * multiple as u32).into();
   Operands::LDR_Imm5(dest,base,adjusted)
}

fn get_ldr_imm8_operands(hw: HalfWord)->Operands{
   let (dest,imm8) = offset_addressing_imm8(hw);
   let src = if hw[1] & 0xF8 == 0x98{
      SrcRegister(STACK_POINTER)
   }else{
      SrcRegister(PROGRAM_COUNTER)
   };
   let adjusted: Literal<8> = (imm8.0 * 4 as u32).into();
   Operands::LDR_Imm8(dest.into(), src, adjusted)
}

fn get_ldr_reg_operands(hw: HalfWord)->Operands{
   let (dest,base_reg,offset_reg) = offset_addressing_regs(hw);
   Operands::LDR_REG(dest.into(),base_reg.0.into(),offset_reg)
}

fn get_ls_imm5_operands(hw: HalfWord)->Operands{
   let (dest,src,offset) = get_dest_src_and_imm5(hw);
   Operands::LS_Imm5(dest,src,offset)
}

fn get_mov_reg_operands<const L: u32>(hw: HalfWord)->Operands{
   let dest: DestRegister = if L == 4{
      //dbg_ln!("using extra byte");
      let d = ((hw[0] & 0x80) >> 4) | (hw[0] & 0x07);
      d.into()
   }else{
      (hw[0] & 0x07).into()
   };
   
   let src: SrcRegister = get_bitfield::<L>(hw[0] as u32,3).0.into();
   Operands::MOV_REG(dest,src)
}

fn get_pop_operands(hw: HalfWord)->Operands{
   let pc_bit = (hw[1] & 0x01) as u16;
   let list = hw[0] as u16 | (pc_bit << 15);
   Operands::RegisterList(list)
}

fn get_push_operands(hw: HalfWord)->Operands{
   let lr_bit = (hw[1] & 0x01) as u16;
   let list = hw[0] as u16 | (lr_bit << 14);
   Operands::RegisterList(list)
}

fn get_str_imm5_operands(hw: HalfWord,shift: u8)->Operands{
   let (dest,base,imm5) = offset_addressing_imm5(hw);
   let adjusted: Literal<5> = (imm5.0 << shift).into(); 
   let src: SrcRegister = dest.into();
   let base_reg: Register = base.into();
   Operands::STR_Imm5(src,base_reg,adjusted)
}

fn get_str_imm8_operands(hw: HalfWord, shift: u8)-> Operands{
   let (src,offset) = offset_addressing_imm8(hw);
   let adjusted: Literal<8> = (offset.0 << shift).into();
   Operands::STR_Imm8(src.into(), adjusted)
}

fn get_str_reg_operands(hw: HalfWord)-> Operands{
   let (src,base,offset) = offset_addressing_regs(hw);
   Operands::STR_REG(src.into(),base,offset)
}

fn get_sub_sp_operands(hw: HalfWord)->Operands{
   let literal: Literal<7> = (((hw[0] & 0x7F) as u32) << 2).into();
   debug_assert!(literal.0 <=508);
   Operands::SP_SUB(literal)
}

fn get_low_byte(hw: HalfWord)->Operands{
   Operands::Byte(hw[0].into())
}

fn get_cps_operands(hw: HalfWord)->Operands{
   let flat = (hw[0] & 0x10) > 0;
   Operands::Primask(flat)
}

fn get_msr_operands(bytes: Word)->Operands{
   Operands::MSR(get_special_register(bytes[2]),(bytes[0] & 0x0F).into())
}

fn get_mrs_operands(bytes: Word)->Operands{
   Operands::MRS((bytes[3] & 0x0F).into(),get_special_register(bytes[2]))
}

fn get_barrier_option(bytes: Word)->Operands{
   Operands::Nibble((bytes[2] & 0x0F).into())
}

fn get_undefined_32b(bytes: Word)->Operands{
   let imm4: u16 = (bytes[0] & 0x0F).into();
   let later_u16 = from_arm_bytes_16b([bytes[2],bytes[3]]);
   let imm12: u16 = later_u16 & 0x0FFF;
   Operands::HalfWord(((imm4 << 12 | imm12) as u32).into())
}

fn get_special_register(byte: u8)->SpecialRegister{
   match byte{
      0 => SpecialRegister::APSR,
      1 => SpecialRegister::IAPSR,
      2 => SpecialRegister::EAPSR,
      3 => SpecialRegister::XPSR,
      5 => SpecialRegister::IPSR,
      6 => SpecialRegister::EPSR,
      7 => SpecialRegister::IEPSR,
      8 => SpecialRegister::MSP,
      9 => SpecialRegister::PSP,
      16 => SpecialRegister::PRIMASK,
      20 => SpecialRegister::CONTROL,
      _ => unreachable!()
   }
}
