fn main() {
    // Location of libperl.a
    println!("cargo:rustc-link-search=native=/private/tmp/perl-5.40.0");
    println!("cargo:rustc-link-lib=static=perl");

    // Perl headers
    println!("cargo:include=/private/tmp/perl-5.40.0");
    println!("cargo:rerun-if-changed=src/main.rs");
}
