# try-drop
Batteries included error handling mechanisms for drops which can fail

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
Second option: create a finish `function` and then check in the `Drop` implementation if its finished.

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

# Dependencies
At the bare minimum, there is only one dependency--`anyhow`. With all default features enabled, there are six 
dependencies.

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
                      for drop strategies, due to Rust's lack of feature namespacing (I think).
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