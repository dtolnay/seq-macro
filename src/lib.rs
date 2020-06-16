//! [![github]](https://github.com/dtolnay/seq-macro)&ensp;[![crates-io]](https://crates.io/crates/seq-macro)&ensp;[![docs-rs]](https://docs.rs/seq-macro)
//!
//! [github]: https://img.shields.io/badge/github-8da0cb?style=for-the-badge&labelColor=555555&logo=github
//! [crates-io]: https://img.shields.io/badge/crates.io-fc8d62?style=for-the-badge&labelColor=555555&logo=rust
//! [docs-rs]: https://img.shields.io/badge/docs.rs-66c2a5?style=for-the-badge&labelColor=555555&logoColor=white&logo=data:image/svg+xml;base64,PHN2ZyByb2xlPSJpbWciIHhtbG5zPSJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3N2ZyIgdmlld0JveD0iMCAwIDUxMiA1MTIiPjxwYXRoIGZpbGw9IiNmNWY1ZjUiIGQ9Ik00ODguNiAyNTAuMkwzOTIgMjE0VjEwNS41YzAtMTUtOS4zLTI4LjQtMjMuNC0zMy43bC0xMDAtMzcuNWMtOC4xLTMuMS0xNy4xLTMuMS0yNS4zIDBsLTEwMCAzNy41Yy0xNC4xIDUuMy0yMy40IDE4LjctMjMuNCAzMy43VjIxNGwtOTYuNiAzNi4yQzkuMyAyNTUuNSAwIDI2OC45IDAgMjgzLjlWMzk0YzAgMTMuNiA3LjcgMjYuMSAxOS45IDMyLjJsMTAwIDUwYzEwLjEgNS4xIDIyLjEgNS4xIDMyLjIgMGwxMDMuOS01MiAxMDMuOSA1MmMxMC4xIDUuMSAyMi4xIDUuMSAzMi4yIDBsMTAwLTUwYzEyLjItNi4xIDE5LjktMTguNiAxOS45LTMyLjJWMjgzLjljMC0xNS05LjMtMjguNC0yMy40LTMzLjd6TTM1OCAyMTQuOGwtODUgMzEuOXYtNjguMmw4NS0zN3Y3My4zek0xNTQgMTA0LjFsMTAyLTM4LjIgMTAyIDM4LjJ2LjZsLTEwMiA0MS40LTEwMi00MS40di0uNnptODQgMjkxLjFsLTg1IDQyLjV2LTc5LjFsODUtMzguOHY3NS40em0wLTExMmwtMTAyIDQxLjQtMTAyLTQxLjR2LS42bDEwMi0zOC4yIDEwMiAzOC4ydi42em0yNDAgMTEybC04NSA0Mi41di03OS4xbDg1LTM4Ljh2NzUuNHptMC0xMTJsLTEwMiA0MS40LTEwMi00MS40di0uNmwxMDItMzguMiAxMDIgMzguMnYuNnoiPjwvcGF0aD48L3N2Zz4K
//!
//! <br>
//!
//! # Imagine for-loops in a macro
//!
//! This crate provides a `seq!` macro to repeat a fragment of source code and
//! substitute into each repetition a sequential numeric counter.
//!
#![cfg_attr(no_proc_macro_hygiene, doc = "```ignore")]
#![cfg_attr(not(no_proc_macro_hygiene), doc = "```")]
//! #![feature(proc_macro_hygiene)]
//!
//! use seq_macro::seq;
//!
//! fn main() {
//!     let tuple = (1000, 100, 10);
//!     let mut sum = 0;
//!
//!     // Expands to:
//!     //
//!     //     sum += tuple.0;
//!     //     sum += tuple.1;
//!     //     sum += tuple.2;
//!     //
//!     // This cannot be written using an ordinary for-loop because elements of
//!     // a tuple can only be accessed by their integer literal index, not by a
//!     // variable.
//!     seq!(N in 0..=2 {
//!         sum += tuple.N;
//!     });
//!
//!     assert_eq!(sum, 1110);
//! }
//! ```
//!
//! There are just two more features to know about:
//!
//! - If the input tokens contain a section surrounded by `#(` ... `)*` then
//!   only that part is repeated.
//!
//! - The numeric counter can be pasted onto the end of some prefix to form
//!   sequential identifiers.
//!
//! ```
//! use seq_macro::seq;
//!
//! seq!(N in 64..=127 {
//!     #[derive(Debug)]
//!     enum Demo {
//!         // Expands to Variant64, Variant65, ...
//!         #(
//!             Variant#N,
//!         )*
//!     }
//! });
//!
//! fn main() {
//!     assert_eq!("Variant99", format!("{:?}", Demo::Variant99));
//! }
//! ```

#![cfg_attr(docs_rs_workaround, feature(proc_macro))]

extern crate proc_macro;

mod parse;

use crate::parse::*;
use proc_macro::{Delimiter, Group, Ident, Literal, TokenStream, TokenTree};
use std::iter::{self, FromIterator};

#[proc_macro]
pub fn seq(input: TokenStream) -> TokenStream {
    match seq_impl(input) {
        Ok(expanded) => expanded,
        Err(error) => error.into_compile_error(),
    }
}

