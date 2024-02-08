# Building The Project
Clone the repository and run `cargo b --release`\
The release binary produced will be in  `<repository root>/target/release/armageddon`

# Launching From The Command Line
You need to provide to simulator with a path to an ELF file to start running.\
You can run the simulator with  `armageddon <path_to_elf>`

# Notes On ELF Compatability
Currently the Simulator only loads the `.text` section of the ELF.\
The simulator has only been tested with ELFs produced by the gnu arm-none-eabi toolchain.

# Using The Simulator
## Start Up Routine
The simulator will start with the `PC` register pointing to the entry point of the ELF file. \
You must manually start executing using either the `step` or `continue` button. \
Initially all registers except the `PC` register will be set to `0` but you should not rely on this behaviour for \
program correctness as real hardware does not provide this guaruntee.

## Reset behaviour
For a reset to function properly you should have a reset handler which is pointed to by the vector table. \
Pressing the reset button will begin the reset routine, at the end of the reset routine the simulator will \
have the `PC` register pointing to the reset handler. From that point onwards you have to manually start execution.
You can trigger a reset at anytime, even during program execution.

## Shutdown
The simulator will continue running until it encounters an error. \
Use the halt button to stop execution at anytime. \
Use the close button to close the simulator.

## Using Exceptions
The vector table offset is always 0 and is not configurable. Therefore if you want exceptions to be 
simulated properly you should make sure the vector table is at the beginning of the binary e.g

code.s
```
   .text
   .thumb
   .equ _STACK_SIZE,0x80
   .global _main
   _vector_table:
        .4byte _SP_RESET_VAL,
        .4byte _reset_handler,
        .4byte _dummy_handler,
    
    .thumb_func
    _dummy_handler:
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
    _STACK_START:
        .align 3
        .fill _STACK_SIZE,1,0
    _SP_RESET_VAL:
        .size _SP_RESET_VAL, . - _STACK_START
```

link.ld
```
ENTRY(_main);
SECTIONS{
    .= 0x0;
    .text : {*(.text)}
}
```
* assemble the binary `arm-none-eabi-as code.s -march=armv6-m -o code.elf`
* link the binary `arm-none-eabi-ld code.elf -T link.ld -o code.o`
* finally simulate the linked executable `armageddon code.o` 

## Notes On compatability with ARMv6-M ISA 
The simulator is still in development so not all instructions are implemented yet.\
Though ARMv6-M is a memory mapped architecture most of the memory mapped system configuration
registers have not been implemented yet, its a work in progress. \
Exceptions are functional the vector table offset is always 0 and is not configurable. \

# Feedback
If you encounter any bugs please open an issue :)

# TODO List
- [x] Add `check_exception()` logic to simulation on UI thread
- [X] Add Halt and Reset button to UI
- [ ] Add Memory Mapped SCS
- [ ] Support NMI through memory mapped SCS
- [ ] Allow breakpoints to be added by clicking on a line of text in the disassembly

