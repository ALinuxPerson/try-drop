mod common;

use crate::common::{ErrorsOnDrop, Random};
use try_drop::drop_adapter::DropAdapter;
use try_drop::drop_strategies::{PanicDropStrategy, WriteDropStrategy};

fn main() {
    let mut strategy = WriteDropStrategy::stderr();
    strategy.prelude("error: ");
    try_drop::install(strategy, PanicDropStrategy::new());
    let errors = DropAdapter(ErrorsOnDrop::<Random, _>::not_given());
    println!("dropping now (will only error randomly)");
    drop(errors);
    println!("finished dropping");
}