#[derive(Copy, Clone)]
struct Range {
    begin: u64,
    end: u64,
    inclusive: bool,
}

impl IntoIterator for Range {
    type Item = u64;
    type IntoIter = Box<dyn Iterator<Item = u64>>;

    fn into_iter(self) -> Self::IntoIter {
        if self.inclusive {
            Box::new(self.begin..=self.end)
        } else {
            Box::new(self.begin..self.end)
        }
    }
}

fn seq_impl(input: TokenStream) -> Result<TokenStream, SyntaxError> {
    let mut iter = input.into_iter();
    let var = require_ident(&mut iter)?;
    require_keyword(&mut iter, "in")?;
    let begin = require_integer(&mut iter)?;
    require_punct(&mut iter, '.')?;
    require_punct(&mut iter, '.')?;
    let inclusive = require_if_punct(&mut iter, '=')?;
    let end = require_integer(&mut iter)?;
    let body = require_braces(&mut iter)?;
    require_end(&mut iter)?;

    let range = Range { begin, end, inclusive };

    let mut found_repetition = false;
    let expanded = expand_repetitions(
        &var,
        range,
        body.clone(),
        &mut found_repetition,
    );
    if found_repetition {
        Ok(expanded)
    } else {
        // If no `#(...)*`, repeat the entire body.
        Ok(repeat(&var, range, &body))
    }
}

fn repeat(var: &Ident, range: Range, body: &TokenStream) -> TokenStream {
    let mut repeated = TokenStream::new();
    for number in range {
        repeated.extend(substitute_number(&var, number, body.clone()));
    }
    repeated
}

fn substitute_number(var: &Ident, number: u64, body: TokenStream) -> TokenStream {
    let mut tokens = Vec::from_iter(body);

    let mut i = 0;
    while i < tokens.len() {
        // Substitute our variable by itself, e.g. `N`.
        let replace = match &tokens[i] {
            TokenTree::Ident(ident) => ident.to_string() == var.to_string(),
            _ => false,
        };
        if replace {
            let original_span = tokens[i].span();
            let mut literal = Literal::u64_unsuffixed(number);
            literal.set_span(original_span);
            tokens[i] = TokenTree::Literal(literal);
            i += 1;
            continue;
        }

        // Substitute our variable concatenated onto some prefix, `Prefix#N`.
        if i + 3 <= tokens.len() {
            let prefix = match &tokens[i..i + 3] {
                [TokenTree::Ident(prefix), TokenTree::Punct(pound), TokenTree::Ident(ident)]
                    if pound.as_char() == '#' && ident.to_string() == var.to_string() =>
                {
                    Some(prefix.clone())
                }
                _ => None,
            };
            if let Some(prefix) = prefix {
                let concat = format!("{}{}", prefix, number);
                let ident = Ident::new(&concat, prefix.span());
                tokens.splice(i..i + 3, iter::once(TokenTree::Ident(ident)));
                i += 1;
                continue;
            }
        }

        // Recursively substitute content nested in a group.
        if let TokenTree::Group(group) = &mut tokens[i] {
            let original_span = group.span();
            let content = substitute_number(var, number, group.stream());
            *group = Group::new(group.delimiter(), content);
            group.set_span(original_span);
        }

        i += 1;
    }

    TokenStream::from_iter(tokens)
}

fn enter_repetition(tokens: &[TokenTree]) -> Option<TokenStream> {
    assert!(tokens.len() == 3);
    match &tokens[0] {
        TokenTree::Punct(punct) if punct.as_char() == '#' => {}
        _ => return None,
    }
    match &tokens[2] {
        TokenTree::Punct(punct) if punct.as_char() == '*' => {}
        _ => return None,
    }
    match &tokens[1] {
        TokenTree::Group(group) if group.delimiter() == Delimiter::Parenthesis => {
            Some(group.stream())
        }
        _ => None,
    }
}

fn expand_repetitions(
    var: &Ident,
    range: Range,
    body: TokenStream,
    found_repetition: &mut bool,
) -> TokenStream {
    let mut tokens = Vec::from_iter(body);

    // Look for `#(...)*`.
    let mut i = 0;
    while i < tokens.len() {
        if let TokenTree::Group(group) = &mut tokens[i] {
            let content =
                expand_repetitions(var, range, group.stream(), found_repetition);
            let original_span = group.span();
            *group = Group::new(group.delimiter(), content);
            group.set_span(original_span);
            i += 1;
            continue;
        }
        if i + 3 > tokens.len() {
            i += 1;
            continue;
        }
        let template = match enter_repetition(&tokens[i..i + 3]) {
            Some(template) => template,
            None => {
                i += 1;
                continue;
            }
        };
        *found_repetition = true;
        let mut repeated = Vec::new();
        for number in range {
            repeated.extend(substitute_number(var, number, template.clone()));
        }
        let repeated_len = repeated.len();
        tokens.splice(i..i + 3, repeated);
        i += repeated_len;
    }

    TokenStream::from_iter(tokens)
}
