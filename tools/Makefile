BIN  := bin
BINS := $(patsubst %.cpp,$(BIN)/%,$(wildcard *.cpp))

.PHONY: all, clean
all: $(BINS)

clean:
	rm -rf $(BIN)

$(BIN)/%: %.cpp
	@mkdir -p $(BIN)
	clang++ -O3 -Wall -Wextra $^ -o $@
