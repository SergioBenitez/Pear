#![feature(core_intrinsics)]
#![feature(proc_macro_diagnostic, proc_macro_span)]
#![recursion_limit="256"]

extern crate proc_macro;
extern crate proc_macro2;
extern crate syn;
#[macro_use] extern crate quote;

mod parser;
mod spanned;

use parser::Parser;
use spanned::Spanned;

use proc_macro2::TokenStream as TokenStream2;
use proc_macro::{TokenStream, Span, Diagnostic};
use syn::visit_mut::{self, VisitMut};
use syn::*;

type PResult<T> = Result<T, Diagnostic>;

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
        let paren_span = f.decl.paren_token.0.unstable();
        paren_span.error("parsing functions require at least one input")
    })?;

    match first.value() {
        FnArg::Captured(ArgCaptured { pat: Pat::Ident(pat), .. }) => Ok(pat.ident.clone()),
        _ => Err(first.span().error("invalid type for parser input"))
    }
}

// fn ty_from(ret_ty: &ReturnType) -> Box<Type> {
//     match *ret_ty {
//         ReturnType::Type(_, ref ty) => ty.clone(),
//         _ => Box::new(syn::parse2(quote!(()).into()).unwrap())
//     }
// }

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

    let new_block_tokens = {
        let fn_block = &function.block;
        let name = &function.ident;
        let name_str = name.to_string();
        quote!({
            #[allow(unused_imports)]
            use ::pear::{Input, Length};

            if ::pear::is_debug!() {
                let ctxt = #input_ident.context().map(|c| c.to_string());
                ::pear::parser_entry(#name_str, ctxt);
            }

            let result = (|| ::pear::AsResult::as_result(#fn_block))();

            if ::pear::is_debug!() {
                let success = result.is_ok();
                let ctxt = #input_ident.context().map(|c| c.to_string());
                ::pear::parser_exit(#name_str, success, ctxt);
            }

            result
        })
    };

    let new_block = syn::parse(new_block_tokens.into()).unwrap();
    function.block = Box::new(new_block);

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

#[derive(Debug)]
struct CallPattern {
    name: Option<Ident>,
    expr: ExprCall,
    span: Span,
}

#[derive(Debug)]
enum PatternKind {
    Wild,
    Calls(Vec<CallPattern>)
}

#[derive(Debug)]
struct Pattern {
    kind: PatternKind,
    span: Span,
}

impl Pattern {
    fn validate(&self) -> parser::Result<()> {
        let mut prev = None;
        if let PatternKind::Calls(ref calls) = self.kind {
            for call in calls.iter() {
                if prev.is_none() { prev = Some(call.name.clone()); }

                let prev_name = prev.as_ref().unwrap();
                if prev_name != &call.name {
                    let mut err = if let Some(ref ident) = call.name {
                        ident.span().unstable()
                            .error("captured name differs from declaration")
                    } else {
                        call.expr.span()
                            .error("expected capture name due to previous declaration")
                    };

                    err = match prev_name {
                        Some(p) => err.span_note(p.span().unstable(), "declared here"),
                        None => err
                    };

                    return Err(err);
                }
            }
        }

        Ok(())
    }
}

#[derive(Debug)]
struct Case {
    pattern: Pattern,
    expr: Expr,
    span: Span,
}

#[derive(Debug)]
struct Switch {
    parser_name: Ident,
    input: Expr,
    cases: Vec<Case>
}

fn parse_expr_call(parser: &mut Parser) -> parser::Result<ExprCall> {
    use parser::Delimiter::*;
    use syn::{punctuated::Punctuated, token::{Comma, Paren}};

    let path: ExprPath = parser.parse()?;
    let paren_span = parser.current_span();
    let args = parser.parse_group(Parenthesis, |p| {
        p.parse_synom("call params", <Punctuated<Expr, Comma>>::parse_terminated)
    })?;

    Ok(ExprCall {
        attrs: vec![],
        func: Box::new(Expr::Path(path)),
        paren_token: Paren(paren_span.into()),
        args: args
    })
}

fn parse_switch(input: TokenStream) -> Result<Switch, Diagnostic> {
    use parser::{Seperator::*, Delimiter::*};

    let mut parser = Parser::new(input);
    let (parser_name, input) = parser.parse_group(Bracket, |p| {
        let name = p.parse::<Ident>()?;
        p.parse::<token::Semi>()?;
        let input = p.parse::<Expr>()?;
        Ok((name, input))
    })?;

    let cases: Vec<Case> = parser.parse_sep(Comma, |parser| {
        let case_span_start = parser.current_span();

        let pattern = if parser.eat::<PatWild>() {
            Pattern {
                kind: PatternKind::Wild,
                span: case_span_start
            }
        } else {
            let call_patterns = parser.parse_sep(Pipe, |parser| {
                let start_span = parser.current_span();
                let name = parser.try_parse(|p| {
                    let ident = p.parse::<Ident>()?;
                    p.parse::<token::At>()?;
                    Ok(ident)
                }).ok();

                let expr = parse_expr_call(parser)?;
                let span = start_span.join(parser.current_span()).unwrap();
                Ok(CallPattern { name, expr, span })
            })?;

            Pattern {
                kind: PatternKind::Calls(call_patterns),
                span: case_span_start.join(parser.current_span()).unwrap()
            }
        };

        pattern.validate()?;
        parser.parse::<token::FatArrow>()?;
        let expr: Expr = parser.parse()?;
        let span = case_span_start.join(parser.current_span()).unwrap();

        Ok(Case { pattern, expr, span })
    })?;

    if !parser.is_eof() {
        parser.current_span()
            .error("trailing characters; expected eof")
            .help("perhaps a comma `,` is missing?")
            .emit();
    }

    for (i, case) in cases.iter().enumerate() {
        if let PatternKind::Wild = case.pattern.kind {
            if i != cases.len() - 1 {
                return Err(case.span.error("`_` matches can only appear as the last case"));
            }
        }

    }

    Ok(Switch { parser_name, input, cases })
}

impl Case {
    fn to_tokens(input: &Expr, parser_name: &Ident, cases: &[Case]) -> TokenStream2 {
        if cases.len() == 0 {
            // FIXME: Should we allow this? What should we do if we get here?
            return quote!(panic!("THIS IS THE CASE WHERE THERE'S NO _"));
        }

        let (this, rest) = (&cases[0], &cases[1..]);

        let mut transformer = ParserTransformer {
            input: input.clone(),
            name: parser_name.clone(),
            state: State::Start
        };

        let mut case_expr = this.expr.clone();
        visit_mut::visit_expr_mut(&mut transformer, &mut case_expr);

        match this.pattern.kind {
            PatternKind::Wild => quote!(#case_expr),
            PatternKind::Calls(ref calls) => {
                let prefix = (0..calls.len()).into_iter().map(|i| {
                    match i {
                        0 => quote!(if),
                        _ => quote!(else if)
                    }
                });

                let name = calls.iter().map(|call| {
                    call.name.as_ref()
                        .map(|c| c.clone())
                        .unwrap_or(Ident::new("_", call.span.into()))
                });

                // FIXME: We're repeating ourselves, aren't we? We alrady do
                // this in the visitor.
                let call_expr = calls.iter().map(|call| {
                    let mut call = call.expr.clone();
                    call.args.insert(0, input.clone());
                    call
                });

                let case_expr = ::std::iter::repeat(&case_expr);
                let rest_tokens = Case::to_tokens(&input, parser_name, rest);

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
        Case::to_tokens(&self.input, &self.parser_name, &self.cases)
    }
}

#[proc_macro]
pub fn switch(input: TokenStream) -> TokenStream {
    match parse_switch(input) {
        Ok(switch) => switch.to_tokens().into(),
        Err(diag) => {
            diag.emit();
            TokenStream::new()
        }
    }
}
