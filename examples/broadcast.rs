use std::thread;
use std::time::Duration;
use tokio::runtime::Runtime;
use try_drop::adapters::DropAdapter;
use try_drop::drop_strategies::broadcast::OkIfAlone;
use try_drop::drop_strategies::{BroadcastDropStrategy, PanicDropStrategy};
use try_drop::test_utils::{ErrorsOnDrop, Random};

fn main() -> Result<(), try_drop::Error> {
    let _guard = Runtime::new()?.enter();
    let (strategy, mut r1) = BroadcastDropStrategy::<OkIfAlone>::new(16);
    let mut r2 = strategy.subscribe();
    try_drop::install_global_handlers(strategy, PanicDropStrategy::DEFAULT);
    let errors = DropAdapter(ErrorsOnDrop::<Random, _>::not_given());

    thread::spawn(move || {
        println!("waiting for error in thread 1");
        let error = r1.recv().unwrap();
        println!("from thread 1: {error}")
    });

    thread::spawn(move || {
        println!("waiting for error in thread 2");
        let error = r2.recv().unwrap();
        println!("from thread 2: {error}")
    });

    println!("dropping now (will only error sometimes)");
    drop(errors);

    thread::sleep(Duration::from_millis(100));

    Ok(())
}
