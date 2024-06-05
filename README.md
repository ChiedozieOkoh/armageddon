# Building The Project
[I recommend using rustup to install cargo](https://doc.rust-lang.org/book/ch01-01-installation.html)

Clone the repository and run `cargo b --release`\ 
The release binary produced will be in  `<repository root>/target/release/armageddon` \ 

NOTE: the program depends the the iced ui library which is fairly new, I have only tested it 
on linux using X11 so I am not sure if its compatible with other operating systems. 

# Launching From The Command Line
You need to provide to simulator with a path to an ELF file.\
You can run the simulator with  `armageddon <path_to_elf>`

# Notes On ELF Compatability
Currently the simulator can load all loadable sections of an ELF.\
Only the `.text` section will be disassembled.\
The simulator has only been tested with ELFs produced by the gnu arm-none-eabi toolchain.

# Using The Simulator

## The Disassembly View
You can open the disassembly view by clicking on the `disassembly` button. \
the disassembly view allows you to see the code in the `.text` section of the loaded program. \
The instruction pointed to by the `PC` register is always displayed in bold. \
You can add/remove breakpoints at a specific instruction by clicking on the line in the diassembly window. 

## The Register View
You can open the register view by clicking on the `registers` button. \
The registers view shows the state of a number of general purpose registers. \
You can change the registers `R0-R12` to display their value in either decimal or hex
by clicking on them.
The register view is updated whenever you manually step the simulator or a halt occurs.

## The Memory View
You can open the memory view by clicking on the `memory` button. \
The memory view allows you to inspect the memory values of the simulator within a certain memory range. \
The display of the memory view is updated whenever you manually step the simulator or a halt occurs.

## Shortcuts
`Alt + Enter` : centres the disassembly around the instruction pointed to by the `PC` register. \
`Ctrl + f` : search the disassembly for a string. \
`Ctrl + d` : remove all breakpoints. \
`backspace` : press backspace to close the focused window. 

## Start Up Routine
### Reset boot mode
By default the simulator will execute a system reset on boot and hand off 
execution to the reset handler in the vector table.\ 
The vector table will be also be used to initialise the `MSP` register.\ 
If you do not want to execute a reset on boot then pass the `--manual-boot` flag to skip this procedure. 

### Manual boot mode
Use the `--manual-boot` flag to enable manual boot mode. 
In manual boot mode the simulator will simply initialise the `PC` register to point to the `entrypoint` of the ELF file.\ 
You can override the `entrypoint` value being used with the `--entrypoint=<HEX>` flag.\ 
Note manual boot mode will leave the `MSP` and the `PSP` registers uninitialised.\
You can specify the reset value of the `MSP` by using the `--sp-reset-val=<HEX>` flag  \
If you are running the simulator in manual-boot mode you should press the `Reset` button on the GUI to initialise
the `SP`. 

## Reset behaviour
For a reset to function properly you should have a reset handler which is pointed to by the vector table. \
Pressing the reset button will begin the reset routine, at the end of the reset routine the simulator will \
have the `PC` register pointing to the reset handler. From that point onwards you have to manually start execution.
You can trigger a reset at anytime, even during program execution.

## Shutdown
The simulator will continue running until it encounters an error. \
Use the halt button to stop execution at anytime. \

## Using Exceptions
The vector table offset by default is 0. but can be configured with the `--vtor=<HEX>` flag.\
Exceptions may be triggered by runtime errors, its recommended that you atleast include \
a HardFault handler and a Reset handler in your binary.  E.G \

code.s
```
   .text
   .thumb
   .equ _STACK_SIZE,0x80
   .global _main
   _vector_table:
        .4byte _SP_RESET_VAL
        .4byte _reset_handler
        .4byte _dummy_handler
        .4byte _hardfault_handler // this points to the HardFault handler
    
    .thumb_func
    _dummy_handler:
        B .
        
    .thumb_func
    _hardfault_handler:
        B .

    .thumb_func
    _reset_handler:
        MOV r0,#0
        MOV r1,#0
        MOV r2,#0
        MOV r3,#0
        B _main
        
        
    _main:
     <main program goes here>
    .pool //make sure the pool occurs before _STACK_START label
    .align 3
    _STACK_START:

    .equ _SP_RESET_VAL, _STACK_START + _STACK_SIZE
```

link.ld
```
ENTRY(_main);
SECTIONS{
    . = 0x0;
    .text : {*(.text)}
}
```
* assemble the binary `arm-none-eabi-as code.s -march=armv6s-m -o code.elf`
* link the binary `arm-none-eabi-ld code.elf -T link.ld -o code.o`
* finally simulate the linked executable `armageddon code.o` 

## Memory Usage
The simulator supports the full 4GB of address space and uses the default armv6-m address map.

## Notes On compatability with ARMv6-M ISA 
The memory mapped registers of the system control space (SCS) are partially implemented.\
You can use the ICSR to trigger NMI and PendSV interrupts. 
The SHPR2 and SHPR3 registers can be used to change the priority of SVCall, SysTick and PendSV. \
The NVIC is supported. \ 
There is no instruction pipeline in the simulator thus all reads and writes are committed instantly. \
The vector table offset is not configurable at runtime. \

# Feedback
If you encounter any bugs please open an issue :)

# TODO List
- [x] Add Halt and Reset button to UI
- [x] Support NMI through memory mapped SCS
- [x] Allow breakpoints to be added by clicking on a line of text in the disassembly
- [x] Add an option to  execute the reset handler as part of the start up routine
- [x] Allow Search Function to also search symbol names
- [x] fix bug where search results dont show if the result is present on the IR line
- [ ] add a line limit to the execution logs
- [x] support focus on code search results
- [ ] add command line option to force a section to be included in the disassembly
- [x] add option to allow to do reset without an explicit reset handler (i.e just jump to `entry_point`) 
