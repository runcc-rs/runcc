# runcc

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
