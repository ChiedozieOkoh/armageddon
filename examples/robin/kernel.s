.text
.thumb
.equ _STACK_SIZE,0x1000
.set TCB_HDR_NO_ACTIVE_TASK,0xFF

.macro _TASK_BLOCK_HEADER
   .4byte 0 //number of tasks
   .4byte TCB_HDR_NO_ACTIVE_TASK //active task
.endm

.macro _TASK_STRUCT_BLOCK
   .4byte 0 //used flag
   .4byte 0 //r0
   .4byte 0 //r1
   .4byte 0 //r2
   .4byte 0 //r3
   .4byte 0 //r12
   .4byte 0 //LR
   .4byte 0 //PC
   .4byte 0 //XPSR
.endm

_vector_table:
   .4byte _SP_RESET_VAL
   .4byte _reset_handler
   .4byte _dummy_handler //NMI
   .4byte _dummy_handler //HardFault
   .4byte _dummy_handler
   .4byte _dummy_handler
   .4byte _dummy_handler
   .4byte _dummy_handler
   .4byte _dummy_handler
   .4byte _dummy_handler
   .4byte _dummy_handler
   .4byte _svc_handler
   .4byte _dummy_handler
   .4byte _dummy_handler
   .4byte _pendsv_handler
   .4byte _systick_handler


