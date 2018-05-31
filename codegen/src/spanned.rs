use proc_macro::Span;

use proc_macro2::TokenStream as TokenStream2;
use quote::ToTokens;

pub trait Spanned {
    fn span(&self) -> Span;
}

// FIXME: Remove this once proc_macro's stabilize.
impl<T: ToTokens> Spanned for T {
    fn span(&self) -> Span {
        let mut tokens = TokenStream2::new();
        self.to_tokens(&mut tokens);
        let mut iter = tokens.into_iter();
        let mut span = match iter.next() {
            Some(tt) => tt.span().unstable(),
            None => {
                return Span::call_site();
            }
        };

        for tt in iter {
            if let Some(joined) = span.join(tt.span().unstable()) {
                span = joined;
            }
        }

        span
    }
}
