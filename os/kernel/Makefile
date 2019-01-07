TARGET ?= aarch64-none-elf
CROSS ?= $(TARGET)
ifeq ($(QEMU),1)
	CARGO_FLAGS ?= --features qemu,no_test
else
	CARGO_FLAGS ?= --features no_test
endif

CC := $(CROSS)-gcc
TTYWRITE ?= ttywrite
PI_TTY ?= /dev/tty.SLAB_USBtoUART -b 230400
CCFLAGS ?= -Wall -O2 -nostdlib -nostartfiles -ffreestanding -pie -fpie
LDFLAGS ?= --gc-sections -static -nostdlib -nostartfiles --no-dynamic-linker
CARGO_XBUILD ?= RUST_TARGET_PATH="$(shell pwd)" cargo xbuild --target=$(TARGET) -v
CARGO ?= cargo

LD_LAYOUT := ext/layout.ld

RUST_BINARY := $(shell cat Cargo.toml | grep name | cut -d\" -f 2 | tr - _)
RUST_BUILD_DIR := target/$(TARGET)
RUST_DEBUG_LIB := $(RUST_BUILD_DIR)/debug/lib$(RUST_BINARY).a
RUST_RELEASE_LIB := $(RUST_BUILD_DIR)/release/lib$(RUST_BINARY).a

RUST_LIB_DEPS = ../pi/src/* ../pi/src/*/** \
				../../1-shell/stack-vec/src/* \
				../../2-fs/fat32/src/* ../../2-fs/fat32/src/*/** src/*/**

RUST_DEPS = Cargo.toml build.rs $(LD_LAYOUT) src/* $(RUST_LIB_DEPS)
EXT_DEPS = $(BUILD_DIR)/init.o

BUILD_DIR := build
KERNEL := $(BUILD_DIR)/$(RUST_BINARY)
RUST_LIB := $(BUILD_DIR)/$(RUST_BINARY).a

.PHONY: all test clean check install

VPATH = ext

all: $(KERNEL).hex $(KERNEL).bin

check:
	@$(CARGO) check

test:
	@$(CARGO) test

install: $(KERNEL).bin
	$(TTYWRITE) -i $< $(PI_TTY)

$(RUST_DEBUG_LIB): $(RUST_DEPS)
	@echo "+ Building $@ [xargo]"
	@$(CARGO_XBUILD) $(CARGO_FLAGS)

$(RUST_RELEASE_LIB): $(RUST_DEPS)
	@echo "+ Building $@ [xargo --release]"
	@$(CARGO_XBUILD) --release $(CARGO_FLAGS)

ifeq ($(DEBUG),1)
$(RUST_LIB): $(RUST_DEBUG_LIB) | $(BUILD_DIR)
	@cp $< $@
else
$(RUST_LIB): $(RUST_RELEASE_LIB) | $(BUILD_DIR)
	@cp $< $@
endif

$(BUILD_DIR):
	@mkdir -p $@

$(BUILD_DIR)/%.o: %.c | $(BUILD_DIR)
	@echo "+ Building $@ [cc $<]"
	@$(CC) $(CCFLAGS) -c $< -o $@

$(BUILD_DIR)/%.o: %.S | $(BUILD_DIR)
	@echo "+ Building $@ [as $<]"
	@$(CC) $(CCFLAGS) -c $< -o $@

$(KERNEL).elf: $(EXT_DEPS) $(RUST_LIB) | $(BUILD_DIR)
	@echo "+ Building $@ [ld $^]"
	@$(CROSS)-ld $(LDFLAGS) -T$(LD_LAYOUT) $^ -o $@

$(KERNEL).hex: $(KERNEL).elf | $(BUILD_DIR)
	@echo "+ Building $@ [objcopy $<]"
	@$(CROSS)-objcopy $< -O ihex $@

$(KERNEL).bin: $(KERNEL).elf | $(BUILD_DIR)
	@echo "+ Building $@ [objcopy $<]"
	@$(CROSS)-objcopy $< -O binary $@

clean:
	$(CARGO) clean
	rm -rf $(BUILD_DIR)

qemu: $(KERNEL).bin
	qemu-system-aarch64 -machine raspi3 -serial null -serial mon:stdio -kernel build/kernel.bin -sd ../../2-fs/files/resources/mock1.fat32.img

qemu-debug: $(KERNEL).bin
	qemu-system-aarch64 -machine raspi3 -serial null -serial mon:stdio -kernel build/kernel.bin -s -sd ../../2-fs/files/resources/mock1.fat32.img
