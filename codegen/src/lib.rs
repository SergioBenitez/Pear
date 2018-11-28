#![feature(crate_visibility_modifier)]
#![feature(proc_macro_diagnostic, proc_macro_span)]
#![recursion_limit="256"]

extern crate proc_macro;
extern crate proc_macro2;
extern crate syn;
#[macro_use] extern crate quote;

mod spanned;
mod parser;

use parser::*;
use spanned::Spanned;

use syn::*;
use proc_macro2::TokenStream as TokenStream2;
use proc_macro::TokenStream;
use syn::visit_mut::{self, VisitMut};

#[derive(Copy, Clone)]
enum State {
    Start,
    InTry
}

struct ParserTransformer {
    input: Expr,
    name: Ident,
    state: State,
}

impl VisitMut for ParserTransformer {
    fn visit_expr_try_mut(&mut self, v: &mut ExprTry) {
        let last_state = self.state;
        self.state = State::InTry;
        visit_mut::visit_expr_try_mut(self, v);
        self.state = last_state;
    }

    fn visit_expr_call_mut(&mut self, call: &mut ExprCall) {
        if let State::InTry = self.state {
            // TODO: Should we keep recursing?
            call.args.insert(0, self.input.clone());
        } else {
            visit_mut::visit_expr_call_mut(self, call);
        }
    }

    fn visit_macro_mut(&mut self, m: &mut Macro) {
        if let Some(ref segment) = m.path.segments.last() {
            let name = segment.value().ident.to_string();
            if name == "switch" || name.starts_with("pear_") {
                let new_stream = {
                    let (input, name, tokens) = (&self.input, &self.name, &m.tts);
                    quote!([#name; #input] #tokens)
                };

                m.tts = new_stream.into();
            }
        }
    }
}

fn extract_input_ident(f: &ItemFn) -> PResult<Ident> {
    let first = f.decl.inputs.first().ok_or_else(|| {
        let paren_span = f.decl.paren_token.span.unstable();
        paren_span.error("parsing functions require at least one input")
    })?;

    match first.value() {
        FnArg::Captured(ArgCaptured { pat: Pat::Ident(pat), .. }) => Ok(pat.ident.clone()),
        _ => Err(first.span().error("invalid type for parser input").into())
    }
}

fn wrapping_fn_block(function: &ItemFn, scope: TokenStream2, raw: bool) -> PResult<Block> {
    let input_ident = extract_input_ident(&function)?;
    let fn_block = &function.block;
    let ret_ty = match &function.decl.output {
        ReturnType::Default => quote!(()),
        ReturnType::Type(_, ty) => quote!(#ty),
    };

    let result_map = if raw {
        quote!((|| #fn_block)())
    } else {
        quote!((|| #scope::AsResult::as_result(#fn_block))())
    };

    let new_block_tokens = {
        let name = &function.ident;
        let name_str = name.to_string();
        quote!({
            #[allow(unused_imports)]
            use #scope::{Input, Length};

            if #scope::is_pear_debug!() {
                let ctxt = #input_ident.context().map(|c| c.to_string());
                #scope::parser_entry(#name_str, ctxt, #raw);
            }

            let result: #ret_ty = #result_map;

            if #scope::is_pear_debug!() {
                let success = result.is_ok();
                let ctxt = #input_ident.context().map(|c| c.to_string());
                #scope::parser_exit(#name_str, success, ctxt, #raw);
            }

            result
        })
    };

    syn::parse(new_block_tokens.into())
        .map_err(|e| function.span().error(format!("bad function: {}", e)).into())
}

// FIXME: Add the now missing `inline` optimization.
fn parser_attribute(input: TokenStream) -> PResult<TokenStream2> {
    let input: proc_macro2::TokenStream = input.into();
    let span = input.span();
    let mut function: ItemFn = syn::parse2(input).map_err(|_| {
        span.error("`parser` attribute only supports functions")
    })?;

    let input_ident = extract_input_ident(&function)?;
    let input_expr = Expr::Path(ExprPath {
        attrs: vec![],
        qself: None,
        path: input_ident.clone().into()
    });

    let mut transformer = ParserTransformer {
        input: input_expr,
        name: function.ident.clone(),
        state: State::Start
    };

    visit_mut::visit_item_fn_mut(&mut transformer, &mut function);
    function.block = Box::new(wrapping_fn_block(&function, quote!(::pear), false)?);

    Ok(quote!(#function))
}

#[proc_macro_attribute]
pub fn parser(_args: TokenStream, input: TokenStream) -> TokenStream {
    match parser_attribute(input) {
        Ok(tokens) => tokens.into(),
        Err(diag) => {
            diag.emit();
            TokenStream::new()
        }
    }
}

fn raw_parser_attribute(input: TokenStream) -> PResult<TokenStream2> {
    let input: proc_macro2::TokenStream = input.into();
    let span = input.span();
    let mut function: ItemFn = syn::parse2(input).map_err(|_| {
        span.error("`raw_parser` attribute only supports functions")
    })?;

    function.block = Box::new(wrapping_fn_block(&function, quote!(crate), true)?);

    Ok(quote!(#function))
}

#[proc_macro_attribute]
pub fn raw_parser(_args: TokenStream, input: TokenStream) -> TokenStream {
    match raw_parser_attribute(input) {
        Ok(tokens) => tokens.into(),
        Err(diag) => {
            diag.emit();
            TokenStream::new()
        }
    }
}


impl Case {
    fn to_tokens<'a, I>(input: &Expr, parser_name: &Ident, mut cases: I) -> TokenStream2
        where I: Iterator<Item = &'a Case>
    {
        let this = match cases.next() {
            None => return quote!(),
            Some(case) => case
        };

        let mut transformer = ParserTransformer {
            input: input.clone(),
            name: parser_name.clone(),
            state: State::Start
        };

        let mut case_expr = this.expr.clone();
        visit_mut::visit_expr_mut(&mut transformer, &mut case_expr);

        match this.pattern {
            Pattern::Wild(..) => quote!(#case_expr),
            Pattern::Calls(ref calls) => {
                let prefix = (0..calls.len()).into_iter().map(|i| {
                    match i {
                        0 => quote!(if),
                        _ => quote!(else if)
                    }
                });

                let name = calls.iter().map(|call| {
                    call.name.as_ref()
                        .map(|c| c.clone())
                        .unwrap_or(Ident::new("_", call.span().into()))
                });

                // FIXME: We're repeating ourselves, aren't we? We alrady do
                // this in the visitor.
                let call_expr = calls.iter().map(|call| {
                    let mut call = call.expr.clone();
                    call.args.insert(0, input.clone());
                    call
                });

                let case_expr = ::std::iter::repeat(&case_expr);
                let rest_tokens = Case::to_tokens(&input, parser_name, cases);

                quote! {
                    #(
                        #prefix let Ok(#name) = #call_expr {
                            #case_expr
                        }
                     )* else {
                        #rest_tokens
                    }
                }
            }
        }
    }
}

impl Switch {
    fn to_tokens(&self) -> TokenStream2 {
        Case::to_tokens(&self.input, &self.parser_name, self.cases.iter())
    }
}

#[proc_macro]
pub fn switch(input: TokenStream) -> TokenStream {
    // TODO: We lose diagnostic information by using syn's thing here. We need a
    // way to get a SynParseStream from a TokenStream to not do that.
    use syn::parse::Parser;
    match Switch::syn_parse.parse(input) {
        Ok(switch) => switch.to_tokens().into(),
        Err(e) => {
            Diagnostic::emit(e.into());
            TokenStream::new()
        }
    }
}
