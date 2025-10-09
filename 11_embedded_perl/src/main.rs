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

// ------------------------------------------------------------
// Copy all Graph::Easy modules into /tmp for loading
// ------------------------------------------------------------
fn copy_graph_easy_modules() {
    let src_root = Path::new("Graph-Easy-0.64/lib/Graph/Easy");
    let dst_root = Path::new("/tmp/graph_easy_lib/Graph/Easy");

    if !src_root.exists() {
        eprintln!("Source Graph::Easy path not found: {:?}", src_root);
        std::process::exit(1);
    }

    fs::create_dir_all(dst_root).expect("failed to create /tmp/graph_easy_lib/Graph/Easy");

    let mut copied = 0;
    for entry in WalkDir::new(src_root) {
        let entry = entry.expect("walkdir error");
        if entry.file_type().is_file() {
            if let Some(ext) = entry.path().extension() {
                if ext == "pm" {
                    let rel_path = entry.path().strip_prefix(src_root).unwrap();
                    let dst_path = dst_root.join(rel_path);
                    if let Some(parent) = dst_path.parent() {
                        fs::create_dir_all(parent).unwrap();
                    }
                    fs::copy(entry.path(), &dst_path).unwrap();
                    copied += 1;
                }
            }
        }
    }
    println!("Copied {} Graph::Easy modules to /tmp/graph_easy_lib", copied);
}

// Tiny helpers to eval code inside the embedded interpreter
unsafe fn peval_raw(code: &str) {
    let c = CString::new(code).unwrap();
    let _ = Perl_eval_pv(c.as_ptr(), 0);
}

fn main() {
    // 1) Copy modules to /tmp
    copy_graph_easy_modules();

    // 2) Paths we want in @INC
    let core_lib = "/tmp/perl_static/lib/5.40.0";
    let graph_lib = "/tmp/graph_easy_lib";

    if !Path::new(core_lib).exists() {
        eprintln!("Core Perl lib path not found: {}", core_lib);
    }
    if !Path::new(graph_lib).exists() {
        eprintln!("Graph::Easy lib path not found: {}", graph_lib);
    }

    // 3) Start embedded Perl with -I flags to set @INC directly
    // Equivalent of: perl -I/tmp/graph_easy_lib -I/tmp/perl_static/lib/5.40.0 -e 0
    unsafe {
        let my_perl = perl_alloc();
        if my_perl.is_null() {
            eprintln!("perl_alloc() returned null pointer");
            std::process::exit(1);
        }
        perl_construct(my_perl);

        let a0 = CString::new("perl").unwrap();
        let a1 = CString::new("-I").unwrap();
        let a2 = CString::new(graph_lib).unwrap();
        let a3 = CString::new("-I").unwrap();
        let a4 = CString::new(core_lib).unwrap();
        let a5 = CString::new("-e").unwrap();
        let a6 = CString::new("0").unwrap();

        let mut argv: [*mut c_char; 8] = [
            a0.as_ptr() as *mut c_char,
            a1.as_ptr() as *mut c_char,
            a2.as_ptr() as *mut c_char,
            a3.as_ptr() as *mut c_char,
            a4.as_ptr() as *mut c_char,
            a5.as_ptr() as *mut c_char,
            a6.as_ptr() as *mut c_char,
            std::ptr::null_mut(),
        ];

        println!("Calling perl_parse with -I {} and -I {}", graph_lib, core_lib);
        let parse_result = perl_parse(my_perl, std::ptr::null_mut(), 7, argv.as_mut_ptr(), std::ptr::null_mut());
        println!("perl_parse() -> {}", parse_result);

        println!("Running perl_run...");
        let run_result = perl_run(my_perl);
        println!("perl_run() -> {}", run_result);

        // 4) Inspect @INC and Config from inside Perl (no env needed)
        peval_raw(r#"print "Perl $]\n";"#);
        peval_raw(r#"print "\@INC:\n", (map { "  $_\n" } @INC), "\n";"#);
        peval_raw(
            r#"
            require Config;
            print "Config: archname=", $Config::Config{archname},
                  " usedl=", ($Config::Config{usedl}//""), 
                  " useithreads=", ($Config::Config{useithreads}//""), "\n";
        "#,
        );

        // 5) Ensure lib paths are active even if code later resets @INC
        let use_lib_snip = format!(r#"use lib q({}), q({});"#, graph_lib, core_lib);
        peval_raw(&use_lib_snip);

        // 6) Try Graph::Easy
        println!("Evaluating Graph::Easy demo...");
        let code = CString::new(
            r#"
                use Graph::Easy;
                my $g = Graph::Easy->new();
                $g->add_edge('Rust', 'Perl');
                print "\n--- Graph::Easy ASCII ---\n";
                print $g->as_ascii();
                print "\n-------------------------\n";
            "#,
        )
        .unwrap();

        let eval_ptr = Perl_eval_pv(code.as_ptr(), 1);
        if eval_ptr.is_null() {
            eprintln!("Perl_eval_pv() returned null (evaluation failed)");
        } else {
            println!("Perl_eval_pv() executed successfully");
        }

        println!("Destroying interpreter...");
        perl_destruct(my_perl);
        perl_free(my_perl);
        println!("Interpreter shutdown complete");
    }
}
