fn main() {
    // Use the real directory containing libperl.dylib
    println!("cargo:rustc-link-search=native=/tmp/perl_embed/lib/5.40.0/darwin-2level/CORE");
    println!("cargo:rustc-link-lib=dylib=perl");
    println!("cargo:rustc-link-arg=-mmacosx-version-min=14.0");
}
