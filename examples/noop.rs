mod common;

use crate::common::{ErrorsOnDrop, Fallible};
use try_drop::drop_adapter::DropAdapter;
use try_drop::drop_strategies::NoOpDropStrategy;

fn main() {
    let errors = DropAdapter(ErrorsOnDrop::<Fallible, _>::given(
        NoOpDropStrategy,
        NoOpDropStrategy,
    ));
    println!("dropping errors now");
    drop(errors);
    println!("you should see this message")
}
