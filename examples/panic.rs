use try_drop::drop_strategies::{AbortDropStrategy, PanicDropStrategy};
use try_drop::test_utils::{ErrorsOnDrop, Random};
use try_drop::adapters::DropAdapter;

fn main() {
    try_drop::install_global_handlers(PanicDropStrategy::DEFAULT, AbortDropStrategy);
    let errors = DropAdapter(ErrorsOnDrop::<Random, _>::not_given());
    println!("dropping now (will only error sometimes)");
    drop(errors);
}
