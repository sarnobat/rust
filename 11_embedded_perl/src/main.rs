use std::env;
use std::ffi::CString;
use std::os::raw::{c_char, c_int, c_void};

unsafe extern "C" {
    fn perl_alloc() -> *mut c_void;
    fn perl_construct(perl: *mut c_void);
    fn perl_parse(
        perl: *mut c_void,
        xs_init: *mut c_void,
        argc: c_int,
        argv: *mut *mut c_char,
        env: *mut *mut c_char,
    ) -> c_int;
    fn perl_run(perl: *mut c_void) -> c_int;
    fn perl_destruct(perl: *mut c_void);
    fn perl_free(perl: *mut c_void);
}

fn main() {
    //----------------------------------------------------------------------
    // 1.  Point Perl at its real “installation root”
    //----------------------------------------------------------------------
    unsafe {
        // Homebrew’s prefix path for Perl 5.40
        env::set_var("PERL5LIB",
            "/opt/homebrew/opt/perl/lib/perl5/5.40:/opt/homebrew/opt/perl/lib/perl5/site_perl/5.40");
        env::set_var("PERL_CORE",
            "/opt/homebrew/opt/perl/lib/perl5/5.40/darwin-thread-multi-2level/CORE");
        // This one is the crucial hint used by libperl’s init code:
        env::set_var("PERL_HOME", "/opt/homebrew/opt/perl");
    }

    println!("PERL_HOME = {:?}", env::var("PERL_HOME").unwrap_or_default());

    unsafe {
        //------------------------------------------------------------------
        // 2.  Equivalent of:
        //     perl -I... -e 'print qq{Hello from embedded Perl\n}'
        //------------------------------------------------------------------
        let arg0 = CString::new("perl").unwrap().into_raw();
        let arg1 = CString::new("-I/opt/homebrew/opt/perl/lib/perl5/5.40").unwrap().into_raw();
        let arg2 = CString::new("-I/opt/homebrew/opt/perl/lib/perl5/site_perl/5.40").unwrap().into_raw();
        let arg3 = CString::new("-e").unwrap().into_raw();
        let arg4 = CString::new("print qq{Hello from embedded Perl\\n};").unwrap().into_raw();
        let mut argv = [arg0, arg1, arg2, arg3, arg4, std::ptr::null_mut()];

        //------------------------------------------------------------------
        // 3.  Standard embedding lifecycle
        //------------------------------------------------------------------
        let my_perl = perl_alloc();
        if my_perl.is_null() {
            eprintln!("Failed to allocate Perl interpreter");
            return;
        }

        perl_construct(my_perl);

        // tell Perl to see our environment
        let parse_code = perl_parse(
            my_perl,
            std::ptr::null_mut(),
            5,
            argv.as_mut_ptr(),
            std::ptr::null_mut(),
        );
        if parse_code != 0 {
            eprintln!("perl_parse failed with code {}", parse_code);
            perl_free(my_perl);
            return;
        }

        perl_run(my_perl);
        perl_destruct(my_perl);
        perl_free(my_perl);
    }
}
