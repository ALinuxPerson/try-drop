mod common;

use common::{ErrorsOnDrop, Fallible};
use try_drop::drop_adapter::DropAdapter;
use try_drop::drop_strategies::ExitDropStrategy;

fn main() {
    let errors = DropAdapter(ErrorsOnDrop::<Fallible, _>::given(
        ExitDropStrategy::DEFAULT,
        ExitDropStrategy::DEFAULT,
    ));
    println!("dropping now");
    drop(errors)
}
