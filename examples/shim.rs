use std::cell::RefCell;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use try_drop::drop_strategies::{AdHocFallibleTryDropStrategy, AdHocTryDropStrategy};
use try_drop::test_utils::{ErrorsOnDrop, Fallible};
use try_drop::PureTryDrop;

fn main() {
    println!("install thread local handlers from main thread");
    let thread_local_fail = Rc::new(RefCell::new(false));
    let tlf = Rc::clone(&thread_local_fail);
    try_drop::install_thread_local_handlers(
        AdHocFallibleTryDropStrategy(move |error| {
            println!("from primary thread local handler: {error}");

            if *tlf.borrow() {
                println!("forcing failure");
                anyhow::bail!("forced failure of primary thread local handler")
            } else {
                Ok(())
            }
        }),
        AdHocTryDropStrategy(|error| println!("from fallback thread local handler: {error}")),
    );

    println!("install global handlers from main thread");
    let global_fail = Arc::new(AtomicBool::new(false));
    let gf = Arc::clone(&global_fail);
    try_drop::install_global_handlers(
        AdHocFallibleTryDropStrategy(move |error| {
            println!("from primary global handler: {error}");

            if gf.load(Ordering::Acquire) {
                println!("forcing failure");
                anyhow::bail!("forced failure of primary global handler")
            } else {
                Ok(())
            }
        }),
        AdHocTryDropStrategy(|error| println!("from fallback global handler: {error}")),
    );
    println!("drop, don't fail for primary thread local handler");
    let thing = ErrorsOnDrop::<Fallible, _>::not_given().adapt();
    drop(thing);

    println!("drop, do fail for primary thread local handler");
    *thread_local_fail.borrow_mut() = true;
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
