use once_cell::sync::OnceCell;
use std::sync::Arc;
use try_drop::drop_strategies::once_cell::Ignore;
use try_drop::drop_strategies::{OnceCellDropStrategy, PanicDropStrategy};
use try_drop::test_utils::{ErrorsOnDrop, Mode, Random, TryDropTypes};
use try_drop::{adapters::DropAdapter, PureTryDrop};

fn drops_value<M: Mode, TDT: TryDropTypes>(value: DropAdapter<ErrorsOnDrop<M, TDT>>)
where
    ErrorsOnDrop<M, TDT>: PureTryDrop,
{
    drop(value)
}

fn main() {
    let error = Arc::new(OnceCell::new());
    let strategy = OnceCellDropStrategy::<Ignore>::new(Arc::clone(&error));
    let value = ErrorsOnDrop::<Random, _>::given(strategy, PanicDropStrategy::DEFAULT).adapt();

    println!("will only error on drop sometimes");
    drops_value(value);

    if let Some(error) = Arc::try_unwrap(error).unwrap().take() {
        println!("an error occurred in `drops_value`: {error}")
    } else {
        println!("no error occurred in `drops_value`")
    }
}
