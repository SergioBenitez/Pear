use proc_macro::Span;

use syn::Token;
use syn::punctuated::Punctuated;
use syn::parse::{Parse as SynParse, ParseStream as SynParseStream};
use proc_macro2::Delimiter;
use spanned::Spanned;

#[derive(Debug)]
pub(crate) struct Diagnostic(crate ::proc_macro::Diagnostic);

impl Diagnostic {
    pub fn emit(self) {
        self.0.emit();
    }
}

impl From<::proc_macro::Diagnostic> for Diagnostic {
    fn from(original: ::proc_macro::Diagnostic) -> Diagnostic {
        Diagnostic(original)
    }
}

impl From<::syn::parse::Error> for Diagnostic {
    fn from(e: ::syn::parse::Error) -> Diagnostic {
        let inner = ::proc_macro::Diagnostic::spanned(
            e.span().unstable(), ::proc_macro::Level::Error, e.to_string());
        Diagnostic(inner)
    }
}

impl Into<::syn::parse::Error> for Diagnostic {
    fn into(self) -> ::syn::parse::Error {
        let span = if self.0.spans().is_empty() {
            Span::call_site()
        } else {
            self.0.spans()[0]
        };

        ::syn::parse::Error::new(span.into(), self.0.message())
    }
}

impl ::std::ops::Deref for Diagnostic {
    type Target = ::proc_macro::Diagnostic;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub(crate) type PResult<T> = Result<T, Diagnostic>;

pub(crate) trait Parse: Sized {
    fn parse(input: syn::parse::ParseStream) -> PResult<Self>;

    fn syn_parse(input: syn::parse::ParseStream) -> syn::parse::Result<Self> {
        Self::parse(input).map_err(|e| e.into())
    }
}

trait ParseStreamExt {
    fn parse_group<F, G>(self, delimiter: Delimiter, parser: F) -> syn::parse::Result<G>
        where F: FnOnce(SynParseStream) -> syn::parse::Result<G>;

    fn try_parse<F, G>(self, parser: F) -> syn::parse::Result<G>
        where F: Fn(SynParseStream) -> syn::parse::Result<G>;
}

impl<'a> ParseStreamExt for SynParseStream<'a> {
    fn parse_group<F, G>(self, delimiter: Delimiter, parser: F) -> syn::parse::Result<G>
        where F: FnOnce(SynParseStream) -> syn::parse::Result<G>
    {
        let content;
        match delimiter {
            Delimiter::Brace => { syn::braced!(content in self); },
            Delimiter::Bracket => { syn::bracketed!(content in self); },
            Delimiter::Parenthesis => { syn::parenthesized!(content in self); },
            Delimiter::None => return parser(self),
        }

        parser(&content)
    }

    fn try_parse<F, G>(self, parser: F) -> syn::parse::Result<G>
        where F: Fn(SynParseStream) -> syn::parse::Result<G>
    {
        let input = self.fork();
        parser(&input)?;
        parser(self)
    }
}

#[derive(Debug)]
pub(crate) struct CallPattern {
    pub(crate) name: Option<syn::Ident>,
    pub(crate) expr: syn::ExprCall,
}

impl syn::parse::Parse for CallPattern {
    fn parse(input: syn::parse::ParseStream) -> syn::parse::Result<Self> {
        Self::syn_parse(input)
    }
}

impl Spanned for CallPattern {
    fn span(&self) -> Span {
        self.name.as_ref()
            .and_then(|name| name.span().unstable().join(self.expr.span()))
            .unwrap_or_else(|| self.expr.span())
    }
}

#[derive(Debug)]
pub(crate) enum Pattern {
    Wild(Token![_]),
    Calls(Punctuated<CallPattern, Token![|]>)
}

#[derive(Debug)]
pub(crate) struct Case {
    pub(crate) pattern: Pattern,
    pub(crate) expr: syn::Expr,
    pub(crate) span: Span,
}

#[derive(Debug)]
pub(crate) struct Switch {
    pub(crate) parser_name: syn::Ident,
    pub(crate) input: syn::Expr,
    pub(crate) cases: Punctuated<Case, Token![,]>
}

// FIXME(syn): Something like this should be in `syn`
fn parse_expr_call(input: SynParseStream) -> syn::parse::Result<syn::ExprCall> {
    let path: syn::ExprPath = input.parse()?;
    let paren_span = input.cursor().span();
    let args: Punctuated<syn::Expr, Token![,]> = input.parse_group(Delimiter::Parenthesis, |i| {
        i.parse_terminated(syn::Expr::parse)
    })?;

    Ok(syn::ExprCall {
        attrs: vec![],
        func: Box::new(syn::Expr::Path(path)),
        paren_token: syn::token::Paren(paren_span),
        args: args
    })
}

impl Parse for CallPattern {
    fn parse(input: SynParseStream) -> PResult<Self> {
        let name = input.try_parse(|input| {
            let ident: syn::Ident = input.parse()?;
            input.parse::<Token![@]>()?;
            Ok(ident)
        }).ok();

        Ok(CallPattern { name, expr: parse_expr_call(input)? })
    }
}

impl Pattern {
    fn validate(&self) -> PResult<()> {
        let mut prev = None;
        if let Pattern::Calls(ref calls) = self {
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

                    return Err(err.into());
                }
            }
        }

        Ok(())
    }
}

impl Parse for Case {
    fn parse(input: SynParseStream) -> PResult<Self> {
        let case_span_start = input.cursor().span().unstable();

        let pattern = if let Ok(wild) = input.parse::<Token![_]>() {
            Pattern::Wild(wild)
        } else {
            let call_patterns =
                input.call(<Punctuated<CallPattern, Token![|]>>::parse_separated_nonempty)?;

            Pattern::Calls(call_patterns)
        };

        pattern.validate()?;
        input.parse::<Token![=>]>()?;
        let expr: syn::Expr = input.parse()?;
        let span = case_span_start
            .join(input.cursor().span().unstable())
            .unwrap_or(case_span_start);

        Ok(Case { pattern, expr, span })
    }
}

impl Parse for Switch {
    fn parse(stream: SynParseStream) -> PResult<Switch> {
        let (parser_name, input) = stream.parse_group(Delimiter::Bracket, |inner| {
            let name: syn::Ident = inner.parse()?;
            inner.parse::<Token![;]>()?;
            let input: syn::Expr = inner.parse()?;
            Ok((name, input))
        })?;

        let cases: Punctuated<Case, Token![,]> = stream.parse_terminated(Case::syn_parse)?;
        if !stream.is_empty() {
            Err(stream.error("trailing characters; expected eof"))?;
        }

        if cases.is_empty() {
            Err(stream.error("switch cannot be empty"))?;
        }

        for case in cases.iter().take(cases.len() - 1) {
            if let Pattern::Wild(..) = case.pattern {
                Err(case.span.error("`_` matches can only appear as the last case"))?;
            }
        }

        Ok(Switch { parser_name, input, cases })
    }
}
