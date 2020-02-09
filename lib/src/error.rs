use crate::input::{Show, ParserInfo};

pub use crate::expected::Expected;

#[derive(Debug, Clone)]
pub struct ParseContext<C> {
    pub parser: ParserInfo,
    pub context: Option<C>,
}

#[derive(Debug, Clone)]
pub struct ParseError<C, E> {
    pub error: E,
    pub contexts: Vec<ParseContext<C>>,
}

impl<C, E> ParseError<C, E> {
    pub fn new(error: E) -> ParseError<C, E> {
        ParseError {
            error: error.into(),
            contexts: vec![]
        }
    }

    pub fn push_context(&mut self, context: Option<C>, parser: ParserInfo) {
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
            if let Some(ctxt) = &ctxt.context {
                write!(f, " at {}", ctxt as &dyn Show)?;
            }
        }

        Ok(())
    }
}
