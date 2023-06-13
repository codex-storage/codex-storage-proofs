#!/bin/bash

circom src/circuit_tests/poseidon-digest-test.circom --r1cs --wasm -o src/circuit_tests/artifacts
circom src/circuit_tests/poseidon-hash-test.circom --r1cs --wasm -o src/circuit_tests/artifacts
circom src/circuit_tests/storer-test.circom --r1cs --wasm -o src/circuit_tests/artifacts
