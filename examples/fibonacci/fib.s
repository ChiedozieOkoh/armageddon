.text
.thumb
.equ _STACK_SIZE,0x1000

.macro zero_fill from, total
   .4byte 0
   .if \total-\from-1 
      zero_fill "(\from+1)",\total
   .endif
.endm

_vector_table:
   .4byte _SP_RESET_VAL 
   .4byte _reset_handler

.thumb_func 
_reset_handler:
   B _main

.global _main
_main:
   MOV r0,#10
   MOV r1,#0
   MOV r2,#1
   LDR r4,=_sequence_arr
   STM r4!, {r1,r2}
   PUSH {r0,r1,r2}
   BL _fibonacci
   B _start_cleanup
   _fibonacci:
      POP {r0,r1,r2}
      ADD r3,r1,r2
      STR r3,[r4,#0]
      CMP r0,#1
      BGT _repeat
      BX LR
      _repeat:
         SUB r0,#1
         ADD r4,#4
         PUSH {r0,r2,r3}
         B _fibonacci

   _start_cleanup:
      MOV r5, #0
      MOV r6, #0
      MOV r7, #11
      LDR r4,=_sequence_arr
      STM r4!, {r5,r6}
      _cleanup:
         STM r4!, {r5}
         SUB r7,#1
         BNE _cleanup
      _done:
         BKPT 0
.data
   .align 2
   _sequence_arr:
      zero_fill 0,12
   .align 3
   stack_end:
   .equ _SP_RESET_VAL, stack_end + _STACK_SIZE
