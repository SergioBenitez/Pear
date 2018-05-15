#![no_main]
#[macro_use] extern crate libfuzzer_sys;
extern crate uri;

use uri::parse_bytes;

fuzz_target!(|data: &[u8]| {
    let _ = parse_bytes(data);
});
