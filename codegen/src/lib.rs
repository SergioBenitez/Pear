#![recursion_limit="256"]

#![cfg_attr(pear_nightly, feature(proc_macro_diagnostic, proc_macro_span))]

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

fn parse_marker_ident(span: proc_macro2::Span) -> syn::Ident {
    const PARSE_MARKER_IDENT: &'static str = "____parse_parse_marker";
    syn::Ident::new(PARSE_MARKER_IDENT, span)
}

fn parser_info_ident(span: proc_macro2::Span) -> syn::Ident {
    const PARSE_INFO_IDENT: &'static str = "____parse_parser_info";
    syn::Ident::new(PARSE_INFO_IDENT, span)
}

#[derive(Copy, Clone)]
enum State {
    Start,
    InTry
}

#[derive(Clone)]
struct ParserTransformer {
    input: syn::Expr,
    output: syn::Type,
    state: State,
}

impl ParserTransformer {
    fn new(input: syn::Expr, output: syn::Type) -> ParserTransformer {
        ParserTransformer { input, output, state: State::Start }
    }
}

impl VisitMut for ParserTransformer {
    fn visit_expr_try_mut(&mut self, v: &mut syn::ExprTry) {
        let last_state = self.state;
        self.state = State::InTry;
        visit_mut::visit_expr_try_mut(self, v);
        self.state = last_state;

        let expr = &v.expr;
        let new_expr = quote_spanned!(expr.span() => #expr.map_err(|e| e.into()));
        let method_call: syn::Expr = syn::parse2(new_expr).expect("okay");
        v.expr = Box::new(method_call);
    }

    fn visit_expr_call_mut(&mut self, call: &mut syn::ExprCall) {
        if let State::InTry = self.state {
            // TODO: Should we keep recursing?
            call.args.insert(0, self.input.clone());

            // Only insert into the _first_ call.
            self.state = State::Start;
        } else {
            visit_mut::visit_expr_call_mut(self, call);
        }
    }

    fn visit_macro_mut(&mut self, m: &mut syn::Macro) {
        if let Some(ref segment) = m.path.segments.last() {
            let name = segment.ident.to_string();
            if name == "switch" || name.starts_with("parse_") {
                let (input, output) = (&self.input, &self.output);
                let tokens = match syn::parse2::<syn::Expr>(m.tokens.clone()) {
                    Ok(mut expr) => {
                        let mut transformer = self.clone();
                        transformer.state = State::Start;
                        visit_mut::visit_expr_mut(&mut transformer, &mut expr);
                        quote!(#expr)
                    },
                    Err(_) => m.tokens.clone()
                };

                let info = parser_info_ident(self.input.span());
                let mark = parse_marker_ident(m.span());

                let parser_info = quote!([#info; #input; #mark; #output]);
                m.tokens = quote_spanned!(m.span() => #parser_info #tokens);
            } else {
                return
            }
        }
    }
}

fn extract_input_ident_ty(f: &syn::ItemFn) -> PResult<(syn::Ident, syn::Type)> {
    use syn::{FnArg::Typed, PatType, Pat::Ident, Type::Reference};

    let first = f.sig.inputs.first().ok_or_else(|| {
        let paren_span = f.sig.paren_token.span;
        paren_span.error("parsing functions require at least one input")
    })?;

    let e = first.span().error("invalid type for parser input");
    match first {
        Typed(PatType { pat, ty, .. }) => match **pat {
            Ident(ref p) => match **ty {
                Reference(ref r) => Ok((p.ident.clone(), *r.elem.clone())),
                _ => Err(e)
            }
            _ => Err(e)
        }
        _ => Err(first.span().error("invalid type for parser input"))
    }
}

fn wrapping_fn_block(
    function: &syn::ItemFn,
    scope: TokenStream2,
    args: &AttrArgs,
    ret_ty: &syn::Type,
) -> PResult<syn::Block> {
    let (input_ident, input_ty) = extract_input_ident_ty(&function)?;
    let fn_block = &function.block;

    let span = function.span();
    let mark_ident = parse_marker_ident(input_ident.span());
    let info_ident = parser_info_ident(function.sig.ident.span());
    let result_map = match args.raw.is_some() {
        true => quote_spanned!(span => (
            |#info_ident, #mark_ident: &mut <#input_ty as #scope::input::Input>::Marker| {
                #fn_block
            })
        ),
        false => quote_spanned!(span => (
            |#info_ident, #mark_ident: &mut <#input_ty as #scope::input::Input>::Marker| {
                use #scope::result::AsResult;
                AsResult::as_result(#fn_block)
            }
        ))
    };

    let rewind = args.rewind.map(|span| quote_spanned! { span =>
        <#input_ty as #scope::input::Rewind>::rewind_to(#input_ident, &___mark);
    });

    let peek = args.peek.map(|span| quote_spanned! { span =>
        <#input_ty as #scope::input::Rewind>::rewind_to(#input_ident, &___mark);
    });

    let new_block_tokens = {
        let (name, raw) = (&function.sig.ident, args.raw.is_some());
        let name_str = name.to_string();
        quote_spanned!(span => {
            // FIXME: Get rid of this!
            let ___info = #scope::input::ParserInfo { name: #name_str, raw: #raw };
            if #scope::macros::is_parse_debug!() {
                #scope::debug::parser_entry(&___info);
            }

            let mut ___mark = #scope::input::Input::mark(#input_ident, &___info);
            let mut ___res: #ret_ty = #result_map(&___info, &mut ___mark);
            if let Err(ref mut ___e) = ___res {
                let ___ctxt = #scope::input::Input::context(#input_ident, &___mark);
                ___e.push_context(___ctxt, ___info);
                #rewind
            } else {
                #peek
            }

            // FIXME: Get rid of this!
            if #scope::macros::is_parse_debug!() {
                let ___ctxt = #scope::input::Input::context(#input_ident, &___mark);
                let ___show = ___ctxt.as_ref().map(|c| c as &dyn #scope::input::Show);
                #scope::debug::parser_exit(&___info, ___res.is_ok(), ___show);
            }

            #scope::input::Input::unmark(#input_ident, &___info, ___res.is_ok(), ___mark);
            ___res
        })
    };

    syn::parse(new_block_tokens.into())
        .map_err(|e| function.span().error(format!("bad function: {}", e)).into())
}

// FIXME: Add the now missing `inline` optimization.
fn parser_attribute(input: TokenStream, args: &AttrArgs) -> PResult<TokenStream2> {
    let input: proc_macro2::TokenStream = input.into();
    let span = input.span();
    let mut function: syn::ItemFn = syn::parse2(input).map_err(|_| {
        span.error("`parser` attribute only supports functions")
    })?;

    let ret_ty: syn::Type = match &function.sig.output {
        syn::ReturnType::Default => {
            return Err(function.sig.span().error("parse function requires return type"));
        },
        syn::ReturnType::Type(_, ty) => (**ty).clone(),
    };

    if !args.raw.is_some() {
        let (input_ident, _) = extract_input_ident_ty(&function)?;
        let input_expr = syn::Expr::Path(syn::ExprPath {
            attrs: vec![],
            qself: None,
            path: input_ident.clone().into()
        });

        let mut transformer = ParserTransformer::new(input_expr, ret_ty.clone());
        visit_mut::visit_item_fn_mut(&mut transformer, &mut function);
    }

    let scope = match args.raw.is_some() { true => quote!(crate), false => quote!(::pear) };
    function.block = Box::new(wrapping_fn_block(&function, scope, args, &ret_ty)?);

    Ok(quote!(#function))
}

#[proc_macro_attribute]
pub fn parser(args: TokenStream, input: TokenStream) -> TokenStream {
    use syn::parse::Parser;
    let args = match AttrArgs::syn_parse.parse(args) {
        Ok(args) => args,
        Err(e) => return Diagnostic::from(e).emit_as_tokens(),
    };

    match parser_attribute(input, &args) {
        Ok(tokens) => tokens.into(),
        Err(diag) => diag.emit_as_tokens(),
    }
}

impl Case {
    fn to_tokens<'a, I>(context: &Context, mut cases: I) -> TokenStream2
        where I: Iterator<Item = &'a Case>
    {
        let this = match cases.next() {
            None => return quote!(),
            Some(case) => case
        };

        let (input, output) = (&context.input, &context.output);
        let mut transformer = ParserTransformer::new(input.clone(), output.clone());
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
                let rest_tokens = Case::to_tokens(context, cases);

                quote_spanned! { this.span =>
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
        Case::to_tokens(&self.context, self.cases.iter())
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
