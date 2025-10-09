use std::ffi::CString;
use std::fs;
use std::os::raw::{c_char, c_int, c_void};
use std::path::Path;
use std::process::Command;
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


// --- real FFI ---
unsafe extern "C" {
    fn PERL_SYS_INIT3(argc: *mut c_int,
                      argv: *mut *mut *mut c_char,
                      env:  *mut *mut *mut c_char);
    fn PERL_SYS_TERM();
}

#[no_mangle]
pub unsafe extern "C" fn PERL_SYS_INIT3(
    _argc: *mut c_int,
    _argv: *mut *mut *mut c_char,
    _env:  *mut *mut *mut c_char,
) {
    eprintln!("[stub] PERL_SYS_INIT3 called");
}

#[no_mangle]
pub unsafe extern "C" fn PERL_SYS_TERM() {
    eprintln!("[stub] PERL_SYS_TERM called");
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


    // --- inside main(), before perl_alloc() ---
    unsafe {
        let mut argc: c_int = 0;
        let mut argv: *mut *mut c_char = std::ptr::null_mut();
        let mut env:  *mut *mut c_char = std::ptr::null_mut();

        // cast addresses of these double-pointer vars to triple-pointer args
        PERL_SYS_INIT3(&mut argc,
                    &mut argv as *mut *mut *mut c_char,
                    &mut env  as *mut *mut *mut c_char);
        println!("PERL_SYS_INIT3 done");
    }

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

        // --- Added diagnostic block ---
        if parse_result != 0 {
            eprintln!("\n=== Perl bootstrap diagnostic ===");
            eprintln!("PERL5LIB = {}", std::env::var("PERL5LIB").unwrap_or_default());
            eprintln!("\n/tmp/graph_easy_lib contents:");
            let _ = Command::new("find")
                .arg("/tmp/graph_easy_lib")
                .arg("-maxdepth")
                .arg("2")
                .status();

            eprintln!("\n/tmp/perl_static/lib/5.40.0 contents:");
            let _ = Command::new("ls")
                .arg("-l")
                .arg("/tmp/perl_static/lib/5.40.0")
                .status();

            eprintln!(
                "\nManual test suggestion:\n\
                /private/tmp/perl_static/bin/perl \\\n\
                \t-I/tmp/graph_easy_lib \\\n\
                \t-I/tmp/perl_static/lib/5.40.0 \\\n\
                \t-e 'use Graph::Easy; print qq(OK\\n)'"
            );
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
