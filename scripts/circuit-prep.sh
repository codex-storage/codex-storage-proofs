#!/bin/bash

circom src/circuit_tests/poseidon-digest-test.circom --r1cs --wasm
circom src/circuit_tests/poseidon-hash-test.circom --r1cs --wasm
circom src/circuit_tests/storer-test.circom --r1cs --wasm
