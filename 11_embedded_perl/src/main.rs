use std::ffi::CString;
use std::os::raw::{c_char, c_int, c_void};

extern "C" {
    // From perl.h / embed.h
    fn perl_alloc() -> *mut c_void;
    fn perl_construct(perl: *mut c_void);
    fn perl_parse(perl: *mut c_void,
                  xs_init: *mut c_void,
                  argc: c_int,
                  argv: *mut *mut c_char,
                  env: *mut *mut c_char) -> c_int;
    fn perl_run(perl: *mut c_void) -> c_int;
    fn perl_destruct(perl: *mut c_void);
    fn perl_free(perl: *mut c_void);
}

fn main() {
    unsafe {
        // Prepare args (mimics "perl -e 'print qq{Hello from embedded Perl\n}'")
        let prog = CString::new("perl").unwrap();
        let arg0 = prog.as_ptr() as *mut c_char;
        let arg1 = CString::new("-e").unwrap().into_raw();
        let arg2 = CString::new("print qq{Hello from embedded Perl\\n};").unwrap().into_raw();
        let mut argv = [arg0, arg1, arg2, std::ptr::null_mut()];

        let my_perl = perl_alloc();
        if my_perl.is_null() {
            eprintln!("Failed to allocate perl interpreter");
            return;
        }
        perl_construct(my_perl);
        perl_parse(my_perl, std::ptr::null_mut(), 3, argv.as_mut_ptr(), std::ptr::null_mut());
        perl_run(my_perl);
        perl_destruct(my_perl);
        perl_free(my_perl);
    }
}
