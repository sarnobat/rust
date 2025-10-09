use std::env;
use std::ffi::CString;
use std::fs;
use std::os::raw::{c_char, c_int, c_void};
use std::path::Path;

// ------------------------------------------------------------
// Perl API (from embed.h / perlapi.h)
// Must be inside an unsafe extern block.
// ------------------------------------------------------------
unsafe extern "C" {
    fn perl_alloc() -> *mut c_void;
    fn perl_construct(p: *mut c_void);
    fn perl_parse(
        p: *mut c_void,
        xs_init: *mut c_void,
        argc: c_int,
        argv: *mut *mut c_char,
        env: *mut *mut c_char,
    ) -> c_int;
    fn perl_run(p: *mut c_void) -> c_int;
    fn perl_destruct(p: *mut c_void);
    fn perl_free(p: *mut c_void);
    fn Perl_eval_pv(code: *const c_char, croak_on_error: i32) -> *mut c_void;
}

// ------------------------------------------------------------
// Write embedded Graph::Easy module to /tmp/graph_easy_lib
// ------------------------------------------------------------
fn write_graph_easy_lib() {
    let base = Path::new("/tmp/graph_easy_lib/Graph");
    fs::create_dir_all(base).expect("failed to create Graph directory");

    // Embed the Graph::Easy module directly into the binary
    fs::write(
        base.join("Easy.pm"),
        include_str!("../embedded_lib/Graph-Easy-0.64/lib/Graph/Easy.pm"),
    )
    .expect("failed to write Easy.pm");
}

// ------------------------------------------------------------
// Main
// ------------------------------------------------------------
fn main() {
    // Step 1. Extract Graph::Easy module to /tmp
    write_graph_easy_lib();

    // Step 2. Point Perl @INC to our temporary library
    unsafe {
        std::env::set_var("PERL5LIB", "/tmp/graph_easy_lib");
    }

    // Step 3. Initialize, run, and tear down Perl interpreter
    unsafe {
        // Allocate and construct interpreter
        let my_perl = perl_alloc();
        perl_construct(my_perl);

        // Fake command-line arguments for perl_parse
        let arg0 = CString::new("perl").unwrap().into_raw();
        let arg1 = CString::new("-e").unwrap().into_raw();
        let arg2 = CString::new("0").unwrap().into_raw();
        let mut argv = [arg0, arg1, arg2, std::ptr::null_mut()];

        perl_parse(
            my_perl,
            std::ptr::null_mut(),
            3,
            argv.as_mut_ptr(),
            std::ptr::null_mut(),
        );
        perl_run(my_perl);

        // Run a small Graph::Easy demo
        let code = CString::new(
            r#"
                use Graph::Easy;
                my $g = Graph::Easy->new();
                $g->add_edge('Rust', 'Perl');
                print $g->as_ascii();
            "#,
        )
        .unwrap();
        Perl_eval_pv(code.as_ptr(), 1);

        perl_destruct(my_perl);
        perl_free(my_perl);
    }
}
