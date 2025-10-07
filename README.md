# TFHE-rs Ciphertext differences when computing with AVX2 vs AVX512


This project provides a minimal test environment for inspecting **Radix-2 ciphertext internals (A mask and b body)** using **TFHE-rs**.  
It is intended for deterministic FHE debugging and ciphertext difference analysis between CPU and GPU backends.

---

## 1️⃣ Environment Setup

### **TFHE-rs Installation**
1. Clone the official TFHE-rs repository (version **1.4.0 or higher** is required):
   ```bash
   git clone https://github.com/zama-ai/tfhe-rs.git
   ```
2. The `tfhe-rs` folder must exist **at the same directory level** as this project.
   ```
   ├── tfhe-rs/
   └── tfhe-ct-test/
   ```

3. In this repo’s `Cargo.toml`, the dependency path should point to the TFHE crate inside the local clone:
   ```toml
   [dependencies]
   tfhe = { path = "../tfhe-rs/tfhe", features = ["integer"] }
   bincode = "1.3.3"
   base64  = "0.22"
   sha2    = "0.10"
   hex     = "0.4"
   ```

---

## 2️⃣ Required TFHE-rs Crate Modification

To access internal ciphertext data (for research/debugging purposes only),  
you must modify **`tfhe-rs/tfhe/src/integer/ciphertext/base_radix_ciphertext.rs`** as follows:

```diff
- pub(crate) struct BaseRadixCiphertext {
-     pub(crate) blocks: Vec<crate::shortint::Ciphertext>,
+ pub struct BaseRadixCiphertext {
+     pub blocks: Vec<crate::shortint::Ciphertext>,
```

Additionally, you may need to expose the inner LWE field in  
**`tfhe-rs/tfhe/src/shortint/ciphertext/mod.rs`**:

```diff
- pub(crate) struct Ciphertext {
-     pub(crate) lwe: LweCiphertextOwned<u64>,
+ pub struct Ciphertext {
+     pub lwe: LweCiphertextOwned<u64>,
```

> ⚠️ **Note:**  
> These edits are for internal ciphertext inspection only.  
> Do **not** commit modified TFHE-rs code to production or public forks.

---

## 3️⃣ Project Usage

### **Build**
```bash
RUSTFLAGS='-C target-cpu=haswell' cargo build --release
```

### **Run**
Two primary runtime flags are used in this project:

| Flag | Description |
|------|--------------|
| `SAVE=true` | Generates client/server keys, encrypts test inputs (123, 456), and stores ciphertexts. |
| `SAVE=false` | Loads existing ciphertexts and performs ciphertext addition. |
| `OUTPUT_MODE=base64` | Prints the resulting ciphertext serialized in Base64. |
| `OUTPUT_MODE=int64` | Prints the ciphertext as raw `i64` chunks (for plotting or diff analysis). |

Example:

```bash
# Generate and save test ciphertexts
SAVE=true cargo run --release

# Load and inspect ciphertext addition (Base64)
SAVE=false OUTPUT_MODE=base64 cargo run --release

# Load and inspect ciphertext addition (raw i64 output)
SAVE=false OUTPUT_MODE=int64 cargo run --release
```

---

## 4️⃣ Notes

- `FheUint32` uses **radix-2 decomposition**.  
  Each ciphertext consists of two short-integer LWE blocks.  
  After modification, their internal A (mask) and b (body) values can be printed for LSB comparison.

- For deterministic ciphertext equality tests, use:
  ```rust
  ConfigBuilder::default()
      .use_custom_parameters(MB_PARAMS.with_deterministic_execution())
      .build();
  ```

- Always ensure both CPU and GPU runs share identical parameters and key sets before comparing ciphertext byte-wise.

---

## 🧩 Directory Overview

```
tfhe-ct-test/
├── src/
│   └── main.rs        # Main test logic (key gen, addition, inspection)
├── data/              # Auto-created output folder for serialized ciphertexts
├── Cargo.toml
└── README.md
```

---

## ⚠️ Disclaimer

This repository is intended **solely for research and internal verification** of TFHE ciphertext behavior.  
Modifying internal crate visibility or analyzing raw LWE data should **never** be used in production or for security-sensitive environments.

---

**Author:** Seunghwan Lee (waLLLnut / CCRL in Hanyang University)  
**Environment:** Rust 1.75+, TFHE-rs ≥ 1.4.0  
**License:** Research / Internal Use Only
