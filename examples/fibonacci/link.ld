ENTRY(_main);
SECTIONS{
   . = 0x0;
   .text : {*(.text)}
   .data : {*(.data)}
}
