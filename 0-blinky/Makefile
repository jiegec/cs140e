ASSIGNMENT_NAME := assignment0
BASE_URL := https://web.stanford.edu/class/cs140e
SUBMISSION_SITE := $(BASE_URL)/assignments/submission/
SUBMIT_TAR := $(ASSIGNMENT_NAME).tar.gz

FILES_DIR := files
FIRMWARE_DIR := $(FILES_DIR)/firmware
FIRMWARE_TAR := $(FIRMWARE_DIR).tar.gz
ASSIGNMENT_FILES := $(FIRMWARE_TAR) $(addprefix $(FILES_DIR)/, activity-led-blink.bin gpio16-blink.bin)

.PHONY: all submission fetch

all:
	@echo "usage: make [target]"
	@echo "fetch          download assignment files"
	@echo "submission     create submission tarball"
	@echo "clean          clean products from all targets"

submission: $(SUBMIT_TAR)
	@echo "Your submission file "$^" was successfully created."
	@echo "Submit it at $(SUBMISSION_SITE)"

fetch: $(FIRMWARE_DIR) $(ASSIGNMENT_FILES)

$(SUBMIT_TAR):
	@if ! [ -z "$$(git status --porcelain)" ]; then \
	  echo "There are uncommited changes! Aborting."; \
	  exit 1; \
	fi
	git archive --format=tar.gz -o $@ HEAD

$(FILES_DIR):
	@mkdir -p $@

$(ASSIGNMENT_FILES): | $(FILES_DIR)
	wget $(BASE_URL)/assignments/0-blinky/data/$(@:$(FILES_DIR)/%=%) -O $@

$(FIRMWARE_DIR): $(FIRMWARE_TAR) | $(FILES_DIR)
	tar -xzvf $^ -C $(FILES_DIR)
	@touch $(FIRMWARE_DIR)

clean:
	rm -rf $(SUBMIT_TAR) $(FILES_DIR)
	make clean -C phase3
	make clean -C phase4
