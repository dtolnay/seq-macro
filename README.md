`seq!`
======

[![Build Status](https://api.travis-ci.org/dtolnay/seq-macro.svg?branch=master)](https://travis-ci.org/dtolnay/seq-macro)
[![Latest Version](https://img.shields.io/crates/v/seq-macro.svg)](https://crates.io/crates/seq-macro)

A `seq!` macro to repeat a fragment of source code and substitute into each
repetition a sequential numeric counter.

```toml
[dependencies]
seq-macro = "0.0"
```

```rust
#![feature(proc_macro_non_items)]

use seq_macro::seq;

fn main() {
    let tuple = (1000, 100, 10);
    let mut sum = 0;

    // Expands to:
    //
    //     sum += tuple.0;
    //     sum += tuple.1;
    //     sum += tuple.2;
    //
    // This cannot be written using an ordinary for-loop because elements of
    // a tuple can only be accessed by their integer literal index, not by a
    // variable.
    seq!(N in 0..=2 {
        sum += tuple.N;
    });

    assert_eq!(sum, 1110);
}
```

There are just two more features to know about:

- If the input tokens contain a section surrounded by `#(` ... `)*` then only
  that part is repeated.

- The numeric counter can be pasted onto the end of some prefix to form
  sequential identifiers.

```rust
use seq_macro::seq;

seq!(N in 64..=127 {
    #[derive(Debug)]
    enum Demo {
        // Expands to Variant64, Variant65, ...
        #(
            Variant#N,
        )*
    }
});

fn main() {
    assert_eq!("Variant99", format!("{:?}", Demo::Variant99));
}
```

## License

Licensed under either of

 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this crate by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
