#[macro_use] extern crate quote;
extern crate syn;
extern crate proc_macro;

use proc_macro::TokenStream;
use syn::{Expr, ExprKind, Stmt};
use syn::parse::IResult;
use syn::visit::{walk_stmt, Visitor};
use quote::Tokens;

const PREFIXES: &'static [&'static str] = &[
    "#[allow(unused)]", "enum", "DummyEnumForProcMacros", "{", "Input", "=", "(",
    "stringify!", "("
];

const SUFFIX: &'static str = "), 0).1,}";

fn source_to_real_input(original: &str) -> String {
    let source: String = original.lines().map(|line| line.trim()).collect();
    let mut source = source.trim_left();
    for prefix in PREFIXES.iter() {
        if source.starts_with(prefix) {
            source = source[prefix.len()..].trim_left();
        } else {
            panic!("unexpected macro input: {:?}", original);
        }
    }

    if !source.ends_with(SUFFIX) {
        panic!("bad suffix: {:?}", original);
    }

    source[..(source.len() - SUFFIX.len())].into()
}

#[proc_macro_derive(nosh_parse)]
pub fn nosh_parse_prelude(input: TokenStream) -> TokenStream {
    let raw_input = source_to_real_input(&input.to_string());
    let (mut input, input_expr) = match syn::parse::expr(&raw_input) {
        IResult::Done(rest, expr) => (rest.trim_left(), expr),
        IResult::Error => panic!("expected an expression at the start of `parse!`")
    };

    if input.starts_with(',') {
        input = input[1..].trim_left();
    } else {
        panic!("expected a comma after the input expression");
    }

    let expr = syn::parse_expr(&input).unwrap();
    let statements = match expr.node {
        ExprKind::Block(_, block) => block.stmts,
        _ => panic!("expected a block after input ident and comma")
    };

    let output = nosh_parse_macro(input_expr, statements);
    quote!(
        macro_rules! get_expr {
            () => { #output }
        }
    ).parse().unwrap()
}

struct ParseMacroVisitor {
    result: Tokens
}

use syn::{Local, Item, Mac};

impl Visitor for ParseMacroVisitor {
    fn visit_local(&mut self, local: &Local) {
        println!("LOCAL: {:?}", local);
        self.result.append_all(&[local]);
    }

    fn visit_item(&mut self, item: &Item) {
        println!("ITEM: {:?}", item);
        self.result.append_all(&[item]);
    }

    fn visit_expr(&mut self, expr: &Expr) {
        println!("EXPR: {:?}", expr);
        self.result.append_all(&[expr]);
    }

    fn visit_mac(&mut self, mac: &Mac) {
        println!("MAC: {:?}", mac);
        self.result.append_all(&[mac]);
    }
}

fn nosh_parse_macro(expr: Expr, statements: Vec<Stmt>) -> Tokens {
    let mut visitor = ParseMacroVisitor { result: Tokens::new() };
    for statement in statements {
        walk_stmt(&mut visitor, &statement);
    }

    if visitor.result.as_str().is_empty() {
        panic!("the parse! macro cannot be empty")
    }

    let result = visitor.result;
    quote! { { #result } }
    // let mut tokens = Tokens::new();
    // tokens.append("12");
    // tokens
}

// macro_rules! chain {
//     (input = $input:expr; $($tokens:tt)*) => ({
//         println!("Beginning new parse for: {:?}", $input);
//         chain!([$input] $($tokens)*)
//     });

//     ([$input:expr] let mut $var:ident; $($tokens:tt)*) => ({
//         let mut $var;
//         chain!([$input] $($tokens)*)
//     });

//     ([$input:expr] let mut $var:ident = $value:expr; $($tokens:tt)*) => ({
//         let mut $var = $value;
//         chain!([$input] $($tokens)*)
//     });

//     ([$input:expr] $parser:ident($($fn_params:tt)*); $($tokens:tt)*) => ({
//         println!("Parsing [{}]: {:?}", stringify!($parser($($fn_params)*)), $input);
//         chain!(@call [$input] _ignored = $parser($($fn_params)*); $($tokens)*)
//     });

//     ([$input:expr] let $var:pat = $parser:ident($($fn_params:tt)*); $($tokens:tt)*) => ({
//         println!("Parsing [{}]: {:?}", stringify!($parser($($fn_params)*)), $input);
//         chain!(@call [$input] $var = $parser($($fn_params)*); $($tokens)*)
//     });

//     ([$input:expr] let $var:pat = $parser:ident! { $($inner:tt)* }; $($tokens:tt)*) => ({
//         match $parser!([$input] $($inner)*) {
//             Done(next_input, $var) => {
//                 chain!([next_input] $($tokens)*)
//             }
//             Error(e) => Error(e)
//         }
//     });

//     (@call [$input:expr] $output:pat = $parser:ident($($fn_params:tt)*); $($tokens:tt)*) => ({
//         match $parser($input, $($fn_params)*) {
//             Done(next_input, $output) => {
//                 chain!([next_input] $($tokens)*)
//             }
//             Error(e) => Error(e)
//         }
//     });

//     ([$input:expr] $parser:ident($($fn_params:tt)*)) => ({
//         println!("Parsing [{}]: {:?}", stringify!($parser($($fn_params)*)), $input);
//         $parser($input, $($fn_params)*)
//     });

//     ([$input:expr] output = $output:expr) => ({
//         println!("Done! Remaining input: {:?}", $input);
//         Done($input, $output)
//     });

//     ([$input:expr] output = try $output:expr, else $err:expr) => ({
//         println!("Done! Remaining input: {:?}", $input);
//         match $output {
//             Ok(output) => Done($input, output),
//             Err(_) => Error($err)
//         }
//     });
// }

