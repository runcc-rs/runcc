# Install

```sh
cargo install runcc
```

# Usage

You can use `cargo runcc --help` to see detailed help messages.

## with a config file

Use `-c` option to run with a config file

- If no config file is specified, runcc will auto look for `runcc.{json, yaml, yml, ron, toml}`
  and `package.metadata.runcc` or `workspace.metadata.runcc` fields in `Cargo.toml`
  in current working directory.

  ```sh
  cargo runcc -c
  ```

- If a directory is specified, runcc will look for those files in that directory

  ```sh
  cargo runcc -c .
  ```

- If a file is specified, runcc will auto recognize formats from file extension.
  `*.{json, yaml, yml, ron, toml}` and `Cargo.toml` are supported.

  ```sh
  cargo runcc -c my-config.yml
  ```

## with cli arguments

```sh
cargo runcc "command1" "command2 a b c"
```

# Implementation Details

- Why using tokio instead of `std::process::Command` and `std::thread`?

  `std::process::Command` doesn't support to be killed while being waited.
