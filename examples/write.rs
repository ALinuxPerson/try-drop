use try_drop::adapters::DropAdapter;
use try_drop::drop_strategies::{PanicDropStrategy, WriteDropStrategy};
use try_drop::test_utils::{ErrorsOnDrop, Random};

fn main() {
    let mut strategy = WriteDropStrategy::stderr();
    strategy.prelude("error: ");
    try_drop::install_global_handlers(strategy, PanicDropStrategy::DEFAULT);
    let errors = DropAdapter(ErrorsOnDrop::<Random, _>::not_given());
    println!("dropping now (will only error randomly)");
    drop(errors);
    println!("finished dropping");
}
