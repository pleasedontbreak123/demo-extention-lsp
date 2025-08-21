use std::{
    ffi::{CStr, CString},
    mem::forget,
};

use serde_json::to_string;

#[repr(C)]
#[derive(Debug, Clone)]
pub struct FfiSpiceParseResult {
    pub code: u32,
    pub payload: *const i8,
}

#[link(name = "spice_parser_clib")]
extern "C" {
    fn pspice_file_parse(file: *const i8) -> FfiSpiceParseResult;
    fn spice_free(result: FfiSpiceParseResult);
}

pub const PARSE_OK: u32 = 1;
pub const PARSE_ERROR: u32 = 0;

pub fn parse_pspice_file(content: &str) {
    let parsed = unsafe { pspice_file_parse(c".bug".as_ptr() as *const i8) };
    eprintln!("{}:{}", parsed.code, unsafe {
        CStr::from_ptr(parsed.payload).to_str().unwrap()
    });
    assert!(parsed.code == PARSE_ERROR);
    unsafe { spice_free(parsed) };
}
