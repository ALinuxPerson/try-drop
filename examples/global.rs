use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use try_drop::drop_strategies::{AdHocFallibleDropStrategy, AdHocDropStrategy};
use try_drop::test_utils::{ErrorsOnDrop, Fallible};
use try_drop::PureTryDrop;

fn main() {
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

    println!("drop, don't fail for global handler");
    let thing = ErrorsOnDrop::<Fallible, _>::not_given().adapt();
    drop(thing);

    println!("starting new thread which uses the global handler implicitly");
    thread::spawn(move || {
        println!("drop, do fail for global thread local handler");
        global_fail.store(true, Ordering::Release);
        let thing = ErrorsOnDrop::<Fallible, _>::not_given().adapt();
        drop(thing);
    })
    .join()
    .unwrap();
}
