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

# Description
At the bare minimum, with no other features enabled, this crate is a collection of traits which make error handling on
fallible drops easier, although there is no built-in way to handle these errors, at least in this case.

With all (default) features enabled, this crate is collection of types and traits which not only allow you to make 
error handling on fallible drops easier, but contains a large set of *strategies*<sup>\[1\]</sup> on how to handle these
errors.

<sup>\[1\] Don't know what that means? don't worry, we'll get to that in a bit.</sup>

# Motivation

Say you have a structure which performs external operations, to let's say a filesystem, which can fail.

```rust
use std::io;

struct Structure { /* fields */ }

impl Structure {
    pub fn do_external_io(&self) -> Result<(), io::Error> {
        // do some external io
    }
}
```

You then think to yourself, "What if I want to run that external IO at the end of a scope?"

Seems simple, you create a guard structure.

```rust
struct StructureGuard<'s> {
    structure: &'s Guard,
    // other fields
}

impl<'s> StructureGuard<'s> {
    pub fn new(structure: &'s Structure) -> Self {
        StructureGuard {
            structure,
            // other fields
        }
    }
}
```

You then proceed to write the `Drop` implementation for the guard.

```rust
impl<'s> Drop for StructureGuard<'s> {
    fn drop(&mut self) {
        if let Err(e) = self.structure.do_external_io() {
            // ...how can i handle this error?
        }
    }
}
```

But, alas! you need to handle the error.

First option: Just panic. But, we'd like to handle the error.

Second option: Print to standard error.

Second option: Create a finish `function` and then check in the `Drop` implementation if its finished.

```rust
struct StructureGuard<'s> {
    structure: &'s Structure,
    finished: bool,
    // other fields
}

impl<'s> StructureGuard<'s> {
    pub fn new(structure: &'s Structure) -> Self {
        StructureGuard {
            structure,
            finished: false,
            // other fields
        }
    }

    pub fn finish(&mut self) -> io::Result<()> {
        if self.finished {
            Ok(())
        } else {
            self.structure.do_external_io()?;
            self.finished = true;
        }
    }
}

impl<'s> Drop for StructureGuard<'s> {
    fn drop(&mut self) {
        if !self.finished {
            panic!("you must finish the guard before dropping it");
        }
    }
}
```

...but what if the user forgets to finish the guard? It would become a runtime error instead of a compile error.

And, what if an error occurs in another part of the code *before* the guard is finished?

```rust
{
    let guard = StructureGuard::new(&structure);
    do_some_thing_that_might_fail()?;
    guard.finish()?
}
```

Then it won't finish the guard, and it will panic.

This is exactly the problem I was having when I was making the [`ideapad`] project. In fact, this project was *spun off*
a module which was a part of the [`ideapad`] project.

I originally meant to only leave it in the [`ideapad`] project, but eventually I started needing this pattern in other
projects, so I decided to make it a standalone library.

# Overview
`try-drop` is based loosely off of how the x86 architecture handles exceptions, there is a primary exception handler,
and then there is an exception handler for the primary exception handler.

Except it isn't exception handlers, it's try drop strategies.

But what is a try drop?

## Try Drop
A try drop or a `TryDrop` is a trait which defines a destructor which can fail.

```rust
pub trait TryDrop {
    type Error;
    
    unsafe fn try_drop(&mut self) -> Result<(), Self::Error>;
}
```

You may have noticed that `TryDrop::try_drop` is unsafe. This is intentional:

See, the `Drop` trait has a special characteristic in that you can't call it's `drop` method directly. The reason is 
that you may call the destructor *twice*, which could potentially cause a double free.

```rust
struct T;

impl Drop for T {
    fn drop(&mut self) {
        // ...
    }
}

let t = T;
t.drop(); // this is not allowed and will result in a compile error
```

However, as far as I know, there is no way to implement this characteristic in your own traits. So you can do this if 
`try_drop` wasn't marked unsafe:

```rust
struct CWrapper(pub *mut some_library_type_t);

impl TryDrop for CWrapper {
    type Error = Error;
    
    fn try_drop(&mut self) -> Result<(), Self::Error> {
        let rc = c_wrapper_sys::some_library_type_t_free(self.0);
        
        if rc != 0 {
            Err(Error::from_rc(rc))
        } else {
            Ok(())
        }
    }
}

// meanwhile, in another crate...
fn do_stuff() -> Result<(), Error> {
    let mut c_wrapper = CWrapper(c_wrapper_sys::some_library_type_t_new());
    c_wrapper.try_drop()?;
    c_wrapper.try_drop()?; // oops!
    Ok(())
}
```

So we have to trust that the user will **never** call `TryDrop::try_drop` directly, and we want to make this invariant
explicit, so that's why I've made the `TryDrop::try_drop` method unsafe. You must only ever call this method from a
`Drop` implementation.

