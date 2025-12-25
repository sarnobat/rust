use std::path::Path;

fn main() {
    // If pre-generated ANTLR sources exist in `src/antlr`, enable a cfg
    // so the crate can compile using the generated lexer. Otherwise
    // fall back to a simple tokenizer at runtime.
    if Path::new("src/antlr/cronlexer.rs").exists() {
        println!("cargo:rerun-if-changed=src/antlr/cronlexer.rs");
        println!("cargo:rustc-cfg=has_antlr");
    } else {
        println!("cargo:warning=No generated ANTLR lexer found; building without ANTLR runtime (fallback tokenizer enabled)");
    }
}