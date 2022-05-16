# LibD2D - A Communication and Task-Coordination Protocol for Autonomous Heterogeneous Swarms

This repository contains the code of the protocol developed as part of my bachelor thesis in Software Technology at Denmarks Technical University. It consists of a Rust package that compiles three distinct binaries, *operator*, *mothership*, and *minion*.

In order to run the code yourself please ensure you have the latest version of Rust and cargo installed. This can be done with the following:
```shell
curl https://sh.rustup.rs -sSf | sh
```

For correct operation, the binaries must be run in the order *mothership*, *minion*, *operator*. You can run multiple minions, up to six with the current implementation, but this can be extended by adding valid addresses to the operator [here](https://github.com/TheFoolishPupil/libd2d/blob/c45ecf459370aa13d0c5b1b3062db0da3ed3157e/src/bin/operator.rs#L62) and to the minion [here](https://github.com/TheFoolishPupil/libd2d/blob/c45ecf459370aa13d0c5b1b3062db0da3ed3157e/src/bin/minion.rs#L65). Please not that the addresses currently provided are not guaranteed to be available on your machine and so you may have to update them in any case.

Assuming you have correctly configured the addresses, you can run the binaries in separate terminal instances with the following:

```
cargo run --bin mothership

cargo run --bin minion

cargo run --bin operator
```

Unit tests for the library can be run with:
```
cargo test
```