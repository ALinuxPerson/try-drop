mod common;

use try_drop::drop_adapter::DropAdapter;
use try_drop::drop_strategies::{AbortDropStrategy, PanicDropStrategy};
use crate::common::{ErrorsOnDrop, Random};

fn main() {
    try_drop::install(PanicDropStrategy::new(), AbortDropStrategy);
    let errors = DropAdapter(ErrorsOnDrop::<Random, _>::not_given());
    println!("dropping now (will only error sometimes)");
    drop(errors);
}