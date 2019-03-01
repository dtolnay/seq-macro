use proc_macro::token_stream::IntoIter as TokenIter;
use proc_macro::{Delimiter, Group, Ident, Literal, Punct, Spacing, Span, TokenStream, TokenTree};
use std::borrow::Borrow;
use std::fmt::Display;
use std::iter::FromIterator;

pub struct SyntaxError {
    message: String,
    span: Span,
}

impl SyntaxError {
    pub fn into_compile_error(self) -> TokenStream {
        // compile_error! { $message }
        TokenStream::from_iter(vec![
            TokenTree::Ident(Ident::new("compile_error", self.span)),
            TokenTree::Punct({
                let mut punct = Punct::new('!', Spacing::Alone);
                punct.set_span(self.span);
                punct
            }),
            TokenTree::Group({
                let mut group = Group::new(Delimiter::Brace, {
                    TokenStream::from_iter(vec![TokenTree::Literal({
                        let mut string = Literal::string(&self.message);
                        string.set_span(self.span);
                        string
                    })])
                });
                group.set_span(self.span);
                group
            }),
        ])
    }
}

fn next_token(iter: &mut TokenIter) -> Result<TokenTree, SyntaxError> {
    iter.next().ok_or_else(|| SyntaxError {
        message: "unexpected end of input".to_owned(),
        span: Span::call_site(),
    })
}

fn syntax<T: Borrow<TokenTree>, M: Display>(token: T, message: M) -> SyntaxError {
    SyntaxError {
        message: message.to_string(),
        span: token.borrow().span(),
    }
}

pub fn require_ident(iter: &mut TokenIter) -> Result<Ident, SyntaxError> {
    match next_token(iter)? {
        TokenTree::Ident(ident) => Ok(ident),
        other => Err(syntax(other, "expected ident")),
    }
}

pub fn require_keyword(iter: &mut TokenIter, keyword: &str) -> Result<(), SyntaxError> {
    let token = next_token(iter)?;
    if let TokenTree::Ident(ident) = &token {
        if ident.to_string() == keyword {
            return Ok(());
        }
    }
    Err(syntax(token, format!("expected `{}`", keyword)))
}

pub fn require_integer(iter: &mut TokenIter) -> Result<u64, SyntaxError> {
    let mut token = next_token(iter)?;

    loop {
        match token {
            TokenTree::Group(group) => {
                let delimiter = group.delimiter();
                let mut stream = group.stream().into_iter();
                token = TokenTree::Group(group);
                if delimiter != Delimiter::None {
                    break;
                }
                let first = match stream.next() {
                    Some(first) => first,
                    None => break,
                };
                match stream.next() {
                    Some(_) => break,
                    None => token = first,
                }
            }
            TokenTree::Literal(lit) => {
                if let Ok(integer) = lit.to_string().parse::<u64>() {
                    return Ok(integer);
                }
                token = TokenTree::Literal(lit);
                return Err(syntax(token, "expected unsuffixed integer literal"));
            }
            _ => break,
        }
    }

    Err(syntax(token, "expected integer"))
}

pub fn require_if_punct(iter: &mut TokenIter, ch: char) -> Result<bool, SyntaxError> {
    let present = match iter.clone().next() {
        Some(TokenTree::Punct(_)) => {
            require_punct(iter, ch)?;
            true
        }
        _ => false,
    };
    Ok(present)
}

pub fn require_punct(iter: &mut TokenIter, ch: char) -> Result<(), SyntaxError> {
    let token = next_token(iter)?;
    if let TokenTree::Punct(punct) = &token {
        if punct.as_char() == ch {
            return Ok(());
        }
    }
    Err(syntax(token, format!("expected `{}`", ch)))
}

pub fn require_braces(iter: &mut TokenIter) -> Result<TokenStream, SyntaxError> {
    let token = next_token(iter)?;
    if let TokenTree::Group(group) = &token {
        if group.delimiter() == Delimiter::Brace {
            return Ok(group.stream());
        }
    }
    Err(syntax(token, "expected curly braces"))
}

pub fn require_end(iter: &mut TokenIter) -> Result<(), SyntaxError> {
    match iter.next() {
        Some(token) => Err(syntax(token, "unexpected token")),
        None => Ok(()),
    }
}
