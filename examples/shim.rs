use std::borrow::{Borrow, BorrowMut};
use std::cell::Cell;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use try_drop::drop_strategies::{AdHocFallibleDropStrategy, AdHocDropStrategy};
use try_drop::test_utils::{ErrorsOnDrop, Fallible};
use try_drop::PureTryDrop;

fn main() {
    println!("install thread local handlers from main thread");
    let thread_local_fail = Rc::new(Cell::new(false));
    let tlf = Rc::clone(&thread_local_fail);
    try_drop::install_thread_local_handlers(
        AdHocFallibleDropStrategy(move |error| {
            println!("from primary thread local handler: {error}");

            if tlf.get() {
                println!("forcing failure");
                anyhow::bail!("forced failure of primary thread local handler")
            } else {
                Ok(())
            }
        }),
        AdHocDropStrategy(|error| println!("from fallback thread local handler: {error}")),
    );

    println!("install global handlers from main thread");
    let global_fail = Arc::new(AtomicBool::new(false));
    let gf = Arc::clone(&global_fail);
    try_drop::install_global_handlers(
        AdHocFallibleDropStrategy(move |error| {
            println!("from primary global handler: {error}");

            if gf.load(Ordering::Acquire) {
                println!("forcing failure");
                anyhow::bail!("forced failure of primary global handler")
            } else {
                Ok(())
            }
        }),
        AdHocDropStrategy(|error| println!("from fallback global handler: {error}")),
    );
    println!("drop, don't fail for primary thread local handler");
    let thing = ErrorsOnDrop::<Fallible, _>::not_given().adapt();
    drop(thing);

    println!("drop, do fail for primary thread local handler");
    thread_local_fail.set(true);
    let thing = ErrorsOnDrop::<Fallible, _>::not_given().adapt();
    drop(thing);

    thread::spawn(move || {
        println!("starting new thread with no thread local handlers installed");
        let thing = ErrorsOnDrop::<Fallible, _>::not_given().adapt();
        drop(thing);

        println!("drop, don't fail for global thread local handler");
        let thing = ErrorsOnDrop::<Fallible, _>::not_given().adapt();
        drop(thing);

        println!("drop, do fail for global thread local handler");
        global_fail.store(true, Ordering::Release);
        let thing = ErrorsOnDrop::<Fallible, _>::not_given().adapt();
        drop(thing);
    })
    .join()
    .unwrap();
}
