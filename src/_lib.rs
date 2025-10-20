//! [`droppable_pin!`]: `droppable_pin!`
//! [`pin_drop!`]: `pin_drop!`
//! [`pin_set!`]: `pin_set!`
#![doc = include_str!("../README.md")]
#![no_std]
#![allow(unused_braces)]
// Base image sourced from Disney's The Lion King, used under fair use.
#![doc(html_logo_url = "\
    https://github.com/danielhenrymantilla/droppable-pin.rs/blob/\
    72bfcfec4e7499eed9fedbc79e8941fed722387e\
    /assets/droppable_pin_logo.jpg?raw=true\
")]

/// Invoke this macro around a given `let [mut] var = pin!()` declaration to allow invoking
/// [`pin_drop!`] and [`pin_set!`] on the given `var`, which have in turn been designed to avoid
/// silly borrow-checking errors.
///
/// # Syntax usage
///
/// ```rust
/// # use ::core::pin::pin;
/// # use ::droppable_pin::droppable_pin;
/// # async fn foo() {}
/// # async fn bar() {}
/// # /*
/// droppable_pin! {
///     let mut <varname> = pin!(<expr>);
///     /* You may have multiple such declarations in the same macro invocation. */
/// }
/// # */
///
/// // For instance:
/// droppable_pin! {
///     let mut a = pin!(foo());
///     let mut b = pin!(bar());
/// }
/// ```
///
/// # Motivation
///
/// <details open class="custom"><summary><span class="summary-box"><span>Click to hide</span></span></summary>
///
/// This is mainly useful when wanting to re-assign an `&mut`-borrowing `async fn` output (or,
/// more generally, some `-> impl Trait` value (or, even more generally, some value with drop glue
/// that makes use of that `&mut` borrow)) to some `pin!` value.
///
/// For instance, consider the following snippet:
///
/// ```rust, compile_fail
/// async fn some_async_fn(_: &mut i32) {}
///
/// let mut borrowed = 42;
/// let mut p = None;
/// for _ in 0..2 {
///     p = Some(some_async_fn(&mut borrowed));
/// }
/// ```
///
/// which errors with:
///
/// ```rust ,ignore
/// # (); /*
/// error[E0499]: cannot borrow `borrowed` as mutable more than once at a time
///   --> src/_lib.rs:70:23
///    |
/// 8  |     p = some_async_fn(&mut borrowed);
///    |                       ^^^^^^^^^^^^^ `borrowed` was mutably borrowed here in the previous iteration of the loop
/// 9  | }
/// # */
/// ```
///
/// And even if you set `p` to `None` before the assignment:
///
/// ```rust ,compile_fail
/// async fn some_async_fn(_: &mut i32) {}
///
/// let mut borrowed = 42;
/// let mut p;
/// for _ in 0..2 {
///     p = None;
///     p = Some(some_async_fn(&mut borrowed));
/// }
/// ```
///
/// it will still fail, with:
///
/// ```rust ,ignore
/// # (); /*
/// error[E0499]: cannot borrow `borrowed` as mutable more than once at a time
///  --> src/_lib.rs:94:28
///   |
/// 8 |     p = None;
///   |     - first borrow might be used here, when `p` is dropped and runs the destructor for type `Option<impl Future<Output = ()>>`
/// 9 |     p = Some(some_async_fn(&mut borrowed));
///   |                            ^^^^^^^^^^^^^ `borrowed` was mutably borrowed here in the previous iteration of the loop
/// # */
/// ```
///
///   - Notice how the error message already starts mentioning ``the destructor for type `Option<impl …>`‎``.
///
/// The only solution, here, is to actually `drop`/move out of the `p` variable, so as to properly
/// mark it as "empty"/exhausted to Rust (dropck, to be more precise), so that the borrow-checker
/// give its green-light to this snippet:
///
/// ```rust
/// async fn some_async_fn(_: &mut i32) {}
///
/// let mut borrowed = 42;
/// let mut p = None;
/// for _ in 0..2 {
///     drop(p);
///     p = Some(some_async_fn(&mut borrowed)); // ✅
/// }
/// ```
///
/// So far so good, but now consider that for this pattern to be genuinely useful, we will want to
/// *actually use `p`* in the loop, and since it's owned from outside, we will probably want to be
/// using it **by reference**, that is, behind (exclusive) pointer _indirection_!.
///
/// And in the context of `async fn`s, such indirection needs to be `Pin`-wrapped:
///
/// ```rust ,compile_fail
/// async fn some_async_fn() {}
///
/// # async {
/// let mut p = some_async_fn();
/// // Error, needed either for `some_async_fn()` to be `Unpin`,
/// // or for the reference to be a `Pin`-wrapped one.
/// (&mut p).await;
/// # };
/// ```
///
/// The correct way to do this would be:
///
/// ```rust
/// async fn some_async_fn() {}
///
/// # async {
/// let mut p = ::core::pin::pin!(some_async_fn());
/// p.as_mut().await; // ✅
/// # };
/// ```
///
/// But going back to our `loop {}`-dropck-borrow-checking-error situation, this runs into a problem:
///
/// ```rust ,compile_fail
/// use ::core::pin::pin;
///
/// async fn some_async_fn(_: &mut i32) {}
///
/// let mut borrowed = 42;
/// let mut p = pin!(None);
/// for _ in 0..2 {
///     drop(*p); // <- Err… how are we supposed to `drop()` the thing? It is what our `pin!`
///               //    points to, but the very safety and soundness of the design of `pin!` is for
///               //    us not to be able to mishandle it! 😕😕
///     p.set(Some(some_async_fn(&mut borrowed)));
/// }
/// ```
///
/// Hence the problem.
///
/// This is where the [`droppable_pin!`] macro, alongside [`pin_drop!`] and [`pin_set!`], enter into
/// play:
///
/// </details>
///
/// ## Example
///
/// ```rust
/// use ::core::pin::pin;
/// use ::droppable_pin::{droppable_pin, pin_drop, pin_set};
///
/// async fn some_async_fn(_: &mut i32) {}
///
/// let mut borrowed = 42;
/// droppable_pin! { // 👈
///     let mut p = pin!(None);
/// }
/// for _ in 0..2 {
///     pin_drop!(p); // 👈
///     pin_set!(p, Some(some_async_fn(&mut borrowed))); // ✅
///     // 👆
/// }
/// ```
///
/// ### A more realistic example
///
/// ```rust
/// use ::core::pin::pin;
/// use ::droppable_pin::{droppable_pin, pin_drop, pin_set}; // 👈
/// use ::futures_util::future::{Fuse, FusedFuture, FutureExt};
///
/// async fn foo() {}
/// async fn bar(_borrowed: &mut i32) {}
///
/// # fn block_on(fut: impl Future<Output = ()>) {
/// #     fut.now_or_never().expect("there to be no suspensions")
/// # }
/// #
/// # block_on(async {
/// let mut borrowed = 42;
/// droppable_pin! { // 👈
///     let mut a = pin!(foo().fuse());
///     let mut b = pin!(bar(&mut borrowed).fuse());
/// }
/// loop {
///     ::futures_util::select! {
///         () = a.as_mut() => {
///             /* handle this case... */
///             # if true { break; }
///
///             // Setup for next loop iteration:
///             pin_set!(a, foo().fuse());
///             // 👆
///             // same as:
///             a.set(foo().fuse());
///         },
///         () = b.as_mut() => {
///             /* handle this case... */
///
///             // Setup for next loop iteration:
///             // 1. Needed because of the `&mut borrowed` capture (see # Motivation).
///             // 👇
///             pin_drop!(b);
///             pin_set!(b, bar(&mut borrowed).fuse());
///             // 👆
///             // 2. Cannot use `Pin::set()` here because of `pin_drop!()`.
///         },
///     }
/// }
/// # });
/// ```
///
/// And rewritten to avoid code duplication (of the (re)initializations of `a` and `b`):
///
/// ```rust
/// use ::core::pin::pin;
/// use ::droppable_pin::{droppable_pin, pin_drop, pin_set}; // 👈
/// use ::futures_util::future::{Fuse, FusedFuture, FutureExt};
///
/// async fn foo() {}
/// async fn bar(_borrowed: &mut i32) {}
///
/// # fn block_on(fut: impl Future<Output = ()>) {
/// #     fut.now_or_never().expect("there to be no suspensions")
/// # }
/// #
/// # block_on(async {
/// let mut borrowed = 42;
/// droppable_pin! { // 👈
///     let mut a = pin!(Fuse::terminated());
///     let mut b = pin!(Fuse::terminated());
/// }
/// loop {
///     if a.is_terminated() {
///         // 👇
///         pin_set!(a, foo().fuse());
///         // same as:
///         a.set(foo().fuse());
///     }
///     if b.is_terminated() {
///         // 1. Needed because of the `&mut borrowed` capture (see # Motivation).
///         // 👇
///         pin_drop!(b);
///         pin_set!(b, bar(&mut borrowed).fuse());
///         // 👆
///         // 2. Cannot use `Pin::set()` here because of `pin_drop!()`.
///     }
///     ::futures_util::select! {
///         () = a.as_mut() => {
///             /* handle this case... */
///             # if true { break; }
///         },
///         () = b.as_mut() => {
///             /* handle this case... */
///         },
///     }
/// }
/// # });
/// ```
#[macro_export]
macro_rules! droppable_pin {(
    $(
        let mut $var:ident $(: $T:ty)? =
            $($(@ if $leading:tt)? ::)? $($pin_macro:ident)::+ !($value:expr $(,)?);
    )+
) => (
    $(
        // Note: safety-invariants-wise, `MaybeDangling<T>` and `T` are equivalent (they only
        // differ in a subtle *validity* invariant aspect). Consider this to be an 100% transparent
        // wrapper around `T`, whose only impact is for `Pin::new_unchecked()` to require a
        // `DerefMut` call.
        let mut hygiene_private_pinned_value = $crate::ඞ::maybe_dangling::MaybeDangling::new($value);

        #[allow(unused_mut)]
        let mut $var $(: $T)? = if true {
            // SAFETY:
            //
            // `hygiene_private_pinned_value`, as indicated by its name, is hygiene-private
            // and thus unable to be directly accessed by the user. So the user cannot misuse this,
            // and we ourselves do not misuse it either (very similar SAFETY reasoning to that
            // of stdlib `pin!`).
            //
            // Indeed:
            //
            // > we never move out of `hygiene_private_pinned_value` or otherwise invalidate its
            // > backing storage without first calling its drop glue in-place!
            //
            //   - The current `hygiene_private_pinned_value` is not `ManuallyDrop`-ped, so by the
            //     time it goes out of scope it shall be disposed of by the Rust scope machinery,
            //     which properly drops it in-place (c.f. stdlib `pin!`).
            //
            //   - The only other (non-stdlib macro) usages we perform on
            //     `hygiene_private_pinned_value` are the following:
            //
            //       - we may `pin_drop!` it (as defined by the following `…ඞpindrop_$var` macro).
            //
            //         This drops it in-place (which invalidates the borrow held in `$var`, making
            //         the `Pin` no longer accessible), which abides by and puts an end to the `Pin`
            //         guarantee.
            //
            //         For instance, as far as the `Pin` contract is concerned, we get to be able to
            //         "move" the value around again.
            //
            //         Granted, as far as the _safety invariants_ of that type are concerned, we
            //         ought to do nothing with that now dropped value.
            //
            //         Which is why we merely `forget()` it, the one API guaranteed not to run into
            //         any safety invariants.
            //
            //       - we may `pin_set!` it (as defined by the following `…ඞpinset_$var` macro).)
            //
            //           - if the value was set / non-empty / non-exhausted, this first drops the
            //             value in its place, before assigning the new value (much like the safe
            //             `Pin::set()` API does).
            //
            //           - else, the value was empty/exhausted, and the `Pin` borrow,
            //             lifetime-invalidated (and the `Pin` contract, fulfilled, by previous
            //             invocation of its drop glue), so there is nothing to be careful about
            //             w.r.t. pinning in this case.
            //
            //         Once set, by re-applying the very reasoning we are having here, we get to
            //         "refresh" the `Pin` borrow with a new `&mut hygiene_private_pinned_value`
            //         borrow, so as to give it a proper, re-usable, lifetime (since the "previous"
            //         one got invalidated upon drop/set).
            unsafe {
                $crate::ඞ::core::pin::Pin::new_unchecked(&mut *hygiene_private_pinned_value)
            }
        } else {
            // Dead-code branch so as to enforce that the otherwise merely syntactical/decorative
            // `pin!()` macro in the input do match the real `pin!` macro.
            //
            // But since we are re-implementing it ourselves, there is no point in actually using
            // it, hence why this check happens in an `unreachable_code` branch.
            #[allow(unreachable_code)] {
                loop {}
                $($(if $leading)? ::)? $($pin_macro)::+ ! { $value }
            }
        };

        $crate::ඞ::paste! {
            /// The exact shape of this name is not part of the public API!
            #[allow(unused)]
            macro_rules! [< missing_droppable_pin_around_var_declaration_ඞpindrop_ $var >] {() => (
                // `drop($var)` (unnecessary, just here for nicer UX. Using 1-tuple for clippy.)
                _ = ($var, );

                $crate::ඞ::maybe_dangling::drop_in_place!(hygiene_private_pinned_value);
                // This is why `MaybeDangling` was used. Not only does it offer *exactly* our
                // desired binding-consuming `drop_in_place`-to-abide-by-`Pin` API, but it also does
                // so:
                //   - guarding against unwind-safety (_i.e., what if said drop were to panic; here
                //     the process would be aborted);
                //   - preventing an aliasing validity invariant violation in case
                //     `hygiene_private_pinned_value` contained a `noalias` type such as `&mut`.
                //
                // Morally, and papering over those "details", what this `drop_in_place!()` macro
                // does is otherwise just:

                /*
                    // SAFETY: see safety comment around `let mut $var` declaration.
                    unsafe {
                        // 1. Drop it in-place so as to abide by the `Pin` contract.
                        (&raw mut hygiene_private_pinned_value).drop_in_place();
                    }
                    // 2. Tag the binding itself as empty/exhausted for dropck to be aware of it, but
                    //    without double-dropping the value.
                    $crate::ඞ::core::mem::forget(hygiene_private_pinned_value);
                */
            )}

            /// The exact shape of this name is not part of the public API!
            #[allow(unused)]
            macro_rules! [< missing_droppable_pin_around_var_declaration_ඞpinset_ $var >] {( $new_value:expr ) => (
                hygiene_private_pinned_value = $crate::ඞ::maybe_dangling::MaybeDangling::new($new_value);
                // SAFETY: see safety comment around `let mut $var` declaration.
                $var = unsafe {
                    $crate::ඞ::core::pin::Pin::new_unchecked(&mut *hygiene_private_pinned_value)
                };
            )}

            #[allow(unused_imports)]
            use $crate::ඞ::diagnostics::declaration_properly_wrapped_in_the_droppable_pin_macro as [< ඞcanary_ $var >];
        }
    )+
)}

