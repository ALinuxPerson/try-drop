use try_drop::drop_strategies::unreachable::UnreachableDropStrategy;
use try_drop::test_utils::{ErrorsOnDrop, Fallible};
use try_drop::PureTryDrop;

fn main() {
    try_drop::install_global_handlers(
        UnreachableDropStrategy::r#unsafe(),
        UnreachableDropStrategy::r#unsafe(),
    );
    let _errors = ErrorsOnDrop::<Fallible, _>::not_given().adapt();
}
