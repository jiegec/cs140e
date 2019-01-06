TARGET ?= aarch64-none-elf
CROSS ?= $(TARGET)

CC := $(CROSS)-gcc
XARGO := CARGO_INCREMENTAL=0 RUST_TARGET_PATH="$(shell pwd)" xargo

LD_LAYOUT := ext/layout.ld

RUST_BINARY := $(shell cat Cargo.toml | grep name | cut -d\" -f 2 | tr - _)
RUST_BUILD_DIR := target/$(TARGET)
RUST_DEBUG_LIB := $(RUST_BUILD_DIR)/debug/lib$(RUST_BINARY).a
RUST_RELEASE_LIB := $(RUST_BUILD_DIR)/release/lib$(RUST_BINARY).a

RUST_DEPS = Xargo.toml Cargo.toml build.rs $(LD_LAYOUT) src/*
EXT_DEPS = $(BUILD_DIR)/crt0.o

BUILD_DIR := build
KERNEL := $(BUILD_DIR)/$(RUST_BINARY)
RUST_LIB := $(BUILD_DIR)/$(RUST_BINARY).a

.PHONY: all clean check

VPATH = ext

all: $(KERNEL).hex $(KERNEL).bin

check:
	@$(XARGO) check --target=$(TARGET)

$(RUST_DEBUG_LIB): $(RUST_DEPS)
	@echo "+ Building $@ [xargo]"
	@$(XARGO) build --target=$(TARGET)

$(RUST_RELEASE_LIB): $(RUST_DEPS)
	@echo "+ Building $@ [xargo --release]"
	@$(XARGO) build --release --target=$(TARGET)

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
	@$(CROSS)-ld --gc-sections -o $@ $^ -T$(LD_LAYOUT)

$(KERNEL).hex: $(KERNEL).elf | $(BUILD_DIR)
	@echo "+ Building $@ [objcopy $<]"
	@$(CROSS)-objcopy $< -O ihex $@

$(KERNEL).bin: $(KERNEL).elf | $(BUILD_DIR)
	@echo "+ Building $@ [objcopy $<]"
	@$(CROSS)-objcopy $< -O binary $@

clean:
	$(XARGO) clean
	rm -rf $(BUILD_DIR)
