use try_drop::adapters::DropAdapter;
use try_drop::drop_strategies::ExitDropStrategy;
use try_drop::test_utils::{ErrorsOnDrop, Fallible};

fn main() {
    let errors = DropAdapter(ErrorsOnDrop::<Fallible, _>::given(
        ExitDropStrategy::DEFAULT,
        ExitDropStrategy::DEFAULT,
    ));
    println!("dropping now");
    drop(errors)
}
