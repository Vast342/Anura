EXE = anura
TYPE = policy

ifeq ($(OS),Windows_NT)
    override EXE := $(EXE).exe
endif

TARGET_TUPLE := $(shell rustc --print host-tuple)

ifeq ($(OS),Windows_NT)
	PGO_MOVE := move /Y "target/$(TARGET_TUPLE)/release/$(EXE)" "$(EXE)"
else
	PGO_MOVE := mv "target/$(TARGET_TUPLE)/release/$(EXE)" "$(EXE)"
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

pgo:
	cargo pgo instrument
	cargo pgo run -- bench
	cargo pgo optimize
	$(PGO_MOVE)
