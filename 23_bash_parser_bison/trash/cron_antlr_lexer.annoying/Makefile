# antlr4-rust currently supports grammars generated with ANTLR 4.9.3
ANTLR_VERSION ?= 4.9.3
ANTLR_DIR     ?= target/antlr
ANTLR_JAR     ?= $(ANTLR_DIR)/antlr-$(ANTLR_VERSION)-complete.jar
GRAMMAR   ?= grammars/CronLexer.g4
ANTLR_OUT ?= src/antlr

.PHONY: all antlr download-antlr

all: antlr
	cargo build && crontab -l | target/debug/croncheck 2>(tee /tmp/out.err)
	//cat /tmp/out.err
	ls /tmp/out.err

antlr: download-antlr
	@mkdir -p $(ANTLR_OUT)
	java -jar $(ANTLR_JAR) -Dlanguage=Rust -o $(ANTLR_OUT) $(GRAMMAR)

download-antlr: $(ANTLR_JAR)

$(ANTLR_JAR):
	@mkdir -p $(ANTLR_DIR)
	curl -L "https://github.com/rrevenantt/antlr4rust/releases/download/antlr4-4.8-2-Rust-0.2/antlr4-4.8-2-SNAPSHOT-complete.jar" -o $(ANTLR_JAR)
