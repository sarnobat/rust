use std::ffi::CString;
use std::fs;
use std::os::raw::{c_char, c_int, c_void};
use std::path::Path;
use walkdir::WalkDir;

// --- Perl C API declarations ---
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

// --- Stub PERL_SYS_INIT3 / PERL_SYS_TERM for static builds ---
#[unsafe(no_mangle)]
pub unsafe extern "C" fn PERL_SYS_INIT3(
    _argc: *mut i32,
    _argv: *mut *mut *mut i8,
    _env: *mut *mut *mut i8,
) {
    println!("[stub] PERL_SYS_INIT3 called");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn PERL_SYS_TERM() {
    println!("[stub] PERL_SYS_TERM called");
}

// ------------------------------------------------------------
// Helper: recursively copy all .pm files from Graph-Easy into /tmp
// ------------------------------------------------------------
fn copy_graph_easy_modules() -> std::io::Result<()> {
    let src_root = Path::new("Graph-Easy-0.64/lib/Graph/Easy");
    if !src_root.exists() {
        eprintln!("Source Graph::Easy path not found: {:?}", src_root);
        std::process::exit(1);
    }

    let dst_root = Path::new("/tmp/graph_easy_lib/Graph/Easy");
    fs::create_dir_all(dst_root)?;

    let mut count = 0;
    for entry in WalkDir::new(src_root) {
        let entry = entry?;
        if entry.file_type().is_file() {
            if let Some(ext) = entry.path().extension() {
                if ext == "pm" {
                    let rel_path = entry.path().strip_prefix(src_root).unwrap();
                    let dst_path = dst_root.join(rel_path);
                    if let Some(parent) = dst_path.parent() {
                        fs::create_dir_all(parent)?;
                    }
                    fs::copy(entry.path(), &dst_path)?;
                    count += 1;
                }
            }
        }
    }
    println!("Copied {} Graph::Easy modules to /tmp/graph_easy_lib", count);
    Ok(())
}

// ------------------------------------------------------------
// Main program
// ------------------------------------------------------------
fn main() {
    // Step 1: Copy Graph::Easy modules into /tmp
    if let Err(e) = copy_graph_easy_modules() {
        eprintln!("Error copying Graph::Easy modules: {}", e);
        std::process::exit(1);
    }

    // Step 2: Set up PERL5LIB safely
    let core_lib = "/tmp/perl_static/lib/5.40.0";
    let graph_lib = "/tmp/graph_easy_lib";
    let perl5lib_value = format!("{}:{}", graph_lib, core_lib);
    unsafe {
        std::env::set_var("PERL5LIB", &perl5lib_value);
    }
    println!("PERL5LIB={}", perl5lib_value);

    // Step 3: Initialize and run embedded Perl
    unsafe {
        let my_perl = perl_alloc();
        if my_perl.is_null() {
            eprintln!("perl_alloc() failed (null pointer)");
            std::process::exit(1);
        }
        perl_construct(my_perl);

        let arg0 = CString::new("perl").unwrap().into_raw();
        let arg1 = CString::new("-I/tmp/graph_easy_lib").unwrap().into_raw();
        let arg2 = CString::new("-I/tmp/perl_static/lib/5.40.0").unwrap().into_raw();
        let arg3 = CString::new("-e").unwrap().into_raw();
        let arg4 = CString::new("0").unwrap().into_raw();
        let mut argv = [arg0, arg1, arg2, arg3, arg4, std::ptr::null_mut()];

        println!(
            "Calling perl_parse with -I /tmp/graph_easy_lib and -I /tmp/perl_static/lib/5.40.0"
        );
        let parse_result =
            perl_parse(my_perl, std::ptr::null_mut(), 5, argv.as_mut_ptr(), std::ptr::null_mut());
        println!("perl_parse() -> {}", parse_result);
        if parse_result != 0 {
            eprintln!("perl_parse() failed, aborting.");
            std::process::exit(1);
        }

        let run_result = perl_run(my_perl);
        println!("perl_run() -> {}", run_result);

        // Step 4: Evaluate a Perl script
        let code = CString::new(
            r#"
                use Graph::Easy;
                my $g = Graph::Easy->new();
                $g->add_edge('Rust', 'Perl');
                print $g->as_ascii();
            "#,
        )
        .unwrap();
        let eval_ptr = Perl_eval_pv(code.as_ptr(), 1);
        if eval_ptr.is_null() {
            eprintln!("Perl_eval_pv() returned NULL");
        }

        perl_destruct(my_perl);
        perl_free(my_perl);
    }
}
