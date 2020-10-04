# ppa

**Password protection app** - a CLI application that allows encrypted storage of usernames and passwords.

## Building

You'll need [Rust](https://www.rust-lang.org/). Having [just](https://github.com/casey/just) helps with development.

If you have both, then just running `just` in this root directory of this project checks and builds the program. If you
don't, then running `cargo check` and `cargo build` does (mostly) the same thing.

## Using

First, build the utility or get a binary release from [GitHub](https://github.com/Celeo/ppa).

The first command you'll need to run is `ppa init`, which takes in a 32-character password from you and initializes the
store. You'll need to remember this password!

Getting program usage information can be done through the help flags, `-h` and `--help`, like `ppa -h`.

## A note on security

You'll likely not want to use this for anything sensitive. Although the crypto library I'm using has undergone review,
this program **has not**. Use something like [Keepass](https://keepass.info/).

## License

Licensed under either of

* Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE))
* MIT license ([LICENSE-MIT](LICENSE-MIT))

## Contributing

Please feel free to contribute. Please open an issue first (or comment on an existing one) so that I know that you want to add/change something.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license,
shall be dual licensed as above, without any additional terms or conditions.
