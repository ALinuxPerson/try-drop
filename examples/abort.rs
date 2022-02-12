use try_drop::test_utils::{ErrorsOnDrop, Fallible};
use try_drop::adapters::DropAdapter;
use try_drop::drop_strategies::AbortDropStrategy;

fn main() {
    try_drop::install_global_handlers(AbortDropStrategy, AbortDropStrategy);
    let errors = DropAdapter(ErrorsOnDrop::<Fallible, _>::not_given());
    println!("dropping now");
    drop(errors);
    panic!("the process should've aborted by now")
}