/// Drops, *in-place*, the `value: T` to which the given `$var: Pin<&mut T>` points to.
///
/// It does so in a manner of which drop-check will be aware of, avoiding a borrow-checking error
/// which the more naive `$var.set(<dummy>)` would run into.
///
/// Note that this API is only available for `$var: Pin<&mut _>` bindings which stem from
/// a [`droppable_pin!`] invocation.
#[macro_export]
macro_rules! pin_drop {( $var:ident $(,)? ) => (
    $crate::ඞ::check_var_stems_from_droppable_pin!($var);
    $crate::ඞ::paste! {
        [< missing_droppable_pin_around_var_declaration_ඞpindrop_ $var >]!();
    }
)}


/// Same as <code>[Pin::set](&mut $var, $value)</code>, but usable even after [`pin_drop!`] has been
/// invoked on the given `$var`.
///
/// Note that this API is only available for `$var: Pin<&mut _>` bindings which stem from
/// a [`droppable_pin!`] invocation.
///
/// [Pin::set]: [`::core::pin::Pin::set()`]
#[macro_export]
macro_rules! pin_set {( $var:ident, $value:expr $(,)? ) => (
    $crate::ඞ::check_var_stems_from_droppable_pin!($var);
    $crate::ඞ::paste! {
        [< missing_droppable_pin_around_var_declaration_ඞpinset_ $var >]!( $value );
    }
)}

