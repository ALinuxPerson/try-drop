use try_drop::drop_strategies::unreachable::UnreachableDropStrategy;
use try_drop::drop_strategies::AdHocTryDropStrategy;
use try_drop::handlers::*;
use try_drop::test_utils::{ErrorsOnDrop, Fallible};
use try_drop::PureTryDrop;

fn main() {
    println!("install main thread local handler");
    primary::thread_local::install(AdHocTryDropStrategy(|error| {
        println!("error from the main thread local: {error}")
    }));
    let thing = ErrorsOnDrop::<Fallible, _>::not_given().adapt();
    println!("drop test from main thread local handler");
    drop(thing);

    {
        println!("install first thread local scope");
        let _guard = primary::thread_local::scope(AdHocTryDropStrategy(|error| {
            println!("error from the first thread local scope: {error}")
        }));

        let thing = ErrorsOnDrop::<Fallible, _>::not_given().adapt();
        println!("drop test from first thread local scope");
        drop(thing)
    }

    println!("after the scope is dropped, we should be using the main thread local scope again");
    let thing = ErrorsOnDrop::<Fallible, _>::not_given().adapt();
    println!("drop test from exit of first thread local scope");
    drop(thing);

    {
        println!("install second thread local scope");
        let _guard = primary::thread_local::scope(AdHocTryDropStrategy(|error| {
            println!("error from the second thread local scope: {error}")
        }));

        let thing = ErrorsOnDrop::<Fallible, _>::not_given().adapt();
        println!("drop test from second thread local scope");
        drop(thing)
    }

    {
        println!("create nested scope");
        let _guard = primary::thread_local::ScopeGuard::try_new(UnreachableDropStrategy::safe());

        {
            println!("nested scopes aren't allowed");
            let error = primary::thread_local::ScopeGuard::try_new(UnreachableDropStrategy::safe())
                .unwrap_err();
            println!(
                "the error comes in a form of a `LockedError`: debug={error:?}, display={error}"
            )
        }
    }
}
