VENDOR?=customrustos
TOOLCHAIN_PATH?=/home/lucas/x-tools-arm-rtos-rs/arm-${VENDOR}-eabihf
SYSROOT=$(TOOLCHAIN_PATH)/lib/gcc/arm-$(VENDOR)-eabihf/14.2.0/

CC=$(TOOLCHAIN_PATH)/bin/arm-${VENDOR}-eabihf-gcc
LD=$(TOOLCHAIN_PATH)/bin/arm-${VENDOR}-eabihf-ld
OBJCOPY=$(TOOLCHAIN_PATH)/bin/arm-${VENDOR}-eabihf-objcopy
OBJDUMP=$(TOOLCHAIN_PATH)/bin/arm-${VENDOR}-eabihf-objdump
READELF=$(TOOLCHAIN_PATH)/bin/arm-${VENDOR}-eabihf-readelf
NM=$(TOOLCHAIN_PATH)/bin/arm-${VENDOR}-eabihf-nm

# Debug options
CFLAGS = -O2 -g

# position independent code
# 
# TODO
#   > https://gcc.gnu.org/onlinedocs/gcc/ARM-Options.html
#   > -fPIC -msingle-pic-base -mpic-register=r9
#   > -mno-pic-data-is-text-relative ?
#   > -mno-pic-data-is-text-relative -mword-relocations
#   > does gcc support RWPI
#   > -fno-plt ?
CFLAGS += -fPIC -msingle-pic-base -mpic-register=r9 -mno-pic-data-is-text-relative -fpie

# set sysroot to the toolchain path (not required, but helps bear/clangd to find the headers)
CFLAGS += --sysroot=$(SYSROOT)

# disable builtins (e.f. strlen inlining)
CFLAGS += -fno-builtin

# Toolchain sets -static by default
# no -pie !
LDFLAGS=--print-memory-usage

.PHONY: clean disassemble all

all: clean hello_world.elf disassemble

libuser.a:
	$(CC) $(CFLAGS) -c -o libuser.o libuser.c -I.
	$(AR) rcs libuser.a libuser.o

libc.a:
	$(CC) $(CFLAGS) -c -o libc.o libc.c -I.
	$(AR) rcs libc.a libc.o

hello_world.o: hello_world.c
	$(CC) $(CFLAGS) -c hello_world.c -o hello_world.o -I.

hello_world.elf: hello_world.o libc.a libuser.a
	$(LD) $(LDFLAGS) hello_world.o -o hello_world.elf -L. -nostdlib -lc -luser -Map hello_world.map

disassemble:
	$(OBJDUMP) -S hello_world.elf > hello_world.elf.src.S
	$(OBJDUMP) -d hello_world.elf > hello_world.elf.S
	$(OBJDUMP) -h hello_world.elf > hello_world.elf.sections
	$(READELF) -a hello_world.elf > hello_world.elf.readelf
	$(READELF) -x .text hello_world.elf > hello_world.elf.text.readelf
	$(READELF) -x .bss hello_world.elf > hello_world.elf.bss.readelf
	$(READELF) -x .data hello_world.elf > hello_world.elf.data.readelf
	$(READELF) -x .noinit hello_world.elf > hello_world.elf.noinit.readelf
	$(READELF) -x .text hello_world.elf > hello_world.elf.text.readelf
	$(READELF) -x .rodata hello_world.elf > hello_world.elf.rodata.readelf
	$(READELF) -x .got hello_world.elf > hello_world.elf.got.readelf
	$(READELF) -x .dynamic hello_world.elf > hello_world.elf.dynamic.readelf
	$(NM) -S -D -n hello_world.elf > hello_world.elf.dyn.nm
	$(NM) -S -s -n hello_world.elf > hello_world.elf.nm

clean:
	rm -f *.o *.elf *.S *.sections *.readelf *.nm *.a
