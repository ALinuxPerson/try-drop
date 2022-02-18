<div align="center">
    <h1><b>try-drop</b></h1>
    <a href="https://www.crates.io/crates/try-drop">
        <img src="https://img.shields.io/crates/v/try-drop">
    </a>
    <a href="https://www.docs.rs/try-drop">
        <img src="https://docs.rs/try-drop/badge.svg">
    </a>
    <p>Batteries included error handling mechanisms for drops which can fail</p>
</div>

# Quick Usage
## For Clients
...where clients mean structures that may fail dropping,

Implement `TryDrop` for your type and `adapt` it like so:

```rust
use try_drop::TryDrop;

pub struct Foo { /* fields */ }

impl TryDrop for Foo {
    type Error = Error;
    
    unsafe fn try_drop(&mut self) -> Result<(), Self::Error> {
        // do stuff
        Ok(())
    }
}

let foo = Foo.adapt();
```

...or, if you want to avoid the `adapt` boilerplate:

```rust
use try_drop::{TryDrop, adapters::DropAdapter};

pub struct FooInner { /* fields */ }

impl TryDrop for FooInner {
    type Error = Error;
    
    unsafe fn try_drop(&mut self) -> Result<(), Self::Error> {
        // do stuff
        Ok(())
    }
}

pub struct Foo(pub DropAdapter<FooInner>);

impl Foo {
    pub fn from_inner(inner: FooInner) -> Self {
        Foo(DropAdapter(inner))
    }
}
```

We must adapt it because implementing `TryDrop` doesn't also implement `Drop` with your type. This is due to the orphan 
rules and the fact that `Drop` can only be implemented for structs, enums, and unions, but not generics.  With this, if
dropping `Foo` fails, it will automatically print the error to standard error.

## For Servers
...where servers mean how to handle the drop errors (also means drop strategies),

Implement `TryDropStrategy` for your structure.

```rust
use try_drop::TryDropStrategy;

pub struct Strategy { /* fields */ }

impl TryDropStrategy for Strategy {
    fn handle_error(&self, error: try_drop::Error) {
        // handle the error here
    }
}
```

...then install either for this thread,

```rust
try_drop::install_thread_local_handlers(Strategy, /* other strategy, use the `PanicDropStrategy` if you don't know */)
```

...install it globally (meaning if no thread local strategies are set, use this),

```rust
try_drop::install_global_handlers(Strategy, /* other strategy */)
```

...or, if possible, install it for a structure.

```rust
struct Sample<D = ShimPrimaryHandler, DD = ShimFallbackHandler>
where
    D: FallibleTryDropStrategy,
    DD: TryDropStrategy,
{
    primary: D,
    fallback: DD,
    /* other fields */
}

impl<D, DD> Sample<D, DD>
where
    D: FallibleTryDropStrategy,
    DD: TryDropStrategy,
{
    pub fn new_with(/* other arguments */ primary: D, fallback: DD) -> Self {
        Self {
            // filled arguments
            primary,
            fallback,
        }
    }
}

let sample = Sample::new_with(Strategy, /* other strategy */)
```

# License

This project is licensed under the MIT License.