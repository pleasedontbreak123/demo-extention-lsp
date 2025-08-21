use std::{
    ffi::{CStr, CString},
    mem::forget,
};

use serde_json::to_string;
use spice_parser_core::{lexer::SpiceLexer, parse::SpiceFileParser};

#[repr(C)]
#[derive(Debug, Clone)]
pub struct SpiceParseResult {
    pub code: u32,
    pub payload: *const i8,
}

#[unsafe(no_mangle)]
pub extern "C" fn pspice_file_parse(content: *const i8) -> SpiceParseResult {
    //let content = unsafe { std::str::from_utf8_unchecked(std::slice::from_raw_parts(content, 1024)) };
    let content = unsafe { CStr::from_ptr(content) }.to_str().unwrap();
    let lexer = SpiceLexer::tokenize(content);
    let mut parser = SpiceFileParser::new(&lexer);
    let result = parser.parse();
    match result {
        Ok(p) => {
            let p = to_string(&p).unwrap();
            let p = CString::new(p).unwrap();
            let ret = SpiceParseResult {
                code: PARSE_OK,
                payload: p.as_ptr(),
            };
            forget(p);
            ret
        }
        Err(e) => {
            let e = CString::new(to_string(&e).unwrap()).unwrap();
            let ret = SpiceParseResult {
                code: PARSE_ERROR,
                payload: e.as_ptr() as *const i8,
            };
            forget(e);
            ret
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn spice_free(result: SpiceParseResult) {
    unsafe {
        let _ = Box::from_raw(result.payload as *mut u8);
    }
}

pub const PARSE_OK: u32 = 1;
pub const PARSE_ERROR: u32 = 0;

#[test]
fn clib_test() {
    let dylib_path = test_cdylib::build_current_project();

    let dylib = dlopen::symbor::Library::open(dylib_path).unwrap();
    let parse = unsafe {
        dylib.symbol::<unsafe extern "C" fn(*const u8) -> SpiceParseResult>("pspice_file_parse")
    }
    .unwrap();
    let free =
        unsafe { dylib.symbol::<unsafe extern "C" fn(SpiceParseResult)>("spice_free") }.unwrap();

    let parsed = unsafe { parse(c".bug".as_ptr() as *const u8) };
    eprintln!("{}:{}", parsed.code, unsafe {
        CStr::from_ptr(parsed.payload).to_str().unwrap()
    });
    assert!(parsed.code == PARSE_ERROR);
    unsafe { free(parsed) };

    let parsed = unsafe { parse(c"OOPS".as_ptr() as *const u8) };
    eprintln!("{}:{}", parsed.code, unsafe {
        CStr::from_ptr(parsed.payload).to_str().unwrap()
    });
    assert!(parsed.code == PARSE_OK);
    unsafe { free(parsed) };
}