// macro internals
#[doc(hidden)] /** Not part of the public API */ pub
mod ඞ {
    pub use ::core; // or `std`
    pub use ::maybe_dangling;
    pub use ::paste::paste;
    #[doc(inline)]
    pub use crate::ඞcheck_var_stems_from_droppable_pin as check_var_stems_from_droppable_pin;

    pub mod diagnostics {
        #![allow(nonstandard_style)]

        pub const FALLBACK: the_given_var = the_given_var {};
        pub struct the_given_var {}
        pub struct declaration_properly_wrapped_in_the_droppable_pin_macro;

        #[diagnostic::on_unimplemented(
            message = "\
                missing `droppable_pin! {{}}` macro invocation wrapping the `let var = pin!(…)` \
                declaration of the given `var`\
            ",
            label = "\
                wrap the `let var = pin!(…)` declaration of this `var` in a `droppable_pin! {{}}` \
                macro invocation.
Like this:

droppable_pin! {{
    let mut var = pin!(...);
}}\
            ",
            note = "\
                remember to wrap the `let var = pin!(…)` declaration of this `var` in a \
                `droppable_pin! {{}}` macro invocation. Like so:

droppable_pin! {{
    let mut var = pin!(...);
}}\
            ",
            note = "\
                it is important that the given `var` not have been renamed \
                (or re-assigned to differently-named binding)\
            "
        )]
        pub trait declaration_of_the_variable_has_been_declaration_properly_wrapped_in_the_droppable_pin_macro {}

        impl declaration_of_the_variable_has_been_declaration_properly_wrapped_in_the_droppable_pin_macro for declaration_properly_wrapped_in_the_droppable_pin_macro {}
        pub fn check_var_stems_from_droppable_pin(_: impl declaration_of_the_variable_has_been_declaration_properly_wrapped_in_the_droppable_pin_macro) {}
    }
}

