use exit_drop_strategy::ExitDropStrategy;
use try_drop::debugging::{ErrorsOnDrop, Fallible};
use try_drop::drop_adapter::DropAdapter;

fn main() {
    let errors = DropAdapter(ErrorsOnDrop::<Fallible, _>::given(
        ExitDropStrategy::DEFAULT,
        ExitDropStrategy::DEFAULT,
    ));
    println!("dropping now");
    drop(errors)
}