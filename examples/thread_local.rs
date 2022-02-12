use std::cell::RefCell;
use std::rc::Rc;
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
    println!("drop, don't fail for primary thread local handler");
    let thing = ErrorsOnDrop::<Fallible, _>::not_given().adapt();
    drop(thing);

    println!("drop, do fail for primary thread local handler");
    *thread_local_fail.borrow_mut() = true;
    let thing = ErrorsOnDrop::<Fallible, _>::not_given().adapt();
    drop(thing);

    thread::spawn(|| {
        println!("start new thread with no thread handlers, should just write the error");
        let _errors = ErrorsOnDrop::<Fallible, _>::not_given().adapt();
    })
    .join()
    .unwrap();
}
