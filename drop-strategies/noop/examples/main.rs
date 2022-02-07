use noop_drop_strategy::NoOpDropStrategy;
use try_drop::debugging::{ErrorsOnDrop, Fallible};
use try_drop::drop_adapter::DropAdapter;

fn main() {
    let errors = DropAdapter(ErrorsOnDrop::<Fallible, _>::given(NoOpDropStrategy, NoOpDropStrategy));
    println!("dropping errors now");
    drop(errors);
}