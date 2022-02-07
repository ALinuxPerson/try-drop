use abort_drop_strategy::AbortDropStrategy;
use try_drop::debugging::{ErrorsOnDrop, Fallible};
use try_drop::drop_adapter::DropAdapter;

fn main() {
    try_drop::install(AbortDropStrategy, AbortDropStrategy);
    let errors = DropAdapter(ErrorsOnDrop::<Fallible, _>::not_given());
    println!("dropping now");
    drop(errors);
    panic!("the process should've aborted by now")
}