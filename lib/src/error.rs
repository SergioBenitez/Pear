use std::fmt;

use crate::input::{Input, Show, ParserInfo};

pub use crate::expected::Expected;

pub struct ParseContext<I: Input> {
    pub parser: ParserInfo,
    pub context: Option<I::Context>,
}

pub struct ParseError<I: Input, E = Expected<I>> {
    pub error: E,
    pub contexts: Vec<ParseContext<I>>,
}

impl<I: Input, E> ParseError<I, E> {
    pub fn new(error: E) -> ParseError<I, E> {
        ParseError {
            error: error.into(),
            contexts: vec![]
        }
    }

    pub fn push_context(&mut self, context: Option<I::Context>, parser: ParserInfo) {
        self.contexts.push(ParseContext { context, parser })
    }

    #[inline(always)]
    pub fn into<E2: From<E>>(self) -> ParseError<I, E2> {
        ParseError {
            error: self.error.into(),
            contexts: self.contexts,
        }
    }
}

impl<I: Input, E: fmt::Debug> fmt::Debug for ParseError<I, E>
    where I::Context: fmt::Debug
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ParseError")
            .field("error", &self.error)
            .field("context", &self.contexts)
            .finish()
    }
}

impl<I: Input> fmt::Debug for ParseContext<I>
    where I::Context: fmt::Debug
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ParseContext")
            .field("parser", &self.parser)
            .field("context", &self.context)
            .finish()
    }
}

impl<I: Input, E: Clone> Clone for ParseError<I, E>
    where I::Context: Clone
{
    fn clone(&self) -> Self {
        ParseError {
            error: self.error.clone(),
            contexts: self.contexts.clone(),
        }
    }
}

impl<I: Input> Clone for ParseContext<I>
    where I::Context: Clone
{
    fn clone(&self) -> Self {
        ParseContext {
            context: self.context.clone(),
            parser: self.parser.clone()
        }
    }
}

impl<I: Input, E: fmt::Display> fmt::Display for ParseError<I, E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.error)?;
        for ctxt in &self.contexts {
            write!(f, "\n + {}", ctxt.parser.name)?;
            if let Some(ctxt) = &ctxt.context {
                write!(f, " at {})", ctxt as &dyn Show)?;
            }
        }

        Ok(())
    }
}