There *is* a way to make it safe, though.

### Drop Adapter
A drop adapter is basically the glue between the `Drop` trait and the `TryDrop` trait. This handles the error handling
for you.

```rust
fn do_stuff() {
    let c_wrapper = CWrapper(c_wrapper_sys::some_library_type_t_new());
    let c_wrapper = DropAdapter(c_wrapper);
    // let c_wrapper = c_wrapper.adapt(); // you can also do this
    // you can't cause a double free as `DropAdapter` doesn't implement `TryDrop`[1]

    // do stuff with c_wrapper
    
    // this results in a compile error as the method does not exist
    // c_wrapper.try_drop()?;
} // ... after this, the drop adapter handles the error

// [1]: well, kinda. we'll explain more later
```

But how can we specify *how* `DropAdapter` handles the error? Well...

# Try Drop Strategies
A try drop strategy or a `TryDropStrategy` is a trait which defines how to handle an error which occurs in a drop.

```rust
pub trait TryDropStrategy {
    fn handle_error(&self, error: try_drop::Error);
}
```

Simply implement it to your type, then boom, Bob's your uncle.

```rust
struct MyStrategy;

impl TryDropStrategy for MyStrategy {
    fn handle_error(&self, error: try_drop::Error) {
        println!("{}", error);
    }
}
```

A try drop strategy can also be *fallible*, meaning that it can fail.

```rust
pub trait FallibleTryDropStrategy {
    type Error;
    
    fn handle_error(&self, error: Self::Error) -> Result<(), Self::Error>;
}
```

Implementing it is like so:

```rust
impl FallibleTryDropStrategy for MyStrategy {
    type Error = io::Error;
    
    fn handle_error(&self, error: Self::Error) -> Result<(), Self::Error> {
        io::stdout().write_all(error.to_string().as_bytes())
    }
}
```

But what if *that* strategy fails? You can provide a last resort strategy.

# Fallback Try Drop Strategies
A fallback try drop strategy or a `FallbackTryDropStrategy` is a trait which defines how to handle an error which
occurs from another try drop strategy.

While the `FallbackTryDropStrategy` *does* exist, any type which implements `TryDropStrategy` will automatically
implement `FallbackTryDropStrategy` too. So `MyStrategy` already implements `FallbackTryDropStrategy`!

```rust
// yeah, cool, but like, how do i use the strategies in the drop adapter in the first place?
let c_wrapper = DropAdapter(CWrapper(c_wrapper_sys::some_library_type_t_new()));
```
There are two ways.

# Ways to use the strategies
## Provide it as a generic parameter
You can provide the drop strategies as a generic parameter for, in our case, `CWrapper`.

```rust
struct CWrapper<D, DD> 
where
    D: TryDropStrategy,
    DD: FallbackTryDropStrategy,
{
    inner: *mut some_library_type_t,
    strategy: D,
    fallback_strategy: DD,
}
```

And then you provide references to it in the `TryDrop` trait!

"...but there is no way to provide it," you may ask.

Well, *actually*, there are two version of the `TryDrop` trait.

### Impure Try Drop
This try drop, ignoring it's kinda ugly name, allows you to not provide any strategy.

```rust
pub trait ImpureTryDrop {
    type Error;
    
    unsafe fn try_drop(&mut self) -> Result<(), Self::Error>;
}
```

...seems familiar? that's because it's actually aliased to `TryDrop`!

### Pure Try Drop
This try drop, on the other hand, is a bit more verbose. You need to explicitly provide the strategies to use.

```rust
pub trait PureTryDrop {
    type Error;
    type FallbackTryDropStrategy;
    type TryDropStrategy;
    
    fn try_drop_strategy(&self) -> &Self::TryDropStrategy;
    fn fallback_try_drop_strategy(&self) -> &Self::FallbackTryDropStrategy;
    unsafe fn try_drop(&mut self) -> Result<(), Self::Error>;
}
```

We can now implement our drop strategy for `CWrapper`!

```rust
impl<D, DD> PureTryDrop for CWrapper<D, DD>
where
    D: TryDropStrategy,
    DD: FallbackTryDropStrategy,
{
    type Error = Error;
    type FallbackTryDropStrategy = DD;
    type TryDropStrategy = D;
    
    fn try_drop_strategy(&self) -> &Self::TryDropStrategy {
        &self.strategy
    }
    
    fn fallback_try_drop_strategy(&self) -> &Self::FallbackTryDropStrategy {
        &self.fallback_strategy
    }
    
    unsafe fn try_drop(&mut self) -> Result<(), Self::Error> {
        let rc = c_wrapper_sys::some_library_type_t_free(self.0);

        if rc != 0 {
            Err(Error::from_rc(rc))
        } else {
            Ok(())
        }
    }
}
```

