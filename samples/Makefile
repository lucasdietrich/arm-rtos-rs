TOOLCHAIN_PATH?=/home/lucas/x-tools-arm-rtos-rs/arm-unknown-eabihf/bin

CC=$(TOOLCHAIN_PATH)/arm-unknown-eabihf-gcc
LD=$(TOOLCHAIN_PATH)/arm-unknown-eabihf-ld
OBJCOPY=$(TOOLCHAIN_PATH)/arm-unknown-eabihf-objcopy
OBJDUMP=$(TOOLCHAIN_PATH)/arm-unknown-eabihf-objdump
READELF=$(TOOLCHAIN_PATH)/arm-unknown-eabihf-readelf
NM=$(TOOLCHAIN_PATH)/arm-unknown-eabihf-nm

LINKER_SCRIPT=linker.ld

# https://gcc.gnu.org/onlinedocs/gcc/ARM-Options.html
# -fPIC -msingle-pic-base -mpic-register=r9
# -mno-pic-data-is-text-relative ?
# -mno-pic-data-is-text-relative -mword-relocations
# does gcc support RWPI
# -fno-plt ?
CFLAGS=-Og -g -fpic -msingle-pic-base -mpic-register=r9 -mno-pic-data-is-text-relative -fpie

# Toolchain sets -static by default
# no -pie !
LDFLAGS=--print-memory-usage

.PHONY: clean disassemble all

all: clean hello_world.elf disassemble

libc.a:
	$(CC) $(CFLAGS) -c -o libc.o libc.c -I.
	$(AR) rcs libc.a libc.o

hello_world.o: hello_world.c
	$(CC) $(CFLAGS) -c hello_world.c -o hello_world.o -I.

hello_world.elf: hello_world.o libc.a
	$(LD) $(LDFLAGS) hello_world.o -o hello_world.elf -L.

disassemble:
	$(OBJDUMP) -S hello_world.elf > hello_world.elf.src.S
	$(OBJDUMP) -d hello_world.elf > hello_world.elf.S
	$(OBJDUMP) -h hello_world.elf > hello_world.elf.sections
	$(READELF) -a hello_world.elf > hello_world.elf.readelf
	$(READELF) -x .text hello_world.elf > hello_world.elf.text.readelf
	$(READELF) -x .bss hello_world.elf > hello_world.elf.bss.readelf
	$(READELF) -x .data hello_world.elf > hello_world.elf.data.readelf
	$(READELF) -x .noinit hello_world.elf > hello_world.elf.noinit.readelf
	$(READELF) -x .got hello_world.elf > hello_world.elf.got.readelf
	$(READELF) -x .dynamic hello_world.elf > hello_world.elf.dynamic.readelf
	$(NM) -S -D -n hello_world.elf > hello_world.elf.dyn.nm
	$(NM) -S -s -n hello_world.elf > hello_world.elf.nm

clean:
	rm -f *.o *.elf *.S *.sections *.readelf *.nm *.a
