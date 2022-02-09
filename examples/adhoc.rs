use try_drop::drop_strategies::{AdHocFallibleTryDropStrategy, AdHocTryDropStrategy};
use try_drop::PureTryDrop;
use try_drop::test_utils::{ErrorsOnDrop, Fallible};

fn main() {
    let fallible_try_drop_strategy = AdHocFallibleTryDropStrategy::new(|error| {
        println!("an error occurred from a drop: {error}");
        anyhow::bail!("this try drop strategy failed")
    });
    let fallback_try_drop_strategy =
        AdHocTryDropStrategy(|error| println!("error from the failed try drop strategy: {error}"));

    let errors =
        ErrorsOnDrop::<Fallible, _>::given(fallible_try_drop_strategy, fallback_try_drop_strategy)
            .adapt();
    drop(errors);
}
