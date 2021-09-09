# 1.0.0 (2021-09-09)


### Bug Fixes

* **cli:** auto trim the starting runcc arg ([b10867a](https://github.com/runcc-rs/runcc/commit/b10867ab70accc803d1ee8e47cd51026f2d22b4f))
* **cli:** better help message ([f38fb9a](https://github.com/runcc-rs/runcc/commit/f38fb9a378a8d738038e88344f4feeae13f18bbb))
* **cli:** exit 2 if any command failed ([2bc2896](https://github.com/runcc-rs/runcc/commit/2bc2896ff7223a46e929538b3c776e246c9df584))
* better logs ([573acd7](https://github.com/runcc-rs/runcc/commit/573acd73ce214c11e60d8635426cfc8ecabd8a8b))
* break command system killer loop after all exited ([68e1731](https://github.com/runcc-rs/runcc/commit/68e1731352e59940f469a0c29bd8ce119b8b5292))
* remove impl Display for WindowsCallCmdWithEnv ([e2d255e](https://github.com/runcc-rs/runcc/commit/e2d255e4952e161daa0bf68c0094551798eb85cb))
* remove unused println ([56ceb91](https://github.com/runcc-rs/runcc/commit/56ceb910c6ac924bfae30e5247814f6536dabede))
* should always use platform shell to run script ([ac965fe](https://github.com/runcc-rs/runcc/commit/ac965fe6698e35060ca92c4a9e00fd09db19a0ce))


### Features

* **cli:** better exit message ([3dc75f1](https://github.com/runcc-rs/runcc/commit/3dc75f12e039cb1e5e2b3b413fa24ed279c00f55))
* **cli:** custom config file ordir ([2c22baa](https://github.com/runcc-rs/runcc/commit/2c22baa8b87f98e8155e09f351866bcfa30b69e7))
* **cli:** done ([f362668](https://github.com/runcc-rs/runcc/commit/f3626688019fb37518360b6ecb5157baafac4a00))
* **cli:** run with command system ([e90fec2](https://github.com/runcc-rs/runcc/commit/e90fec247d3017cab27416205f68931c82b27b0b))
* **lib:** command system report ([a54c2fe](https://github.com/runcc-rs/runcc/commit/a54c2fe1a3651aa315b6e9f8043cad0c8925aba5))
* consider command that failed to spawn as failed command in command system ([4c8112f](https://github.com/runcc-rs/runcc/commit/4c8112f91cb987f007a9b470592b4b5830854fdb))
* **lib:** cli ([158f00d](https://github.com/runcc-rs/runcc/commit/158f00deda95cadda6712c20ff8d864503bf9a43))
* **lib:** CommandConfig into tokio command ([5133d93](https://github.com/runcc-rs/runcc/commit/5133d93567bd55821bc31161fdcd98f3fe610bda))
* **lib:** config inputs ([0f3dc3c](https://github.com/runcc-rs/runcc/commit/0f3dc3c21c36a1af91fc3e29b16c6d53fb516b86))
* **lib:** find_config_file_in_cwd ([f4c78f8](https://github.com/runcc-rs/runcc/commit/f4c78f8a5518bb09d4bce7ccf2821490aca255c8))
* **lib:** impl Error and Display for ConfigDeserializeError ([74ce3fb](https://github.com/runcc-rs/runcc/commit/74ce3fb9865db76466d8b7cc9614dff3d223a73a))
* **lib:** kill behavior config ([bec3fb8](https://github.com/runcc-rs/runcc/commit/bec3fb804f038a1f7b1894b0e78f301870d37218))
* **lib:** label ([9d1ed9e](https://github.com/runcc-rs/runcc/commit/9d1ed9e9456a11f527a7a7dd15633db231640ea9))
* **lib:** match_program_with_envs ([b6397c4](https://github.com/runcc-rs/runcc/commit/b6397c4765e6f0460c581d6cfef4b08a60db0fd2))
* **lib:** run ([489fa21](https://github.com/runcc-rs/runcc/commit/489fa21c3b7ddeea1f3ba16574215e9d03fa364f))
* **lib:** run configs ([779ffb2](https://github.com/runcc-rs/runcc/commit/779ffb24a093904423a95ca2652157fa38a9a90f))
* **lib:** run with command system ([1038add](https://github.com/runcc-rs/runcc/commit/1038add0e266128a97e0a8305f0cea15b2c6e517))
* **lib:** RunConfig.envs ([2ae877b](https://github.com/runcc-rs/runcc/commit/2ae877b2d1152ddf8190dbc70d3ff9ce3616abbb))
* **lib:** windows_call_cmd_with_env ([a338c9d](https://github.com/runcc-rs/runcc/commit/a338c9dfb9186acd9aee3b1da70b94ff192dfe09))