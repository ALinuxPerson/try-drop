use try_drop::drop_strategies::unreachable::UnreachableDropStrategy;
use try_drop::test_utils::{ErrorsOnDrop, Fallible};
use try_drop::PureTryDrop;

fn main() {
    try_drop::install_global_handlers(
        UnreachableDropStrategy::safe(),
        UnreachableDropStrategy::safe(),
    );
    let _errors = ErrorsOnDrop::<Fallible, _>::not_given().adapt();
}
