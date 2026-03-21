#!/bin/bash
set -e

echo "=== ATLAS Circom Pipeline ==="

echo "1. Generating witnesses..."
cd ..
cargo run --example generate_witness
cd circom

echo "2. Compiling circuit..."
circom main.circom --r1cs --wasm --sym

echo "3. Generating witness..."
node main_js/generate_witness.js main_js/main.wasm input.json witness.wtns

echo "4. Trusted setup..."
snarkjs groth16 setup main.r1cs pot12_final.ptau main_0000.zkey
snarkjs zkey contribute main_0000.zkey main_0001.zkey --name="ATLAS" -e="atlas entropy"
snarkjs zkey export verificationkey main_0001.zkey verification_key.json

echo "5. Proving..."
snarkjs groth16 prove main_0001.zkey witness.wtns proof.json public.json

echo "6. Verifying..."
snarkjs groth16 verify verification_key.json public.json proof.json

echo "=== Done ==="