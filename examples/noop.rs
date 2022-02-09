use try_drop::drop_strategies::NoOpDropStrategy;
use try_drop::DropAdapter;
use try_drop::test_utils::{ErrorsOnDrop, Fallible};

fn main() {
    let errors = DropAdapter(ErrorsOnDrop::<Fallible, _>::given(
        NoOpDropStrategy,
        NoOpDropStrategy,
    ));
    println!("dropping errors now");
    drop(errors);
    println!("you should see this message")
}
