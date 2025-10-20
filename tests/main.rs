use ::core::pin::{Pin, pin};

use droppable_pin::pin_set;

fn _basic() {
    async fn async_fn(_: &mut ()) {}

    let mut borrowed = ();
    ::droppable_pin::droppable_pin! {
        let mut p = pin!(async_fn(&mut borrowed));
    }
    for _ in 0..2 {
        p.as_mut();
        ::droppable_pin::pin_drop!(p);
        ::droppable_pin::pin_set!(p, async_fn(&mut borrowed));
        p.as_mut();
    }
}

fn _type_annotations() {
    ::droppable_pin::droppable_pin! {
        let mut a: Pin<&mut i32> = pin!(<_>::default());
        let mut b: Pin<&mut [i32]> = pin!([]);
    }
    let _: *mut Pin<&mut i32> = &raw mut a;
    let _: *mut Pin<&mut [i32]> = &raw mut b;
    pin_set!(a, <_>::default());
    pin_set!(b, <_>::default());
}
