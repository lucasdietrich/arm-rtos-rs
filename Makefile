.PHONY: build run disassemble qemu release clean debug

all: build disassemble

NAME?=demo
TARGET?=thumbv7em-none-eabihf
ELF=target/$(TARGET)/debug/$(NAME)

build:
	cargo build --target $(TARGET) --package $(NAME) -vvv

run:
	cargo run --target $(TARGET)

disassemble:
	./scripts/disassemble.sh $(ELF)

qemu: build disassemble
	qemu-system-arm \
		-cpu cortex-m4 \
		-machine mps2-an386 \
		-nographic \
		-vga none \
		-semihosting-config enable=on,target=native \
		-device loader,file=$(ELF) \
		-s -S

debug: qemu

release:
	cargo build --release --target $(TARGET)

clean:
	cargo clean
