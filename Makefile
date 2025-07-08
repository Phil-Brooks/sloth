VER = 0.1.9
EXE = sloth

ifeq ($(OS),Windows_NT)
	NAME := $(EXE).exe
	OLD := sloth-$(VER).exe
else
	NAME := $(EXE)
	OLD := sloth-$(VER)
endif

rule:
	cargo clean
	cargo rustc --release -- -C target-cpu=native --emit link=$(NAME)

release:
	cargo rustc --release -- --emit link=$(OLD)
