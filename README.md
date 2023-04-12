# codex-zk

[![License: Apache](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)

WIP Zero Knowledge tooling for the Codex project

## License

Licensed and distributed under either of

* MIT license: [LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT

or

* Apache License, Version 2.0, ([LICENSE-APACHEv2](LICENSE-APACHEv2) or http://www.apache.org/licenses/LICENSE-2.0)

at your option. These files may not be copied, modified, or distributed except according to those terms.

## Usage

First, clone the repo and install the circom components:

```sh
git clone git@github.com:status-im/codex-storage-proofs.git
cd codex-storage-proofs
npm i
cd circuits
```

Nex, compile circuits:

```sh
../scripts/circuit_prep.sh
```

Running the tests:

```sh
cargo test # don't run in release more as it dissables circuit assets
```
