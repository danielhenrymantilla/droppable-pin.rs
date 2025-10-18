# `::droppable-pin`

The eponoymous [`droppable_pin!`] macro around a given `let var = pin!()` declaration allows
invoking [`pin_drop!`] and [`pin_set!`] on the given `var`, which have in turn been designed to
avoid silly borrow-checking errors.

[`droppable_pin!`]: https://docs.rs/droppable-pin/*/droppable_pin/macro.droppable_pin.html
[`pin_drop!`]: https://docs.rs/droppable-pin/*/droppable_pin/macro.pin_drop.html
[`pin_set!`]: https://docs.rs/droppable-pin/*/droppable_pin/macro.pin_set.html

[![Repository](https://img.shields.io/badge/repository-GitHub-brightgreen.svg)](
https://github.com/danielhenrymantilla/droppable-pin.rs)
[![Latest version](https://img.shields.io/crates/v/droppable-pin.svg)](
https://crates.io/crates/droppable-pin)
[![Documentation](https://docs.rs/droppable-pin/badge.svg)](
https://docs.rs/droppable-pin)
[![MSRV](https://img.shields.io/badge/MSRV-1.85.0-white)](
https://gist.github.com/danielhenrymantilla/9b59de4db8e5f2467ed008b3c450527b)
[![unsafe used](https://img.shields.io/badge/unsafe-used-ffcc66.svg)](
https://github.com/rust-secure-code/safety-dance/)
[![so that you don't](https://img.shields.io/badge/so_that-you_dont-success.svg)](
https://github.com/rust-secure-code/safety-dance/)
[![License](https://img.shields.io/crates/l/droppable-pin.svg)](
https://github.com/danielhenrymantilla/droppable-pin.rs/blob/master/LICENSE-ZLIB)
[![CI](https://github.com/danielhenrymantilla/droppable-pin.rs/workflows/CI/badge.svg)](
https://github.com/danielhenrymantilla/droppable-pin.rs/actions)
[![no_std compatible](https://img.shields.io/badge/no__std-compatible-success.svg)](
https://github.com/rust-secure-code/safety-dance/)

<!-- Templated by `cargo-generate` using https://github.com/danielhenrymantilla/proc-macro-template -->

## Example

```rust
# async {
#
use ::core::pin::pin;
use ::droppable_pin::{droppable_pin, pin_drop, pin_set}; // 👈
use ::futures_util::future::{Fuse, FusedFuture, FutureExt};

async fn foo() {}
async fn bar(_borrowed: &mut i32) {}

let mut borrowed = 42;
droppable_pin! { // 👈
    let mut a = pin!(Fuse::terminated()); // Reminder: `Fuse::terminated()` is akin to `None`,
    let mut b = pin!(Fuse::terminated()); // and `future().fuse()`, to `Some(future())`.
}
loop {
    if a.is_terminated() {
        // 👇
        pin_set!(a, foo().fuse());
        // same as:
        a.set(foo().fuse());
    }
    if b.is_terminated() {
        // 1. Needed because of the `&mut borrowed` capture (see # Motivation).
        // 👇
        pin_drop!(b);
        pin_set!(b, bar(&mut borrowed).fuse());
        // 👆
        // 2. Cannot use `Pin::set()` here because of `pin_drop!()`.
    }
    ::futures_util::select! {
        () = a.as_mut() => {
            /* handle this case... */
            # if true { break; }
        },
        () = b.as_mut() => {
            /* handle this case... */
        },
    }
}
#
# };
```

---

See the docs of [`droppable_pin!`] for more information and the motivation behind this.
