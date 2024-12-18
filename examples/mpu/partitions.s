.text
.thumb
.equ _STACK_SIZE,0X1000

   _VECTOR_TABLE:
      .4byte _SP_RESET_VAL
      .4byte _reset_handler
      .4byte _dummy_handler
      .4byte _hardfault_handler

   .thumb_func
   .global _reset_handler
   _reset_handler:
      MOV r0, #0
      MOV r1, #0
      MOV r2, #0
      MOV r3, #0
      B .program_region_0
   
   .thumb_func
   _hardfault_handler: 
      BKPT #0 

   .thumb_func
   _dummy_handler: 
      BX LR

   .program_region_0: 
      MOV r1, #0
      LDR r0, _MPU_RNR 
      STR r1, [r0]

      LDR r0, _MPU_RASR
      LDR r1, _ALL_ACCESS
      STR r1, [r0]

      MOV r1, #0
      LDR r0, _MPU_RBAR
      STR r1, [r0]

   .program_region_1:
      MOV r1, #1
      LDR r0, _MPU_RNR 
      STR r1, [r0]
      
      LDR r0, _MPU_RASR
      LDR r1, _PRIV_ONLY
      STR r1, [r0]

      LDR r1, =.very_important_data
      MOV r2, #8
      LSL r1, r2
      LDR r0, _MPU_RBAR
      STR r1, [r0]


   .init_mpu:
      LDR r0, _MPU_CTRL
      MOV r1, #5  
      STR r1, [r0]


   //unprivileged writes to .very_important_data -> very_important_data + 256
   //should cause a hardfault

   .downgrade_thread:
      MOV r3, #1
      MSR CONTROL, r3 

   .do_illegal_access:
      MOV r1, #255
      LDR r0, =.very_important_data
      STR r1, [r0]
      NOP

.macro MPU_OFFSET offset
   .4byte 0xE000ED90 + \offset
.endm

.align 2
_MPU_CTRL: 
   MPU_OFFSET offset=0x4

_MPU_RNR:
   MPU_OFFSET offset=0x8

_MPU_RASR:
   MPU_OFFSET offset=0x10

_MPU_RBAR:
   MPU_OFFSET offset=0xC

_PRIV_ONLY:
   @  XN    |PV:RW, UP:NA| normal memory  | size=256 | enable=1 
   @ 1 << 28,   1 << 24   1 << 18 1 << 17    7 << 1       1
   .4byte 0x1106000F

_ALL_ACCESS:
   @ PV:RW, UP:RW | normal memory | size=2^13 bytes | enable = 1
   @  0x3 << 24        3 << 17       12 << 1            1
   .4byte 0x3060019

.align 8 @ symbol to 2^8 
.very_important_data:
   .4byte 0xDEADBEEF
   .4byte 74771
   .4byte 1
   .4byte 2
   .4byte 3
.align 8 
   stack_end:
   .equ _SP_RESET_VAL, stack_end + _STACK_SIZE
