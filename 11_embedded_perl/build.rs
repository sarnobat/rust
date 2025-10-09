fn main() {
    // Path to Perl's CORE dir (headers + dylib)
    let perl_core = "/opt/homebrew/opt/perl/lib/perl5/5.40/darwin-thread-multi-2level/CORE";

    println!("cargo:rustc-link-search=native={perl_core}");
    println!("cargo:rustc-link-lib=dylib=perl");  // libperl.dylib → -lperl
    println!("cargo:rustc-link-arg=-mmacosx-version-min=14.0");

    // Optional: pass include path to bindgen if you later generate bindings
    println!("cargo:include={perl_core}");
}
