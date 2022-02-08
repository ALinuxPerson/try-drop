mod common;

use common::{ErrorsOnDrop, Fallible};
use try_drop::drop_strategies::ExitDropStrategy;
use try_drop::DropAdapter;

fn main() {
    let errors = DropAdapter(ErrorsOnDrop::<Fallible, _>::given(
        ExitDropStrategy::DEFAULT,
        ExitDropStrategy::DEFAULT,
    ));
    println!("dropping now");
    drop(errors)
}
