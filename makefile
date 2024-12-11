EXE = anura
TYPE = policy

ifeq ($(OS),Windows_NT)
    override EXE := $(EXE).exe
endif

all:
	cargo rustc --release -- -C target-cpu=native --emit link=$(EXE)

debug:
	cargo rustc -- -C target-cpu=native --emit link=$(EXE)

datagen:
	cargo rustc --release --features "datagen, $(TYPE)" -- -C target-cpu=native --emit link=$(EXE)

perftsuite:
	cargo rustc --release --features "perftsuite" -- -C target-cpu=native --emit link=$(EXE)
	./$(EXE) perftsuite

clean: 
	rm -rf $(EXE) target

run: all
	./$(EXE)

debug-run: debug
	./$(EXE)

bench: all
	./$(EXE) bench