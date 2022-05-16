# LibD2D - A Communication and Task-Coordination Protocol for Autonomous Heterogeneous Swarms

This repository contains the code of the protocol developed as part of my bachelor thesis in Software Technology at Denmarks Technical University. It consists of a Rust package that compiles three distinct binaries, *operator*, *mothership*, and *minion*.

In order to run the code yourself please ensure you have the latest version of Rust and cargo installed. This can be done with the following:
```shell
curl https://sh.rustup.rs -sSf | sh
```

For correct operation, the binaries must be run in the order *mothership*, *minion*, *operator*. You can run multiple minions, up to six with the current implementation, but this can be extended by adding