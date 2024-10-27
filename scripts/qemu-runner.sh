#!/usr/bin/bash

# get first argument
elf=$1

qemu-system-arm \
    -cpu cortex-m4 \
    -machine mps2-an386 \
    -nographic \
    -vga none \
    -semihosting-config enable=on,target=native \
    -device loader,file=${elf}

    # -s -S
    # -pidfile qemu.pid
    # -device loader,file=${target}

    # -rtc clock=vm
    # -icount shift=10,align=off,sleep=on
    
    # -chardev stdio,id=myCon
    # -serial chardev:myCon
    # -chardev file,id=myFile,mux=on,path=io.txt
    # -serial chardev:myFile
    # -serial chardev:myFile
    # -serial chardev:myFile
    # -serial chardev:myFile
