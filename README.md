# runcc

[![Crates.io](https://img.shields.io/crates/v/runcc?style=for-the-badge)](https://crates.io/crates/runcc)
[![docs.rs](https://img.shields.io/docsrs/runcc/latest?style=for-the-badge)](https://docs.rs/runcc)
[![GitHub license](https://img.shields.io/github/license/runcc-rs/runcc?style=for-the-badge)](https://github.com/runcc-rs/runcc/blob/main/LICENSE)
[![GitHub stars](https://img.shields.io/github/stars/runcc-rs/runcc?style=for-the-badge)](https://github.com/runcc-rs/runcc/stargazers)

Run commands concurrently with rust and cargo

# Install

```sh
cargo install runcc
```

# Usage

- with a `runcc.{json, yaml, yml, ron, toml}` config file or
  `package.metadata.runcc` or `workspace.metadata.runcc` fields in `Cargo.toml`

  ```sh
  cargo runcc -c
  ```

- with cli arguments

  ```sh
  cargo runcc "command1" "command2 a b c"
  ```

For detailed usage, see [runcc/README.md](runcc/README.md) or `cargo runcc --help`
