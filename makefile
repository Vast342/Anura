EXE = anura

ifeq ($(OS),Windows_NT)
    override EXE := $(EXE).exe
endif

all:
	cargo rustc --release -- -C target-cpu=native --emit link=$(EXE)

debug:
	cargo rustc -- -C target-cpu=native --emit link=$(EXE)

clean: 
	rm -rf $(EXE) target

run: all
	./$(EXE)

debug-run: debug
	./$(EXE)

bench: all
	./$(EXE) bench