.global _reset_handler
.thumb_func
_reset_handler:
   MOV r0,#0
   MOV r1,#0
   MOV r2,#0
   MOV r3,#0
   .zero_tcb_hdr_struct:
      LDR r1,=_tcb_hdr_struct
      STR r0,[r1, #0]

   //TODO use SVC to init_clock so that we can enable systicks after _main has registers the tasks.
   //TODO make timer only increment clock_tick(&sys) if sys.mode == THREAD
   .set_up_psp: 
      LDR r4, =_stack_size
      LDR r1,[r4,#0]
      
      .get_sp_from_vector_table:
         LDR r2,=_vector_table
         LDR r0,[r2, #0]
         ADD r0,r1

      MSR PSP,r0

   .descalate_threads: //has to run last becuase we need elevated privelages to do system configuration
      MOV r4,#3
      MSR CONTROL,r4

   B  _main

.thumb_func
_dummy_handler:
   B .

.thumb_func
_svc_handler:
   .set EXC_RETURN_TO_THREAD_PSP, 0xFFFFFFFD
   LDR r1,=EXC_RETURN_TO_THREAD_PSP 
   CMP r1,LR
   BEQ .get_svc_arg
   .panic_wrong_stack:
      BKPT 0
   .get_svc_arg: // assume the SVC arg was passed in as r0
      MRS r0, PSP 
      LDR r1, [r0,#0]
      LDR r2, [r0,#4]
      PUSH {r4,r5,r6,r7}
      PUSH {r2}
      CMP r1,#42
      BEQ .add_new_task
      CMP r1,#69
      BEQ .remove_active_task
      CMP r1,#11
      BEQ .set_up_timer
      .panic_unrecognised_svc_arg:
         BKPT 0
      .add_new_task:
         LDR r1, =_tcb_hdr_struct
         LDR r0, [r1, #0]
         ADD r0, #1
         .save_task_count: 
            STR r0, [r1, #0]
         .init_next_task:
            ADD r1, #8
            MOV r5,r1
            ADD r5,#72
            .check_next_tcb:
               LDR r2,[r1]
               CMP r2,#0
               BEQ .init_tcb_vec_element
               ADD r1,#36
               CMP r1,r5
               BNE .check_next_tcb
               .panic_no_free_taskblocks:
                  BKPT 0
            
            .init_tcb_vec_element:
               MOV r2, #1
               STR r2, [r1] //is_used flag
               MOV r0,#0 //r0
               MOV r2,#0 //r1
               MOV r3,#0 //r2
               MOV r4,#0 //r3
               MOV r5,#0 //r12
               MOV r6,#0 //LR //maybe put a magic lr here to hold a task
               POP {r7}  //pc
               ADD r1,#4
               STM r1!, {r0,r2-r7}
               STR r0, [r1,#0] //xpsr
               POP {r4-r7}
               BX LR

      .remove_active_task: 
         BKPT 0

      .set_up_timer:
         .set SYST_RVR, 0xE000E014
         LDR r1,=SYST_RVR
         MOV r0,#6
         STR r0, [r1,#0]
         .set SYST_CSR, 0xE000E010
         LDR r1,=SYST_CSR
         MOV r0,#3
         STR r0, [r1, #0]
         POP {r2,r4-r7}
         BX LR

.thumb_func
_pendsv_handler:
   LDR r1,=EXC_RETURN_TO_THREAD_PSP 
   CMP r1,LR
   BEQ .setup_next_task
   .pendsv_panic_wrong_stack:
      BKPT 0

   .setup_next_task:
      LDR r0, =_tcb_hdr_struct
      MOV r4,r0
      ADD r4,#72
      LDR r1, [r0,#4]
      CMP r1, #0xFF
      BEQ .init_active_tcb_pointer
         LDR r3,=_tcb_vec_arr
         MOV r2,#36
         MUL r1,r2,r1
         ADD r1,#4
         ADD r3,r1
         MOV r1,r3
         MRS r0,PSP
         PUSH {r0,r1,LR}
         BL .copy_context_frame_from_r0_to_r1
         POP {r0,r1}
         SUB r1,#4
         MOV r3,#2
         ADD r1,#36
         LDR r5,=_tcb_vec_end
         .find_next_in_use_tcb:
            CMP r3,#0
            BEQ .pendsv_panic_no_free_task_blocks
            CMP r1,r5
            BGE .wrap_search
            LDR r6,[r1]
            CMP r6,#1
            BEQ .do_context_switch
            ADD r1,#36
            SUB r3,#1
            B .find_next_in_use_tcb

            .wrap_search: 
               ldr r1,=_tcb_vec_arr
               SUB r3,#1
               B .find_next_in_use_tcb

            .pendsv_panic_no_free_task_blocks:
               BKPT 0 

            .do_context_switch:
               MOV r3,r1
               .change_active_task_idx:
                  LDR r4,=_tcb_vec_arr
                  MOV r5,#0
                  .move_to_next_element:
                     CMP r4,r3
                     BEQ .store_new_index 
                     ADD r4,#36
                     ADD r5,#1
                     B .move_to_next_element
                  .store_new_index:
                     LDR r4,=_tcb_hdr_struct
                     STR r5,[r4,#4]

               ADD r0,r1,#4
               MRS r1,PSP
               BL .copy_context_frame_from_r0_to_r1
               POP {PC}

            
      .init_active_tcb_pointer:
         MOV r1, #0
         STR r1, [r0,#4]
         ADD r0,#12
         MRS r1,PSP
      .copy_context_frame_from_r0_to_r1:
         MOV r5,r1
         LDM r0!,{r1-r4}
         STM r5!,{r1-r4}
         LDM r0!,{r1-r4}
         STM r5!,{r1-r4}
         BX LR

.thumb_func
_systick_handler:
   LDR r1,=EXC_RETURN_TO_THREAD_PSP 
   CMP r1,LR
   BEQ .trigger_pendsv
   .early_return:
      BX LR
   .trigger_pendsv:
      .set SCS_ICSR,  0xE000ED04 
      LDR r1,=SCS_ICSR
      .set PENDSV_SET,0x10000000
      LDR r0,=PENDSV_SET
      STR r0,[r1, #0]
   BX LR
   

_enter_critical_section:
   CPSID i
   BX LR

_exit_critical_section:
   CPSIE i
   BX LR


   _main:
      MOV r0,#42
      LDR r1,=_task_a
      SVC #0

      LDR r1,=_task_b
      SVC #0

      MOV r0,#11
      SVC #0
      B .

   _task_a:
      MOV r0,#100
      MOV r1,#20
      MOV r3,#9
      MOV r6,#44
      MOV r4,#32
      B .

   _task_b:
      MOV r0,#7
      MOV r1,#42
      MOV r7,#88
      MOV r3,#51
      MOV r4,#35
      B .

.pool

.data

   _tcb_hdr_struct: 
      _TASK_BLOCK_HEADER
   _tcb_vec_arr:
      _TASK_STRUCT_BLOCK
   _sec_tcb:
      _TASK_STRUCT_BLOCK
   _tcb_vec_end:
      .4byte 0
   _stack_size:
      .4byte _STACK_SIZE
.align 3
_STACK_END:
   .equ _SP_RESET_VAL, _STACK_END + _STACK_SIZE
