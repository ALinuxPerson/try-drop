
use try_drop::test_utils::{ErrorsOnDrop, Mode, Random, TryDropTypes};
use once_cell::sync::OnceCell;
use std::sync::Arc;
use try_drop::drop_strategies::once_cell::Ignore;
use try_drop::drop_strategies::{OnceCellTryDropStrategy, PanicDropStrategy};
use try_drop::{DropAdapter, PureTryDrop};

fn drops_value<M: Mode, TDT: TryDropTypes>(value: DropAdapter<ErrorsOnDrop<M, TDT>>)
where
    ErrorsOnDrop<M, TDT>: PureTryDrop,
{
    drop(value)
}

fn main() {
    let error = Arc::new(OnceCell::new());
    let strategy = OnceCellTryDropStrategy::<Ignore>::new(Arc::clone(&error));
    let value = ErrorsOnDrop::<Random, _>::given(strategy, PanicDropStrategy::DEFAULT).adapt();

    println!("will only error on drop sometimes");
    drops_value(value);

    if let Some(error) = Arc::try_unwrap(error).unwrap().take() {
        println!("an error occurred in `drops_value`: {error}")
    } else {
        println!("no error occurred in `drops_value`")
    }
}
