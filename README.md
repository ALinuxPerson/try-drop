# try-drop
Batteries included error handling mechanisms for drops which can fail

# Description
At the bare minimum, with no other features enabled, this crate is a collection of traits which make error handling on
fallible drops easier, although there is no built-in way to handle these errors, at least in this case.

With all (default) features enabled, this crate is collection of types and traits which not only allow you to make 
error handling on fallible drops easier, but contains a large set of *strategies*<sup>[1]</sup> on how to handle these
errors.

<sup>[1] Don't know what that means? don't worry, we'll get to that in a bit.</sup>

# Dependencies
At the bare minimum, there is only one dependency--`anyhow`. With all default features enabled, there are six 
dependencies.

# Features
Here is a tree of the features and their explanations.

  * `default`: Enables the global try drop strategy, downcasting of try drop strategies, standard library, newtype 
               derefs, `derives`s for most types, and the default try drop strategies.
  * `global`: This enables the global try drop strategy without nothing set to it. `OnceCell` is required for lazy 
              initialization of the global and parking lot to write to the global.
  * `std`: Enable types which require the standard library to work.
  * `derives`: Derives `Debug`, `Copy`, `Clone`, `PartialEq`, `Eq`, `PartialOrd`, `Ord`, `Hash`, and `Default` to all 
               public types if possible.
  * `drop-stratgies`: Enables the default drop strategies. Each drop strategy is explained below.
    * `ds-abort`: A drop strategy which aborts the current program if called.
    * `ds-broadcast`: A drop strategy which broadcasts the error to all receivers. This is a heavy drop strategy; it 
                      depends on the Tokio broadcast channel and therefore the runtime as it provides a good 
                      implementation of a broadcast channel, with the cost of overhead and code bloat.
    * `ds-exit`: A drop strategy which exits the program without calling any destructors with the specified exit code if 
                 called.
    * `ds-noop`: A drop strategy which does nothing when called.
    * `ds-panic`: A drop strategy which panics if called.
    * `ds-write`: A drop strategy which writes the error to a writer if called.
    * `ds-adhoc`: A drop strategy which calls a function if called. This only supports `Fn`s, which is the strictest 
                  type of function trait based on its trait bounds. If you want a less strict version, use...
      * `ds-adhoc-mut`: A drop strategy which calls a function if called. This supports `FnMut`s, which is has more 
                        flexible trait bounds compared to `Fn`, at the cost of using a `Mutex` internally, in order to
                        call it, since try drop strategies **must** take self by reference.
    * `ds-once-cell`: A drop strategy which stores an error value once. This is primarily useful for `struct`s which own
                      their drop strategy and is adjustable.

# License
This project is licensed under the MIT License.