#!/bin/bash

gdb_cmd_file=$1
elf_file=$2

sudo openocd -f interface/cmsis-dap.cfg -f target/rp2040.cfg -c "adapter speed 5000" &
bg_ocd=$!
arm-none-eabi-gdb --batch --command=$gdb_cmd_file $elf_file
kill $bg_ocd
