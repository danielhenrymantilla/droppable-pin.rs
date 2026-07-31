# The following snippets fail to compile

```rust ,compile_fail
use ::droppable_pin::*;

compile_fail!("TODO");
```

## Coërcions should be rejected

See <https://github.com/danielhenrymantilla/droppable-pin.rs/issues/3>

```rust ,compile_fail
use droppable_pin::droppable_pin;
use std::pin::Pin;

macro_rules! my_pin {
    ($val:expr) => {
        panic!()
    };
}

pub fn wrong_pin<T>(x: &mut T, callback: impl FnOnce(Pin<&mut T>)) {
    droppable_pin! {
        let mut y: Pin<&mut T> = my_pin!(x);
    }
    callback(y);
}
```

<!-- Templated by `cargo-generate` using https://github.com/danielhenrymantilla/proc-macro-template -->
