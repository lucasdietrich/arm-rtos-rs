.PHONY: build run disassemble qemu release clean

all: build disassemble

NAME?=demo
TARGET?=thumbv7em-none-eabihf

ELF=target/$(TARGET)/debug/$(NAME)

build:
	cargo build --target $(TARGET) --package $(NAME) -vvv

run:
	cargo run --target $(TARGET)

disassemble:
	arm-none-eabi-objdump -S target/$(TARGET)/debug/$(NAME) > target/$(TARGET)/debug/$(NAME)_src.s
	arm-none-eabi-objdump -d target/$(TARGET)/debug/$(NAME) > target/$(TARGET)/debug/$(NAME).s
	arm-none-eabi-objdump -h target/$(TARGET)/debug/$(NAME) > target/$(TARGET)/debug/$(NAME)_sections.s
	arm-none-eabi-readelf -a target/$(TARGET)/debug/$(NAME) > target/$(TARGET)/debug/$(NAME)_readelf.txt
	arm-none-eabi-readelf -x .isr_vector target/$(TARGET)/debug/$(NAME) > target/$(TARGET)/debug/$(NAME).isr_vector.txt
	arm-none-eabi-readelf -x .text target/$(TARGET)/debug/$(NAME) > target/$(TARGET)/debug/$(NAME).text.txt
	arm-none-eabi-nm --print-size --size-sort --radix=x target/$(TARGET)/debug/$(NAME) > target/$(TARGET)/debug/$(NAME)_size.txt
	arm-none-eabi-readelf -x .bss target/$(TARGET)/debug/$(NAME) > target/$(TARGET)/debug/$(NAME).bss.txt
	arm-none-eabi-readelf -x .data target/$(TARGET)/debug/$(NAME) > target/$(TARGET)/debug/$(NAME).data.txt
	arm-none-eabi-readelf -x .noinit target/$(TARGET)/debug/$(NAME) > target/$(TARGET)/debug/$(NAME).noinit.txt

qemu: build disassemble
	qemu-system-arm \
		-cpu cortex-m4 \
		-machine mps2-an386 \
		-nographic \
		-vga none \
		-semihosting-config enable=on,target=native \
		-kernel $(ELF) \
		-s -S

release:
	cargo build --release --target $(TARGET)

clean:
	cargo clean
