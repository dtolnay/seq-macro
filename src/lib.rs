//! # Imagine for-loops in a macro
//!
//! This crate provides a `seq!` macro to repeat a fragment of source code and
//! substitute into each repetition a sequential numeric counter.
//!
#![cfg_attr(no_proc_macro_hygiene, doc = "```rust,ignore")]
#![cfg_attr(not(no_proc_macro_hygiene), doc = "```rust")]
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
//! ```rust
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

use parse::*;
use proc_macro::{Delimiter, Group, Ident, Literal, Spacing, TokenStream, TokenTree};
use std::iter::{self, FromIterator};

#[proc_macro]
pub fn seq(input: TokenStream) -> TokenStream {
    match seq_impl(input) {
        Ok(expanded) => expanded,
        Err(error) => error.into_compile_error(),
    }
}

fn seq_impl(input: TokenStream) -> Result<TokenStream, SyntaxError> {
    let mut iter = input.into_iter();
    let var = require_ident(&mut iter)?;
    require_keyword(&mut iter, "in")?;
    let begin = require_integer(&mut iter)?;
    require_punct(&mut iter, '.', Spacing::Joint)?;
    require_punct(&mut iter, '.', Spacing::Joint)?;
    require_punct(&mut iter, '=', Spacing::Alone)?;
    let end = require_integer(&mut iter)?;
    let body = require_braces(&mut iter)?;
    require_end(&mut iter)?;

    let mut found_repetition = false;
    let expanded = expand_repetitions(&var, begin, end, body.clone(), &mut found_repetition);
    if found_repetition {
        Ok(expanded)
    } else {
        // If no `#(...)*`, repeat the entire body.
        Ok(repeat(&var, begin, end, &body))
    }
}

fn repeat(var: &Ident, begin: u64, end: u64, body: &TokenStream) -> TokenStream {
    let mut repeated = TokenStream::new();
    for number in begin..=end {
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
    begin: u64,
    end: u64,
    body: TokenStream,
    found_repetition: &mut bool,
) -> TokenStream {
    let mut tokens = Vec::from_iter(body);

    // Look for `#(...)*`.
    let mut i = 0;
    while i < tokens.len() {
        if let TokenTree::Group(group) = &mut tokens[i] {
            let content = expand_repetitions(var, begin, end, group.stream(), found_repetition);
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
        for number in begin..=end {
            repeated.extend(substitute_number(var, number, template.clone()));
        }
        let repeated_len = repeated.len();
        tokens.splice(i..i + 3, repeated);
        i += repeated_len;
    }

    TokenStream::from_iter(tokens)
}
