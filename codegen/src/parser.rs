#![allow(dead_code)]

use syn::token;
use syn::synom::Synom;
use syn::buffer::{Cursor, TokenBuffer};
use syn::synom::PResult;

use proc_macro::{TokenStream, Span, Diagnostic};

pub use proc_macro2::Delimiter;

pub type Result<T> = ::std::result::Result<T, Diagnostic>;

pub enum Seperator {
    Comma,
    Pipe,
    Semi,
}

pub struct Parser {
    buffer: Box<TokenBuffer>,
    cursor: Cursor<'static>,
}

impl Parser {
    /// Creates a new parser that will feed off of `tokens`.
    pub fn new(tokens: TokenStream) -> Parser {
        let buffer = Box::new(TokenBuffer::new(tokens.into()));
        let cursor = unsafe {
            let buffer: &'static TokenBuffer = ::std::mem::transmute(&*buffer);
            buffer.begin()
        };

        Parser {
            buffer: buffer,
            cursor: cursor,
        }
    }

    pub fn remaining_stream(&self) -> TokenStream {
        self.cursor.token_stream().into()
    }

    /// Returns the `Span` of token that will be parsed next.
    pub fn current_span(&self) -> Span {
        self.cursor.token_tree()
            .map(|_| self.cursor.span().unstable())
            .unwrap_or_else(|| Span::call_site())
    }

    /// Parses the current tokens into a type `T`.
    pub fn parse<T: Synom>(&mut self) -> Result<T> {
        let description = match T::description() {
            Some(desc) => desc,
            None => unsafe { ::std::intrinsics::type_name::<T>() }
        };

        self.parse_synom(description, T::parse)
    }

    pub fn parse_synom<F, T>(&mut self, desc: &str, f: F) -> Result<T>
        where F: FnOnce(Cursor) -> PResult<T>
    {
        f(self.cursor).map(|(value, next_cursor)| {
            self.cursor = next_cursor;
            value
        }).map_err(|e| {
            self.current_span().error(format!("{}: expected {}", e, desc))
        })
    }

    pub fn eat<T: Synom>(&mut self) -> bool {
        self.try_parse(|p| p.parse::<T>()).is_ok()
    }

    pub fn try_parse<F, T>(&mut self, f: F) -> Result<T>
        where F: FnOnce(&mut Parser) -> Result<T>
    {
        let saved_cursor = self.cursor;
        f(self).map_err(|e| { self.cursor = saved_cursor; e })
    }

    /// Parses inside of a group using `f` delimited by `delim`.
    pub fn parse_group<F, T>(&mut self, delim: Delimiter, f: F) -> Result<T>
        where F: FnOnce(&mut Parser) -> Result<T>
    {
        if let Some((group_cursor, _, next_cursor)) = self.cursor.group(delim) {
            self.cursor = group_cursor;
            let result = f(self);
            self.cursor = next_cursor;
            result
        } else {
            let expected = match delim {
                Delimiter::Brace => "curly braced group",
                Delimiter::Bracket => "square bracketed group",
                Delimiter::Parenthesis => "parenthesized group",
                Delimiter::None => "invisible group"
            };

            Err(self.current_span()
                .error(format!("parse error: expected {}", expected)))
        }
    }

    /// Parses a `sep` separated list of `T`s and returns the `Vec`.
    pub fn parse_sep<F, T>(&mut self, sep: Seperator, mut f: F) -> Result<Vec<T>>
        where F: FnMut(&mut Parser) -> Result<T>
    {
        let mut output = vec![];
        while !self.is_eof() {
            output.push(f(self)?);
            let have_sep = match sep {
                Seperator::Comma => self.eat::<token::Comma>(),
                Seperator::Pipe => self.eat::<token::Or>(),
                Seperator::Semi => self.eat::<token::Semi>(),
            };

            if !have_sep {
                break;
            }
        }

        Ok(output)
    }

    pub fn eof(&self) -> Result<()> {
        if !self.cursor.eof() {
            let diag = self.current_span()
                .error("trailing characters; expected eof");

            return Err(diag);
        }

        Ok(())
    }

    pub fn is_eof(&self) -> bool {
        self.eof().is_ok()
    }
}

