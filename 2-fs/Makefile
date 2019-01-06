ASSIGNMENT_NAME := assignment2
BASE_URL := https://web.stanford.edu/class/cs140e
BASE_FILES_URL := https://web.stanford.edu/class/cs140e/assignments/2-fs/data
SUBMISSION_SITE := $(BASE_URL)/assignments/submission/
SUBMIT_TAR := $(ASSIGNMENT_NAME).tar.gz

FILES_DIR := files
RESOURCES_DIR := $(FILES_DIR)/resources
RESOURCES_TAR := $(RESOURCES_DIR).tar.gz
FIRMWARE_DIR := $(FILES_DIR)/firmware
FIRMWARE_TAR := $(FIRMWARE_DIR).tar.gz
ASSIGNMENT_FILES := $(FIRMWARE_TAR) $(RESOURCES_TAR)

CS140E_REL_ROOT := ..
REPO_NAMES := 0-blinky 1-shell 2-fs os
QUESTIONS_DIRS := $(shell find . -type d -name "questions")

.PHONY: all test check submission clean

all:
	@echo "usage: make [target]"
	@echo
	@echo "available targets:"
	@echo "fetch          download assignment files"
	@echo "test           run tests for all targets"
	@echo "check          ensure every question is answered"
	@echo "submission     create submission tarball"
	@echo "clean          clean products from all targets"

test:
	cd ../os/kernel && make test
	cd fat32 && cargo test

check:
	@okay=true; \
	for qdir in $(QUESTIONS_DIRS); do \
	    for file in "$${qdir}/"*; do \
		    if ! [ -s "$${file}" ]; then \
			  okay=false; \
			  echo "Question file '$${file}' is empty."; \
		    fi \
	    done \
	done; \
	if ! $$okay; then \
		echo "Questions remain unanswered. Aborting."; \
		exit 1; \
	else \
		echo "All questions appear to be answered."; \
	fi

submission: $(SUBMIT_TAR)
	@echo "Your submission file "$^" was successfully created."
	@echo "Submit it at $(SUBMISSION_SITE)"

fetch: $(FIRMWARE_DIR) $(RESOURCES_DIR) $(ASSIGNMENT_FILES)

.FORCE:
$(SUBMIT_TAR): .FORCE
	@rm -f $@
	@cwd="$${PWD}"; \
	for repo in $(REPO_NAMES); do \
	    repo_path="$${cwd}/$(CS140E_REL_ROOT)/$${repo}"; \
	    cd "$${repo_path}"; \
	    if ! [ -z "$$(git status --porcelain)" ]; then \
		    echo "There are uncommited changes in $${repo}! Aborting."; \
			rm -f $@; \
		    exit 1; \
	    else \
			git_files=$$(git ls-files) ; \
			cd "$${repo_path}/.." ; \
			for file in $$git_files; do \
				tar -rf "$${cwd}/$@" "$${repo}/$${file}"; \
			done \
	    fi \
	done
	@gzip -f $@
	@mv $@.gz $@

$(FILES_DIR):
	@mkdir -p $@

$(ASSIGNMENT_FILES): | $(FILES_DIR)
	wget $(BASE_FILES_URL)/$(@:$(FILES_DIR)/%=%) -O $@

$(FIRMWARE_DIR): $(FIRMWARE_TAR) | $(FILES_DIR)
	tar -xzvf $^ -C $(FILES_DIR)
	@touch $@

$(RESOURCES_DIR): $(RESOURCES_TAR) | $(FILES_DIR)
	tar -xzvf $^ -C $(FILES_DIR)
	@touch $@

clean:
	rm -rf $(FILES_DIR)
	rm -f $(SUBMIT_TAR)
	cd fat32 && cargo clean