Actually, all types that implement `ImpureTryDrop` also implement `PureTryDrop`!

> ...wait, what types are used then for `FallBackTryDropStrategy` and `TryDropStrategy`?

I'm glad you asked.

## Use it as the global drop strategy

By default, `try-drop` has the `global` feature enabled which, as you may have guessed, provides a global drop strategy.
However, it is only initialized if the `ds-write` (write drop strategy) and the `ds-panic` (panic drop strategy) 
features are enabled, which are the main drop strategy and fallback drop strategy respectively. You'd need to provide a 
drop strategy for each global if you don't have those features enabled.

We can install our `MyStrategy` as the main drop strategy like so:

```rust
try_drop::global::install(MyStrategy);
```

We can also install it as the fallback drop strategy:

```rust
try_drop::fallback::install(MyStrategy);
```

Going back to our (previous) `CWrapper`, it should already be using our global drop strategy.

# Other features
## Repeatable Try Drops
If you're 100% sure that your `TryDrop` implementation is safe, you can implement the `RepeatableTryDrop` marker trait.

```rust
pub unsafe trait RepeatableTryDrop: PureTryDrop {}
```

Implement it like so:

```rust
unsafe impl RepeatableTryDrop for T {}
```

With this, a safe version of `try_drop` will be provided.

```rust
// look ma, no `unsafe`!
let t = T;
t.safe_try_drop();
t.safe_try_drop();
```

Providing `DropAdapter` with this type will make `DropAdapter` implement `TryDrop` and `RepeatableTryDrop`, therefore 
allowing you to nest `DropAdapter`s.

```rust
let t = DropAdapter(DropAdapter(T));
```

# Dependencies
At the bare minimum, there is only one dependency--`anyhow`. With all default features enabled, there are six 
dependencies.

  * `anyhow`: used as the main error type.
  * `downcast-rs`: used to support downcasting the drop strategies in globals. Is optional.
  * `once_cell`: used to support the global drop strategy and for the OnceCell drop strategy. Is optional.
  * `shrinkwraprs`: used for more flexible newtypes. Is optional.
  * `tokio`: used for the broadcast drop strategy. Is optional.

# Features
Here is a tree of the features (which aren't optional dependencies) and their explanations.

  * `default`: Enables the global try drop strategy, downcasting of try drop strategies, standard library, newtype 
               derefs, `derives`s for most types, and the default try drop strategies.
  * `global`: This enables the global try drop strategy without nothing set to it. `OnceCell` is required for lazy 
              initialization of the global and parking lot to write to the global. By default, there are... *defaults*,
              which are...
    * `global-defaults`: This enables the default try drop and fallback try drop strategies, the write drop strategy 
                         and panic drop strategy.
  * `std`: Enable types which require the standard library to work.
  * `derives`: Derives `Debug`, `Copy`, `Clone`, `PartialEq`, `Eq`, `PartialOrd`, `Ord`, `Hash`, and `Default` to all 
               public types if possible.
  * `drop-strategies`: Enables the default drop strategies. Each drop strategy is explained below. Note that `ds` stands
                       for drop strategies, due to Rust's lack of feature namespacing (I think). You can find these drop
                       strategies in the `try_drop::drop_strategies` module.
    * `ds-abort`: A drop strategy which aborts the current program if called.
    * `ds-broadcast`: A drop strategy which broadcasts the error to all receivers. This is a heavy drop strategy; it 
                      depends on the Tokio broadcast channel and therefore the runtime as it provides a good 
                      implementation of a broadcast channel, with the cost of overhead and code bloat.
    * `ds-exit`: A drop strategy which exits the program without calling any destructors with the specified exit code if 
                 called.
    * `ds-noop`: A drop strategy which does nothing when called.
    * `ds-panic`: A drop strategy which panics if called. This is used as the default global fallback drop strategy if 
                  it's available.
    * `ds-write`: A drop strategy which writes the error to a writer if called. This is used as the default global drop 
                  strategy if it's available.
    * `ds-adhoc`: A drop strategy which calls a function if called. This only supports `Fn`s, which is the strictest 
                  type of function trait based on its trait bounds. If you want a less strict version, use...
      * `ds-adhoc-mut`: A drop strategy which calls a function if called. This supports `FnMut`s, which has more 
                        flexible trait bounds compared to `Fn`, at the cost of using a `Mutex` internally, in order to
                        call it, since try drop strategies **must** take self by reference.
    * `ds-once-cell`: A drop strategy which stores an error value once. This is primarily useful for `struct`s which own
                      their drop strategy and is adjustable.

# License
This project is licensed under the MIT License.

[`ideapad`]: https://github.com/ALinuxPerson/ideapad