/// Not part of the public API.
///
/// Checks that the given `$var` do correspond to a [`droppable_pin!`] declaration.
#[macro_export]
#[doc(hidden)]
macro_rules! ඞcheck_var_stems_from_droppable_pin {( $var:ident $(,)? ) => ({
    if false {
        #[allow(warnings)] {
            // ensure we are in dead-code in case `$var` has been `drop!`ped already.
            loop {}
            // 1. Basic, sanity type-check.
            let _: $crate::ඞ::core::pin::Pin<&mut _> = $var;

            // 2. (Ab)use `#[diagnostic::on_unimplemented()]` functionality and name shadowing
            //    to observe when the given variable does not have the associated names in scope,
            //    and then produce nicer-reading diagnostics suggesting the appropriate fix.
            $crate::ඞ::paste!({
                // This makes it so `[< ඞcanary_ $var >]` is in the current scope, at least in the
                // *type* namespace (so that referring to it not fail, but also not shadow the
                // genuine `[< ඞcanary_ $var >]` in the *value* namespace).
                use $crate::ඞ::diagnostics::the_given_var as [< ඞcanary_ $var >];
                // Make sure that `$var` be in the current scope (in any applicable
                // namespace), so the following glob-import be of lesser priority and thus unable
                // to shadow anything.
                use [< ඞcanary_ $var >] as $var;
                // Now glob-import with lower/fallback priority `$var` from any namespace which may
                // currently be missing:
                //   - type namespace: always `the_given_var`.
                //   - value namespace: when properly-annotated, previous `use` brought it into
                //     scope, and is of type `declaration_properly_wrapped_in_the_droppable_pin_macro`.
                //     Otherwise, it brings `FALLBACK`, of type `the_given_var`.
                use helper::*;
                mod helper {
                    pub(super) use $crate::ඞ::diagnostics::FALLBACK as $var;
                }
                // Perform a trait implementation assertion, knowing that `the_given_var` does not
                // implement it, whereas `declaration_properly_wrapped_in_the_droppable_pin_macro`
                // does. The trait is aptly named
                // declaration_of_the_variable_has_been_declaration_properly_wrapped_in_the_droppable_pin_macro
                // and features extra, human-readable, `#[diagnostic::on_unimplemented()]` diagnostics.
                $crate::ඞ::diagnostics::check_var_stems_from_droppable_pin($var);
            });
        }
    }
})}

#[doc = include_str!("compile_fail_tests.md")]
mod _compile_fail_tests {}
