use panic_drop_strategy::PanicDropStrategy;
use try_drop::debugging::{ErrorsOnDrop, Fallible, Infallible, Random};
use try_drop::drop_adapter::DropAdapter;
use write_drop_strategy::WriteDropStrategy;

fn main() {
    let mut strategy = WriteDropStrategy::stderr();
    strategy.prelude("error: ");
    try_drop::install(strategy, PanicDropStrategy::new());
    let errors = DropAdapter(ErrorsOnDrop::<Random, _>::not_given());
    println!("dropping now (will only error randomly)");
    drop(errors);
    println!("finished dropping");
}