# ATLAS

**A**lgebraic **T**ransform for **L**ow-constraint **A**rithmetic **S**tructures

A Rust implementation of the paper:

> **Constraint-Friendly Map-to-Elliptic-Curve-Group Relations and Their Applications**
> Groth, Malvai, Miller, Zhang (2025) — [ePrint 2025/1503](https://eprint.iacr.org/2025/1503)

---

## What is this?

Traditional hash-to-curve constructions combine an inner cryptographic hash
(SHA-256, Poseidon, MiMC) with an outer map-to-curve. The inner hash is
expensive in ZK circuits:

| Construction       | Constraints per invocation |
|--------------------|---------------------------|
| SHA-256            | ~7,095                    |
| Poseidon           | ~948                      |
| MiMC               | ~351                      |
| **ATLAS (this)**   | **~30**                   |

ATLAS bypasses the inner hash entirely. Security is proven in the
**Elliptic Curve Generic Group Model (EC-GGM)**.

---

## How it works

Given message `m ∈ M` and tweak bound `T`, the relation `R_M2G` is:
(m, (x,y), (k,z)) ∈ R_M2G  iff
k ∈ [0, T)
x = t + m*T
y = z²         (z is the sqrt witness)
(x, y) ∈ G     (valid curve point)
The tweak `k` becomes the ZK witness — constant time from the verifier's perspective.

---

## Crates

| Crate                | Description                              | Paper    |
|----------------------|------------------------------------------|----------|
| `atlas-core`         | Errors, field helpers, MemoryRecord      | §2       |
| `atlas-map-to-curve` | Increment-and-check relation             | §4       |
| `atlas-multiset-hash`| Multiset hash for zkVM memory checking   | §5       |
| `atlas-bls`          | Relational BLS signatures for zkPoS      | §6       |
| `atlas-circuits`     | Constraint cost comparison               | §7       |

---

## Curve

Uses the **Grumpkin** curve (`y² = x³ - 17`) for map-to-curve.
Grumpkin's base field = BN254's scalar field, making it
native to Barretenberg/Noir ZK circuits.

BLS signatures use BN254 G1/G2 with the full pairing check:
e(σ, g2) = e(hm, vk)
---

## Examples

```bash
# Map a message to a Grumpkin group element
cargo run --example map_to_group

# zkVM offline memory consistency checking
cargo run --example zkvm_memory

# Relational BLS signature (sign + verify with pairing)
cargo run --example bls_signature

# Multiset hash properties
cargo run --example multiset_hash

# Constraint cost comparison (Table 2 from paper)
cargo run --example constraint_cost
```

## Security
For zkVM memory checking (§5.4):

- Message space: |M| ≤ 2^100 (97-bit RISC-V records)
- Tweak bound: T = 256
- Security: >120 bits (Theorem 3)

For zkPoS BLS (§6.4):

- Message space: |M| ≤ 2^120
- Tweak bound: T = 128
- Security: ≥120 bits (Theorem 5)

### Tests
```bash
cargo test --workspace
```
---

## Circom Integration

ATLAS includes circom circuits and a Rust witness generator for use with
Groth16 proofs via snarkjs.

### Prerequisites

```bash
# Install circom
git clone https://github.com/iden3/circom.git
cd circom && cargo build --release && cargo install --path circom && cd ..

# Install snarkjs
npm install -g snarkjs
```

### Circuit Structure

```bash
circom/
├── map_to_curve.circom   ← 3 constraints from Figure 2
├── multiset_hash.circom  ← N × MapToCurve instances  
└── main.circom           ← ATLASMemoryCheck(10, 256)
```

### Constraints

```
template instances    : 3
non-linear constraints: 40   (3 per record × 10 + overhead)
public inputs         : 30   (messages[10] + xs[10] + ys[10])
private inputs        : 20   (zs[10] + ks[10])
```

### Run the Full Pipeline

**1. Download Powers of Tau**
```bash
cd circom
wget https://storage.googleapis.com/zkevm/ptau/powersOfTau28_hez_final_12.ptau \
     -O pot12_final.ptau
```

**2. Compile Circuit**
```bash
circom main.circom --r1cs --wasm --sym
```

**3. Generate Witnesses (Rust → JSON)**
```bash
cd ..
cargo run --example generate_witness
cd circom
node main_js/generate_witness.js main_js/main.wasm input.json witness.wtns
```

**4. Trusted Setup**
```bash
snarkjs groth16 setup main.r1cs pot12_final.ptau main_0000.zkey
snarkjs zkey contribute main_0000.zkey main_0001.zkey \
        --name="ATLAS" -e="atlas random entropy"
snarkjs zkey export verificationkey main_0001.zkey verification_key.json
```

**5. Prove & Verify**
```bash
snarkjs groth16 prove main_0001.zkey witness.wtns proof.json public.json
snarkjs groth16 verify verification_key.json public.json proof.json
# [INFO] snarkJS: OK! ✅
```

### Or Run Everything at Once

```bash
chmod +x circom/run.sh
./circom/run.sh
```

### License
MIT