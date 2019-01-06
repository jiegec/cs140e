ASSIGNMENT_NAME := assignment1
BASE_URL := https://web.stanford.edu/class/cs140e

FILES_DIR := files
FIRMWARE_DIR := $(FILES_DIR)/firmware
FIRMWARE_TAR := $(FIRMWARE_DIR).tar.gz
ASSIGNMENT_FILES := $(FIRMWARE_TAR) $(addprefix $(FILES_DIR)/,act-led-blink.bin)

.PHONY: all fetch

all:
	@echo "usage: make [target]"
	@echo "fetch          download assignment files"
	@echo "clean          clean products from all targets"

fetch: $(FIRMWARE_DIR) $(ASSIGNMENT_FILES)

$(FILES_DIR):
	@mkdir -p $@

$(ASSIGNMENT_FILES): | $(FILES_DIR)
	wget $(BASE_URL)/files/$(@:$(FILES_DIR)/%=%) -O $@

$(FIRMWARE_DIR): $(FIRMWARE_TAR) | $(FILES_DIR)
	tar -xzvf $^ -C $(FILES_DIR)
	@touch $(FIRMWARE_DIR)

clean:
	rm -rf $(FILES_DIR)
	make clean -C kernel
	cd volatile && cargo clean
	cd pi && cargo clean
