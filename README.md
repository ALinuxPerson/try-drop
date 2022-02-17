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

With this, if dropping `Foo` fails, it will automatically print the error to standard error.
