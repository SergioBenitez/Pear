#![no_main]
#[macro_use] extern crate libfuzzer_sys;
extern crate uri;

use uri::{Uri, parse_bytes};

fuzz_target!(|data: &[u8]| {
    if let Ok(uri) = parse_bytes(data) {
        if let Uri::Authority(ref auth) = uri {
            if auth.port().is_some() {
                return;
            }
        } else if let Uri::Absolute(ref abs) = uri {
            if let Some(auth) = abs.authority() {
                if auth.port().is_some() {
                    return;
                }
            }
        }

        let string = ::std::str::from_utf8(data).expect("parsed UTF-8");
        assert_eq!(string.to_string(), uri.to_string());
    }
});
