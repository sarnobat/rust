TARGET := git-uncommitted
CARGO  := cargo
PROFILE := release
BIN := target/$(PROFILE)/$(TARGET)

.PHONY: all build test clean

all: build test

build:
	$(CARGO) build --$(PROFILE)

test: build
	ls -d \
		/Volumes/git/github/* \
		/Volumes/git/github/depot_parent/* \
		/Volumes/git/2023/repos_personal.git/* \
		| target/release/git-uncommitted --long

clean:
	$(CARGO) clean
