#[macro_use] extern crate nosh;
#[macro_use] extern crate nosh_codegen;

use nosh::Input;
use nosh::parsers::eat;

fn main() {
    let (x, y) = parse!(input, {
        let x = eat(b'a');
        let y = eat(b'b');
        (x, y)
    }).unwrap();

    // let x = get_expr!();
    println!("(x, y) == ({}, {})", x, y);
}

// parse!(input, {
//     let top = take_some_while(|c| is_valid_byte(c) && c != b'/');
//     eat(b'/');
//     let sub = take_some_while(is_valid_byte);

//     // // OWS* ; OWS*
//     let mut params = Vec::new();
//     let _ = try_repeat! {
//         skip_while(is_whitespace);
//         eat(b';');
//         skip_while(is_whitespace);

//         let key = take_some_while(|c| is_valid_byte(c) && c != b'=');
//         eat(b'=');

//         let value = switch! {
//             peek(b'"') => { quoted_string() },
//             _ => { take_some_while(|c| is_valid_byte(c) && c != b';') }
//         };

//         output = params.push((key, value))
//     };

//     output = MediaType {
//         top: top,
//         sub: sub,
//         params: params
//     }
// })

