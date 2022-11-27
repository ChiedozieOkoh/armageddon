mod asm;
mod elf;

#[cfg(test)]
mod tests;

use asm::get_bit;

use crate::asm::clear_bit;
fn main() {
    println!("{:04b}",3);
    assert_eq!(get_bit(1,3),1);
    assert_eq!(get_bit(0,3),1);
    assert_eq!(get_bit(2,3),0);

    println!("{:04b}",10);
    assert_eq!(clear_bit(1, 10),8);
}


