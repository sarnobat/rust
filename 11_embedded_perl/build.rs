fn main() {
    println!("cargo:rustc-link-search=native=/tmp/perl_static/lib/5.40.0/darwin-2level/CORE");
    println!("cargo:rustc-link-lib=static=perl");
    println!("cargo:rustc-link-arg=-mmacosx-version-min=14.0");
}
