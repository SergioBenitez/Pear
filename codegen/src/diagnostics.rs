use proc_macro2::Span;

pub trait SpanExt {
    fn error<T: Into<String>>(self, message: T) -> Diagnostic;
    fn warning<T: Into<String>>(self, message: T) -> Diagnostic;
    fn note<T: Into<String>>(self, message: T) -> Diagnostic;
    fn help<T: Into<String>>(self, message: T) -> Diagnostic;
    fn join(&self, other: Span) -> Option<Span>;
}

mod private {
    pub trait Sealed {}
    impl<T: quote::ToTokens> Sealed for T {}
}

pub trait Spanned: private::Sealed {
    fn span(&self) -> Span;
}

#[allow(dead_code)]
#[cfg(parse_nightly)]
mod imp {
    use super::{Span, Spanned, SpanExt};

    #[derive(Debug)]
    pub struct Diagnostic(proc_macro::Diagnostic);

    macro_rules! span_ext_method {
        ($name:ident) => (
            fn $name<T: Into<String>>(self, message: T) -> Diagnostic {
                Diagnostic(self.unstable().$name(message))
            }
        )
    }

    impl SpanExt for proc_macro2::Span {
        span_ext_method!(error);
        span_ext_method!(warning);
        span_ext_method!(note);
        span_ext_method!(help);

        fn join(&self, other: Span) -> Option<Span> {
            self.unstable().join(other.unstable()).map(|span| span.into())
        }
    }

    macro_rules! diagnostic_child_methods {
        ($spanned:ident, $regular:ident, $level:expr) => (
            pub fn $spanned<S, T>(self, spans: S, message: T) -> Diagnostic
                where S: MultiSpan, T: Into<String>
            {
                let inner = self.0;
                let spans = spans.into_spans();
                Diagnostic(inner.$spanned(spans, message))
            }

            /// Adds a new child diagnostic message to `self` with the level
            /// identified by this method's name with the given `message`.
            pub fn $regular<T: Into<String>>(self, message: T) -> Diagnostic {
                let inner = self.0;
                Diagnostic(inner.$regular(message))
            }
        )
    }

    /// Trait implemented by types that can be converted into a set of `Span`s.
    pub trait MultiSpan {
        /// Converts `self` into a `Vec<Span>`.
        fn into_spans(self) -> Vec<proc_macro::Span>;
    }

    impl MultiSpan for Span {
        fn into_spans(self) -> Vec<proc_macro::Span> { vec![self.unstable()] }
    }

    impl Diagnostic {
        diagnostic_child_methods!(span_error, error, Level::Error);
        diagnostic_child_methods!(span_warning, warning, Level::Warning);
        diagnostic_child_methods!(span_note, note, Level::Note);
        diagnostic_child_methods!(span_help, help, Level::Help);

        pub fn emit_as_tokens(self) -> proc_macro::TokenStream {
            self.0.emit();
            proc_macro::TokenStream::new()
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
                proc_macro::Span::call_site()
            } else {
                self.0.spans()[0]
            };

            ::syn::parse::Error::new(span.into(), self.0.message())
        }
    }

    impl<T: quote::ToTokens> Spanned for T {
        fn span(&self) -> Span {
            let mut tokens = proc_macro2::TokenStream::new();
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

            span.into()
        }
    }
}

#[allow(dead_code)]
#[cfg(not(parse_nightly))]
mod imp {
    use super::{Span, Spanned, SpanExt};

    /// An enum representing a diagnostic level.
    #[derive(Copy, Clone, Debug)]
    pub enum Level {
        /// An error.
        Error,
        /// A warning.
        Warning,
        /// A note.
        Note,
        /// A help message.
        Help,
        #[doc(hidden)]
        __NonExhaustive
    }

    /// A structure representing a diagnostic message and associated children
    /// messages.
    #[derive(Clone, Debug)]
    pub struct Diagnostic {
        level: Level,
        message: String,
        spans: Vec<Span>,
        children: Vec<Diagnostic>
    }

    /// Trait implemented by types that can be converted into a set of `Span`s.
    pub trait MultiSpan {
        /// Converts `self` into a `Vec<Span>`.
        fn into_spans(self) -> Vec<Span>;
    }

    impl MultiSpan for Span {
        fn into_spans(self) -> Vec<Span> { vec![self] }
    }

    macro_rules! diagnostic_child_methods {
        ($spanned:ident, $regular:ident, $level:expr) => (
            /// Adds a new child diagnostic message to `self` with the level
            /// identified by this method's name with the given `spans` and
            /// `message`.
            pub fn $spanned<S, T>(mut self, spans: S, message: T) -> Diagnostic
                where S: MultiSpan, T: Into<String>
            {
                self.children.push(Diagnostic::spanned(spans, $level, message));
                self
            }

            /// Adds a new child diagnostic message to `self` with the level
            /// identified by this method's name with the given `message`.
            pub fn $regular<T: Into<String>>(mut self, message: T) -> Diagnostic {
                self.children.push(Diagnostic::new($level, message));
                self
            }
        )
    }

    impl Diagnostic {
        /// Creates a new diagno, Spannedstic with the given `level` and `message`.
        pub fn new<T: Into<String>>(level: Level, message: T) -> Diagnostic {
            Diagnostic {
                level: level,
                message: message.into(),
                spans: vec![],
                children: vec![]
            }
        }

        /// Creates a new diagnostic with the given `level` and `message` pointing to
        /// the given set of `spans`.
        pub fn spanned<S, T>(spans: S, level: Level, message: T) -> Diagnostic
            where S: MultiSpan, T: Into<String>
        {
            Diagnostic {
                level: level,
                message: message.into(),
                spans: spans.into_spans(),
                children: vec![]
            }
        }

        diagnostic_child_methods!(span_error, error, Level::Error);
        diagnostic_child_methods!(span_warning, warning, Level::Warning);
        diagnostic_child_methods!(span_note, note, Level::Note);
        diagnostic_child_methods!(span_help, help, Level::Help);

        /// Emit the diagnostic.
        pub fn emit_as_tokens(self) -> proc_macro::TokenStream {
            // FIXME: Probably consider more than the first error.
            let syn_error: syn::parse::Error = self.into();
            syn_error.to_compile_error().into()
        }
    }

    macro_rules! diagnostic_method {
        ($name:ident, $level:expr) => (
            /// Creates a new `Diagnostic` with the given `message` at the span
            /// `self`.
            fn $name<T: Into<String>>(self, message: T) -> Diagnostic {
                Diagnostic::spanned(self, $level, message)
            }
        )
    }

    impl SpanExt for Span {
        diagnostic_method!(error, Level::Error);
        diagnostic_method!(warning, Level::Warning);
        diagnostic_method!(note, Level::Note);
        diagnostic_method!(help, Level::Help);

        fn join(&self, _other: Span) -> Option<Span> {
            Some(self.clone())
        }
    }

    impl From<::syn::parse::Error> for Diagnostic {
        fn from(e: ::syn::parse::Error) -> Diagnostic {
            Diagnostic::spanned(e.span(), Level::Error, e.to_string())
        }
    }

    impl Into<::syn::parse::Error> for Diagnostic {
        fn into(self) -> ::syn::parse::Error {
            let span = if self.spans.is_empty() {
                proc_macro2::Span::call_site()
            } else {
                self.spans[0]
            };

            ::syn::parse::Error::new(span.into(), &self.message)
        }
    }

    impl<T: quote::ToTokens> Spanned for T {
        fn span(&self) -> Span {
            syn::spanned::Spanned::span(self)
        }
    }
}

pub use imp::Diagnostic;
