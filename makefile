EXE = anura

ifeq ($(OS),Windows_NT)
    override EXE := $(EXE).exe
endif

define get_target
$(shell rustc --print target-spec-json | grep -o '"llvm-target": *"[^"]*"' | cut -d'"' -f4)
endef

all:
	cargo rustc --release --features "datagen" -- -C target-cpu=native --emit link=$(EXE)

debug:
	cargo rustc -- -C target-cpu=native --emit link=$(EXE)

datagen:
	cargo rustc --release --features "datagen" -- -C target-cpu=native --emit link=$(EXE)

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
ifeq ($(OS),Windows_NT)
	$(eval TARGET := $(call get_target))
	move /Y "target/$(TARGET)/release/$(EXE)" "$(EXE)"
else
	$(eval TARGET := $(call get_target))
	mv "target/$(TARGET)/release/$(EXE)" "$(EXE)"
endif