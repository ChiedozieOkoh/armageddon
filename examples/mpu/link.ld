ENTRY(_reset_handler);
SECTIONS{
   . = 0x0;
   .text : {*(.text)}
}
