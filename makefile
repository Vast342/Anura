NAME = anura

ifeq ($(OS),Windows_NT)
	EXE := $(NAME).exe
else
	EXE := $(NAME)
endif

all:
	cargo rustc --release -- -C target-cpu=native --emit link=$(EXE)

debug:
	cargo rustc -- -C target-cpu=native --emit link=$(EXE)

clean: 
	rm -rf $(EXE) $(NAME).pdb target