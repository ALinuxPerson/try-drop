use abort_drop_strategy::AbortDropStrategy;
use panic_drop_strategy::PanicDropStrategy;
use try_drop::debugging::{ErrorsOnDrop, Random};
use try_drop::drop_adapter::DropAdapter;

fn main() {
    try_drop::install(PanicDropStrategy::new(), AbortDropStrategy);
    let errors = DropAdapter(ErrorsOnDrop::<Random, _>::not_given());
    println!("dropping now (will only error sometimes)");
    drop(errors);
}