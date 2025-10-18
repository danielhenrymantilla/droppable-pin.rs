use ::core::pin::pin;

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
