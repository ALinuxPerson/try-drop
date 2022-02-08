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
