mod common;

use try_drop::drop_adapter::DropAdapter;
use try_drop::drop_strategies::NoOpDropStrategy;
use crate::common::{ErrorsOnDrop, Fallible};

fn main() {
    let errors = DropAdapter(ErrorsOnDrop::<Fallible, _>::given(NoOpDropStrategy, NoOpDropStrategy));
    println!("dropping errors now");
    drop(errors);
    println!("you should see this message")
}