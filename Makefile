.DEFAULT_GOAL = help
SHELL := /bin/bash

GREEN := $(shell command -v tput >/dev/null 2>&1 && tput -Txterm setaf 2 || echo "")
YELLOW := $(shell command -v tput >/dev/null 2>&1 && tput -Txterm setaf 3 || echo "")
RED := $(shell command -v tput >/dev/null 2>&1 && tput -Txterm setaf 1 || echo "")
RESET := $(shell command -v tput >/dev/null 2>&1 && tput -Txterm sgr0 || echo "")

HELP_FUN = %help; \
	while(<>) { push @{$$help{$$2 // "Other"}}, [$$1, $$3] if /^([a-zA-Z\-._]+)\s*:.*\#\#(?:@([a-zA-Z\-_]+))?\s(.*)$$/ }; \
	print "$(RESET)project: $(PURPLE)$(NAME)$(RESET)\n"; \
	print "usage: make [target]\n\n"; \
	for (sort keys %help) { \
	print "$$_:\n"; \
	for (@{$$help{$$_}}) { \
	$$sep = " " x (25 - length $$_->[0]); \
	print " ${YELLOW}$$_->[0]${RESET}$$sep${GREEN}$$_->[1]${RESET}\n"; \
	}; \
	print "\n"; }

.PHONY: help flamegraph geiger tree udeps crev audit deny bloat clean

help: ## Show this help.
	@perl -e '$(HELP_FUN)' $(MAKEFILE_LIST)

_TEXT_NB_FILES := 10
text: ## Create test data
	rm -rf text
	mkdir text
	curl http://metaphorpsum.com/paragraphs/20 > text/a
	curl http://metaphorpsum.com/paragraphs/20 > text/b
	for ((i=1;i<=$(_TEXT_NB_FILES);i++)); do \
		mkdir text/$$i; \
		cp text/a text/a$$i &\
		cp text/b text/b$$i &\
		for ((j=1;j<=$(_TEXT_NB_FILES);j++)); do \
			mkdir text/$$i/$$j; \
			cp text/a text/$$i/a$$j &\
			cp text/b text/$$i/b$$j &\
			for ((k=1;k<=$(_TEXT_NB_FILES);k++)); do \
				mkdir text/$$i/$$j/$$k; \
				cp text/a text/$$i/$$j/a$$k &\
				cp text/b text/$$i/$$j/b$$k &\
				for ((l=1;l<=$(_TEXT_NB_FILES);l++)); do \
					mkdir text/$$i/$$j/$$k/$$l; \
					cp text/a text/$$i/$$j/$$k/a$$l &\
					cp text/b text/$$i/$$j/$$k/b$$l &\
				done;\
			done;\
		done;\
	done;\
	wait
	git -C ./text init
	git -C ./text add -A
	git -C ./text commit -m "lol"

perf: text ## Run perf
	cargo build --release
	perf record ./target/release/rsr ./text -s 'a' -r 'b'
	perf report

flamegraph: text ## Run perf with (bad) svg output
	cargo install flamegraph
	cargo flamegraph --open ./text -s 'a' -r 'b'

run: text ## Rust run, on the test data
	cargo build --release
	./target/release/rsr ./text -s 'a' -r 'b'

clean: ## Kinda clean the repo
	rm -rf text flamegraph.svg perf.data* text

miri: ## Run tests
	rustup +nightly component add miri
	cargo clean
	cargo miri run ./text -s '(a)' -r 'SKJFf;$$(1U)sdfh'
	cargo clean

geiger: ## Warn about unsafe
	cargo install cargo-geiger
	cargo geiger

tree: ## Dependency tree
	cargo tree

fmt: ## __
	cargo fmt

udeps: ## Unused dependencies
	cargo install cargo-udeps --locked
	cargo udeps

crev: ## ??
	cargo install cargo-crev
	cargo crev

audit: ## ??
	cargo install cargo-audit
	cargo audit

deny: ## Deny crates
	cargo install cargo-deny
	cargo deny init
	cargo deny check

bloat: ## Why is my executable so big ?
	cargo install cargo-bloat
	cargo bloat

release: ## Release
	cargo install cargo-release
	cargo release --dry-run

