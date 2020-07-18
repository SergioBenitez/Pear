use crate::input::{Show, ParserInfo};

pub use crate::expected::Expected;

#[derive(Debug, Clone)]
pub struct ParseError<C, E> {
    pub error: E,
    pub contexts: Vec<ParseContext<C>>,
}

#[derive(Debug, Clone)]
pub struct ParseContext<C> {
    pub parser: ParserInfo,
    pub context: C,
}

impl<C, E> ParseError<C, E> {
    #[inline(always)]
    pub fn new(error: E) -> ParseError<C, E> {
        ParseError { error, contexts: vec![] }
    }

    pub fn push_context(&mut self, context: C, parser: ParserInfo) {
        self.contexts.push(ParseContext { context, parser })
    }

    #[inline(always)]
    pub fn into<E2: From<E>>(self) -> ParseError<C, E2> {
        ParseError {
            error: self.error.into(),
            contexts: self.contexts,
        }
    }
}

impl<C: Show, E: std::fmt::Display> std::fmt::Display for ParseError<C, E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.error)?;
        for ctxt in &self.contexts {
            write!(f, "\n + {}", ctxt.parser.name)?;
            write!(f, " {}", &ctxt.context as &dyn Show)?;
        }

        Ok(())
    }
}
