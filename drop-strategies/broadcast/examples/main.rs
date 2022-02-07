use std::thread;
use std::time::Duration;
use broadcast_drop_strategy::{BroadcastDropStrategy, OkIfAlone};
use panic_drop_strategy::PanicDropStrategy;
use try_drop::debugging::Random;
use try_drop::drop_adapter::DropAdapter;

fn main() -> Result<(), try_drop::Error> {
    let (strategy, mut r1) = BroadcastDropStrategy::<OkIfAlone>::new(16)?;
    let mut r2 = strategy.subscribe();
    try_drop::install(strategy, PanicDropStrategy::new());
    let errors = DropAdapter(try_drop::debugging::ErrorsOnDrop::<Random, _>::not_given());

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

    drop(errors);

    thread::sleep(Duration::from_millis(100));

    Ok(())
}