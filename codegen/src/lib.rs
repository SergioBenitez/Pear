#![recursion_limit="256"]

#![cfg_attr(parse_nightly, feature(proc_macro_diagnostic, proc_macro_span))]

extern crate proc_macro;
extern crate proc_macro2;
extern crate syn;
#[macro_use] extern crate quote;

mod parser;
mod diagnostics;

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use syn::visit_mut::{self, VisitMut};

use crate::diagnostics::{Diagnostic, Spanned, SpanExt};
use crate::parser::*;

fn parse_mark_ident(span: proc_macro2::Span) -> syn::Ident {
    const PARSE_MARK_IDENT: &'static str = "____parse_parse_mark";

    syn::Ident::new(PARSE_MARK_IDENT, span)
}

#[derive(Copy, Clone)]
enum State {
    Start,
    InTry
}

struct ParserTransformer {
    input: syn::Expr,
    name: syn::Ident,
    state: State,
}

impl VisitMut for ParserTransformer {
    fn visit_expr_try_mut(&mut self, v: &mut syn::ExprTry) {
        let last_state = self.state;
        self.state = State::InTry;
        visit_mut::visit_expr_try_mut(self, v);
        self.state = last_state;
    }

    fn visit_expr_call_mut(&mut self, call: &mut syn::ExprCall) {
        if let State::InTry = self.state {
            // TODO: Should we keep recursing?
            call.args.insert(0, self.input.clone());
        } else {
            visit_mut::visit_expr_call_mut(self, call);
        }
    }

    fn visit_macro_mut(&mut self, m: &mut syn::Macro) {
        if let Some(ref segment) = m.path.segments.last() {
            let name = segment.value().ident.to_string();
            let (input, parser_name) = (&self.input, &self.name);
            m.tts = if name == "parse_mark" || name == "parse_context" {
                let mark = parse_mark_ident(self.input.span());
                quote!([#parser_name; #input] #mark)
            } else if name == "switch" || name.starts_with("parse_") {
                let tokens = &m.tts;
                quote!([#parser_name; #input] #tokens)
            } else {
                return
            };
        }
    }
}

fn extract_input_ident(f: &syn::ItemFn) -> PResult<syn::Ident> {
    use syn::{FnArg::Captured, ArgCaptured, Pat::Ident};

    let first = f.decl.inputs.first().ok_or_else(|| {
        let paren_span = f.decl.paren_token.span;
        paren_span.error("parsing functions require at least one input")
    })?;

    match first.value() {
        Captured(ArgCaptured { pat: Ident(pat), .. }) => Ok(pat.ident.clone()),
        _ => Err(first.span().error("invalid type for parser input"))
    }
}

fn wrapping_fn_block(
    function: &syn::ItemFn,
    scope: TokenStream2,
    raw: bool,
) -> PResult<syn::Block> {
    let input_ident = extract_input_ident(&function)?;
    let fn_block = &function.block;
    let ret_ty = match &function.decl.output {
        syn::ReturnType::Default => quote!(()),
        syn::ReturnType::Type(_, ty) => quote!(#ty),
    };

    let mark_ident = parse_mark_ident(input_ident.span());
    let result_map = if raw {
        quote!((|#mark_ident| #fn_block))
    } else {
        quote!((|#mark_ident| #scope::result::AsResult::as_result(#fn_block)))
    };

    let new_block_tokens = {
        let name = &function.ident;
        let name_str = name.to_string();
        quote!({
            let info = #scope::input::ParserInfo {
                name: #name_str,
                raw: #raw,
            };

            // FIXME: Get rid of this!
            if #scope::macros::is_parse_debug!() {
                #scope::debug::parser_entry(&info);
            }

            let marker = #scope::input::Input::mark(#input_ident, &info);
            let mut result: #ret_ty = #result_map(marker.as_ref());
            if let Err(ref mut error) = result {
                let ctxt = #scope::input::Input::context(#input_ident, marker.as_ref());
                error.push_context(ctxt, info);
            }

            // FIXME: Get rid of this!
            if #scope::macros::is_parse_debug!() {
                let ctxt = #scope::input::Input::context(#input_ident, marker.as_ref());
                let string = ctxt.map(|c| c.to_string());
                #scope::debug::parser_exit(&info, result.is_ok(), string);
            }

            #scope::input::Input::unmark(#input_ident, &info, result.is_ok(), marker);
            result
        })
    };

    syn::parse(new_block_tokens.into())
        .map_err(|e| function.span().error(format!("bad function: {}", e)).into())
}

// FIXME: Add the now missing `inline` optimization.
fn parser_attribute(input: TokenStream, is_raw: bool) -> PResult<TokenStream2> {
    let input: proc_macro2::TokenStream = input.into();
    let span = input.span();
    let mut function: syn::ItemFn = syn::parse2(input).map_err(|_| {
        span.error("`parser` attribute only supports functions")
    })?;

    if !is_raw {
        let input_ident = extract_input_ident(&function)?;
        let input_expr = syn::Expr::Path(syn::ExprPath {
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
    }

    let scope = match is_raw { true => quote!(crate), false => quote!(::pear) };
    function.block = Box::new(wrapping_fn_block(&function, scope, is_raw)?);

    Ok(quote!(#function))
}

#[proc_macro_attribute]
pub fn parser(args: TokenStream, input: TokenStream) -> TokenStream {
    let args: proc_macro2::TokenStream = args.into();
    let args_span = args.span();
    let is_raw = match syn::parse2::<syn::Ident>(args).ok() {
        Some(ref ident) if ident == "raw" => true,
        Some(_) => return args_span.error("unsupported arguments").emit_as_tokens(),
        None => false,
    };

    match parser_attribute(input, is_raw) {
        Ok(tokens) => tokens.into(),
        Err(diag) => diag.emit_as_tokens(),
    }
}

impl Case {
    fn to_tokens<'a, I>(
        input: &syn::Expr,
        parser_name: &syn::Ident,
        mut cases: I
    ) -> TokenStream2
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
                        .unwrap_or(syn::Ident::new("_", call.span()))
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
        Err(e) => Diagnostic::from(e).emit_as_tokens(),
    }
}
