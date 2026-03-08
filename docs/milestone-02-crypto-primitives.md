# Milestone 2: Cryptographic Primitives

## Goal

Implement all core cryptographic operations in a single Rust crate (`glasuvai-crypto`) with **zero external dependencies**. The crate is `#[no_std]`-compatible so it compiles to both native (server/CLI) and WASM (browser). This is the mathematical foundation that every other component depends on. Every function must be traceable to a textbook definition.

## Prerequisites

- **M1**: Rust workspace initialized, election types defined

## Dependencies

**None.** The `glasuvai-crypto` crate has zero entries in `[dependencies]`. All cryptographic primitives — P-256 field arithmetic, SHA-256, RSA, baby-step giant-step — are implemented from first principles.

This is deliberate: the entire crypto path must be auditable without trusting any third-party code.

## Deliverables

```
packages/crypto/
  src/
    lib.rs                  # Crate root, re-exports modules
    biguint.rs              # Arbitrary-precision unsigned integer arithmetic
    biguint_test.rs         # Tests for BigUint
    field.rs                # P-256 prime field Fp (modular arithmetic)
    field_test.rs
    scalar.rs               # Scalar arithmetic mod N (curve order)
    scalar_test.rs
    point.rs                # P-256 affine/jacobian point operations
    point_test.rs
    sha256.rs               # SHA-256 from FIPS 180-4
    sha256_test.rs
    hash.rs                 # Domain-separated hashing (Fiat-Shamir)
    hash_test.rs
    rng.rs                  # CSPRNG interface (getrandom on native, Web Crypto on WASM)
    elgamal.rs              # Exponential ElGamal: keygen, encrypt, add, rerandomize
    elgamal_test.rs
    zkp.rs                  # Chaum-Pedersen DLOG equality proof
    zkp_test.rs
    zkp_01.rs               # Zero-knowledge proof that a ciphertext encrypts 0 or 1
    zkp_01_test.rs
    threshold.rs            # Pedersen DKG + Feldman VSS
    threshold_test.rs
    blind.rs                # RSA blind signatures
    blind_test.rs
```

## Mathematical Background

### P-256 Group

- Prime field: $p = 2^{256} - 2^{224} + 2^{192} + 2^{96} - 1$
- Curve order: $n = \text{0xFFFFFFFF00000000FFFFFFFFFFFFFFFFBCE6FAADA7179E84F3B9CAC2FC632551}$
- Generator: $G$ (standard NIST P-256 base point)
- Group operation: elliptic curve point addition
- Scalar multiplication: $k \cdot P$ for scalar $k$ and point $P$
- **Implemented from scratch**: field arithmetic, point add/double in Jacobian coordinates, constant-time scalar multiplication via double-and-add with complete formulas

### Exponential ElGamal

Unlike standard ElGamal (which encrypts messages as curve points), exponential ElGamal encrypts **integers** by encoding them as $m \cdot G$. This enables homomorphic addition but requires solving discrete log for decryption (feasible for small $m$ like vote counts).

## Data Structures

### Big Integer (`biguint.rs`)

```rust
/// Fixed-width unsigned integer stored as an array of 64-bit limbs.
/// All arithmetic is constant-time where needed for crypto operations.
///
/// We need at least 256-bit integers for P-256, and ~4096-bit for RSA.
/// The implementation is generic over the number of limbs.
#[derive(Clone, Debug)]
pub struct BigUint<const N: usize> {
    limbs: [u64; N], // little-endian: limbs[0] is the least significant
}

/// Type alias for 256-bit integers (P-256 field elements / scalars)
pub type U256 = BigUint<4>;

/// Type alias for 512-bit intermediate products
pub type U512 = BigUint<8>;

/// Type alias for RSA (3072-bit keys)
pub type U3072 = BigUint<48>;

impl<const N: usize> BigUint<N> {
    pub const ZERO: Self = Self { limbs: [0u64; N] };
    pub const ONE: Self = { /* limbs[0] = 1, rest 0 */ };

    /// Addition with carry, returns (result, carry)
    pub fn add_with_carry(&self, other: &Self) -> (Self, bool) { ... }

    /// Subtraction with borrow, returns (result, borrow)
    pub fn sub_with_borrow(&self, other: &Self) -> (Self, bool) { ... }

    /// Full multiplication: U256 × U256 → U512
    pub fn mul_wide(&self, other: &Self) -> BigUint<{N * 2}> { ... }

    /// Comparison (constant-time for crypto)
    pub fn ct_eq(&self, other: &Self) -> bool { ... }
    pub fn ct_lt(&self, other: &Self) -> bool { ... }

    /// Bit operations
    pub fn bit(&self, index: usize) -> bool { ... }
    pub fn leading_zeros(&self) -> u32 { ... }
    pub fn shr1(&self) -> Self { ... }  // shift right by 1
    pub fn shl1(&self) -> Self { ... }  // shift left by 1

    /// Serialization (big-endian bytes, matching SEC1 / RFC conventions)
    pub fn to_bytes_be(&self) -> [u8; N * 8] { ... }
    pub fn from_bytes_be(bytes: &[u8]) -> Self { ... }
}
```

### Field Element (`field.rs`)

```rust
/// Element of the P-256 prime field Fp.
/// All operations are performed modulo p.
/// Uses Montgomery multiplication for efficiency.
#[derive(Clone, Debug)]
pub struct FieldElement {
    inner: U256, // stored in Montgomery form: x * R mod p, where R = 2^256
}

/// The P-256 prime
pub const P: U256 = /* 2^256 - 2^224 + 2^192 + 2^96 - 1 */;

impl FieldElement {
    pub const ZERO: Self = /* ... */;
    pub const ONE: Self = /* R mod p (Montgomery form of 1) */;

    /// Modular addition: (a + b) mod p
    pub fn add(&self, other: &Self) -> Self { ... }

    /// Modular subtraction: (a - b) mod p
    pub fn sub(&self, other: &Self) -> Self { ... }

    /// Montgomery multiplication: (a * b) mod p
    pub fn mul(&self, other: &Self) -> Self { ... }

    /// Modular squaring (optimized path)
    pub fn square(&self) -> Self { ... }

    /// Modular inversion: a^(-1) mod p via Fermat's little theorem
    pub fn inv(&self) -> Self { ... }

    /// Modular negation: (-a) mod p = (p - a) mod p
    pub fn neg(&self) -> Self { ... }

    /// Square root mod p (Tonelli-Shanks; P-256 p ≡ 3 mod 4 so use (p+1)/4)
    pub fn sqrt(&self) -> Option<Self> { ... }

    /// Convert from/to canonical (non-Montgomery) representation
    pub fn from_bytes(bytes: &[u8; 32]) -> Option<Self> { ... }
    pub fn to_bytes(&self) -> [u8; 32] { ... }
}
```

### Point (`point.rs`)

```rust
/// A point on the P-256 curve in affine coordinates (x, y).
/// The identity (point at infinity) is represented as a separate variant.
#[derive(Clone, Debug)]
pub enum AffinePoint {
    Identity,
    Coordinates { x: FieldElement, y: FieldElement },
}

/// A point in Jacobian coordinates (X, Y, Z) where affine (x,y) = (X/Z², Y/Z³).
/// Used internally for efficient point arithmetic (avoids field inversions).
#[derive(Clone, Debug)]
pub struct JacobianPoint {
    x: FieldElement,
    y: FieldElement,
    z: FieldElement,
}

/// The P-256 generator point
pub const G: AffinePoint = /* standard NIST P-256 base point */;

impl AffinePoint {
    /// Convert to Jacobian for arithmetic
    pub fn to_jacobian(&self) -> JacobianPoint { ... }

    /// SEC1 uncompressed encoding: 0x04 || X (32 bytes) || Y (32 bytes)
    pub fn to_bytes_uncompressed(&self) -> [u8; 65] { ... }

    /// SEC1 compressed encoding: 0x02/0x03 || X (32 bytes)
    pub fn to_bytes_compressed(&self) -> [u8; 33] { ... }

    /// Deserialize from SEC1 encoding
    pub fn from_bytes(data: &[u8]) -> Option<Self> { ... }

    /// Check if this is the identity point
    pub fn is_identity(&self) -> bool { ... }
}

impl JacobianPoint {
    /// Point doubling: 2P (complete formula, handles P = identity)
    pub fn double(&self) -> Self { ... }

    /// Point addition: P + Q (complete formula, handles P = Q and P = -Q)
    pub fn add(&self, other: &Self) -> Self { ... }

    /// Fixed-base scalar multiplication: k·G
    /// Uses precomputed table for the generator
    pub fn scalar_base_mul(k: &Scalar) -> Self { ... }

    /// Variable-base scalar multiplication: k·P
    /// Uses constant-time double-and-add
    pub fn scalar_mul(&self, k: &Scalar) -> Self { ... }

    /// Convert back to affine (requires one field inversion)
    pub fn to_affine(&self) -> AffinePoint { ... }

    /// Negate: -(X, Y, Z) = (X, -Y, Z)
    pub fn neg(&self) -> Self { ... }
}
```

### Scalar (`scalar.rs`)

```rust
/// A scalar in the range [0, n-1] where n is the P-256 curve order.
#[derive(Clone, Debug)]
pub struct Scalar {
    inner: U256,
}

/// The P-256 curve order
pub const N: U256 = /* 0xFFFFFFFF00000000FFFFFFFFFFFFFFFFBCE6FAADA7179E84F3B9CAC2FC632551 */;

impl Scalar {
    pub const ZERO: Self = /* ... */;
    pub const ONE: Self = /* ... */;

    /// Generate a random scalar in [1, n-1]
    pub fn random() -> Self { ... }

    /// Addition: (a + b) mod n
    pub fn add(&self, other: &Self) -> Self { ... }

    /// Subtraction: (a - b) mod n
    pub fn sub(&self, other: &Self) -> Self { ... }

    /// Multiplication: (a * b) mod n
    pub fn mul(&self, other: &Self) -> Self { ... }

    /// Modular inverse: a^(-1) mod n
    pub fn inv(&self) -> Self { ... }

    /// Negation: (-a) mod n = (n - a) mod n
    pub fn neg(&self) -> Self { ... }

    /// Serialization
    pub fn to_bytes(&self) -> [u8; 32] { ... }
    pub fn from_bytes(bytes: &[u8; 32]) -> Option<Self> { ... }
}
```

### ElGamal (`elgamal.rs`)

```rust
/// An ElGamal key pair for exponential ElGamal on P-256.
pub struct Keypair {
    pub secret: Scalar,      // private key
    pub public: AffinePoint, // public key = secret·G
}

/// An exponential ElGamal ciphertext: (C1, C2).
/// Encrypts integer m as: C1 = r·G, C2 = m·G + r·PK
#[derive(Clone, Debug)]
pub struct Ciphertext {
    pub c1: AffinePoint, // randomness commitment
    pub c2: AffinePoint, // encrypted message
}

/// Generates a fresh ElGamal keypair.
pub fn generate_keypair() -> Keypair { ... }

/// Encrypts integer m ∈ {0, 1, ...} under public key pk.
/// Returns ciphertext and the randomness r (needed for ZKPs and Benaloh challenge).
pub fn encrypt(pk: &AffinePoint, m: u64, r: &Scalar) -> Ciphertext {
    // C1 = r·G
    // C2 = m·G + r·PK
    // If m == 0: C2 = r·PK (since 0·G = Identity)
    // If m == 1: C2 = G + r·PK
}

/// Homomorphically adds two ciphertexts: Enc(a) + Enc(b) = Enc(a+b)
pub fn add(a: &Ciphertext, b: &Ciphertext) -> Ciphertext {
    Ciphertext {
        c1: (a.c1.to_jacobian().add(&b.c1.to_jacobian())).to_affine(),
        c2: (a.c2.to_jacobian().add(&b.c2.to_jacobian())).to_affine(),
    }
}

/// Decrypts using the secret key to recover m·G, then solves DLOG for m.
/// Only works for small m (up to ~7,000,000 for vote counts).
pub fn decrypt(sk: &Scalar, ct: &Ciphertext) -> AffinePoint {
    // m·G = C2 - sk·C1
    let neg_sk_c1 = ct.c1.to_jacobian().scalar_mul(sk).neg();
    ct.c2.to_jacobian().add(&neg_sk_c1).to_affine()
}

/// Finds m such that m·G = target, using baby-step giant-step.
/// Returns m or error if m > max_m.
pub fn solve_dlog(target: &AffinePoint, max_m: u64) -> Option<u64> { ... }
```

### Zero-Knowledge Proofs (`zkp.rs`)

```rust
/// Chaum-Pedersen DLOG equality proof.
/// Proves that log_G(A) == log_H(B), i.e., ∃ x: A = x·G ∧ B = x·H.
/// Made non-interactive via Fiat-Shamir transform.
///
/// Used for: proving correct partial decryption by trustees.
pub struct DlogEqualityProof {
    pub commitment1: AffinePoint, // k·G
    pub commitment2: AffinePoint, // k·H
    pub response: Scalar,         // s = k - c·x (mod n), where c = Hash(...)
}

/// Creates a proof that log_G(a) == log_H(b) with witness x,
/// where a = x·G and b = x·H.
pub fn prove_dlog_equality(
    x: &Scalar,
    g: &AffinePoint,
    h: &AffinePoint,
    a: &AffinePoint,
    b: &AffinePoint,
) -> DlogEqualityProof {
    // 1. k ← random scalar
    // 2. R1 = k·G, R2 = k·H
    // 3. c = hash_to_scalar("dlog-eq", G, H, A, B, R1, R2)
    // 4. s = k - c·x (mod n)
    // 5. Return (R1, R2, s)
}

/// Verifies a DLOG equality proof.
pub fn verify_dlog_equality(
    proof: &DlogEqualityProof,
    g: &AffinePoint,
    h: &AffinePoint,
    a: &AffinePoint,
    b: &AffinePoint,
) -> bool {
    // 1. c = hash_to_scalar("dlog-eq", G, H, A, B, R1, R2)
    // 2. Check: s·G + c·A == R1
    // 3. Check: s·H + c·B == R2
}
```

### Zero-One Proof (`zkp_01.rs`)

```rust
/// Disjunctive Chaum-Pedersen proof that a ciphertext encrypts 0 or 1,
/// without revealing which.
///
/// Given ciphertext (C1, C2) = (r·G, m·G + r·PK) where m ∈ {0, 1}:
/// - If m=0: C2 = r·PK, so log_G(C1) == log_PK(C2)
/// - If m=1: C2 - G = r·PK, so log_G(C1) == log_PK(C2 - G)
///
/// The proof shows DLOG equality for ONE of these (the real one)
/// and simulates the other, without revealing which is real.
pub struct ZeroOneProof {
    // Branch 0 (m=0 case):
    pub r0: AffinePoint, // commitment
    pub c0: Scalar,      // challenge
    pub s0: Scalar,      // response
    // Branch 1 (m=1 case):
    pub r1: AffinePoint, // commitment
    pub c1: Scalar,      // challenge
    pub s1: Scalar,      // response
}

/// Creates a proof that the ciphertext encrypts 0 or 1.
/// `m` is the actual value (0 or 1), `r` is the encryption randomness.
pub fn prove_zero_one(
    pk: &AffinePoint,
    ct: &Ciphertext,
    m: u64,
    r: &Scalar,
) -> ZeroOneProof {
    // If m == 0:
    //   Real branch: prove log_G(C1) == log_PK(C2) with witness r
    //   Simulated branch (m=1): pick c1_fake, s1_fake randomly,
    //     compute R1_fake = s1_fake·G + c1_fake·C1
    //
    // If m == 1:
    //   Real branch: prove log_G(C1) == log_PK(C2-G) with witness r
    //   Simulated branch (m=0): pick c0_fake, s0_fake randomly,
    //     compute R0_fake = s0_fake·G + c0_fake·C1
    //
    // Overall challenge: c = hash_to_scalar("zk01", all commitments)
    //   c_real = c - c_fake (mod n)
    //   s_real = k - c_real·r (mod n)
}

/// Verifies a zero-one proof.
pub fn verify_zero_one(
    pk: &AffinePoint,
    ct: &Ciphertext,
    proof: &ZeroOneProof,
) -> bool {
    // 1. Recompute overall challenge c from commitments
    // 2. Check c0 + c1 == c (mod n)
    // 3. Check branch 0: s0·G + c0·C1 == R0 AND s0·PK + c0·C2 == R0'
    // 4. Check branch 1: s1·G + c1·C1 == R1 AND s1·PK + c1·(C2-G) == R1'
}
```

### Threshold Cryptography (`threshold.rs`)

```rust
/// One trustee's key material from DKG.
pub struct TrusteeShare {
    pub index: u32,              // Trustee index (1-based)
    pub secret_share: Scalar,    // This trustee's share of the election secret key
    pub public_share: AffinePoint, // secret_share·G (published for verification)
}

/// A trustee's Feldman commitment during DKG.
pub struct FeldmanCommitment {
    pub index: u32,                  // Trustee index
    pub commitments: Vec<AffinePoint>, // [a₀·G, a₁·G, ..., aₜ₋₁·G]
}

/// Performs round 1 of Pedersen DKG for one trustee:
/// - Generates random polynomial of degree (threshold-1)
/// - Computes Feldman commitments
/// - Computes shares for all other trustees
pub fn dkg_round1(
    trustee_index: u32,
    total_trustees: u32,
    threshold: u32,
) -> (Vec<Scalar>, FeldmanCommitment, Vec<(u32, Scalar)>) {
    // Returns: (polynomial coefficients, commitments, shares for each trustee)
}

/// Verifies a share received from trustee `sender` using their
/// published Feldman commitments.
pub fn dkg_round2_verify(
    received_share: &Scalar,
    my_index: u32,
    sender_commitments: &FeldmanCommitment,
) -> bool {
    // Check: received_share·G == Σ (j^k · C_k) for k=0..t-1
}

/// Computes a trustee's final secret share by summing all received shares.
pub fn dkg_finalize(
    my_polynomial_constant: &Scalar,
    received_shares: &[(u32, Scalar)],
) -> Scalar {
    // sum = a_{i,0} + Σ f_j(i) for all j ≠ i
}

/// Computes the election public key from all Feldman commitments.
pub fn combine_public_key(commitments: &[FeldmanCommitment]) -> AffinePoint {
    // PK = Σ C_{i,0} for all trustees i
}

/// Computes a trustee's partial decryption of an aggregated ciphertext.
pub fn partial_decrypt(
    share: &Scalar,
    public_share: &AffinePoint,
    ct: &Ciphertext,
) -> (AffinePoint, DlogEqualityProof) {
    // D_i = share_i · C1
    // Proof: log_G(public_share_i) == log_{C1}(D_i)
}

/// Combines t partial decryptions using Lagrange interpolation.
pub fn combine_partial_decryptions(
    partials: &[(u32, AffinePoint)], // (trustee_index, partial decryption)
    ct: &Ciphertext,
) -> AffinePoint {
    // Lagrange coefficients: λ_i = Π_{j≠i} (j / (j - i)) mod n
    // Combined: D = Σ λ_i · D_i
    // Plaintext point: m·G = C2 - D
}
```

### RSA Blind Signatures (`blind.rs`)

```rust
/// RSA key pair for blind signatures (used by the Identity Provider).
/// Implemented from scratch using BigUint arithmetic.
pub struct BlindSignatureKey {
    pub n: U3072,        // RSA modulus
    pub e: U3072,        // Public exponent (65537)
    pub d: U3072,        // Private exponent
    pub mir_id: u32,     // Which MIR this key is for
}

/// Generates a new RSA key pair for blind signing.
/// Uses 3072-bit modulus for ~128-bit security.
pub fn generate_blind_signature_key(mir_id: u32) -> BlindSignatureKey {
    // 1. Generate two random primes p, q of ~1536 bits each
    // 2. n = p * q
    // 3. φ(n) = (p-1)(q-1)
    // 4. e = 65537
    // 5. d = e^(-1) mod φ(n)
    //
    // Prime generation: random odd number + Miller-Rabin primality test
}

/// Blinds a message for the signer.
/// Returns the blinded message and the blinding factor r.
pub fn blind_message(
    n: &U3072,
    e: &U3072,
    message: &[u8],
) -> (Vec<u8>, U3072) {
    // 1. h = SHA256(message)
    // 2. Encode hash as integer m (full-domain-hash padding)
    // 3. Generate random r coprime to n
    // 4. blinded = m · r^e mod n
    // Return (blinded_bytes, r)
}

/// Signs a blinded message (performed by IdP — sees only blinded value).
pub fn blind_sign(d: &U3072, n: &U3072, blinded: &[u8]) -> Vec<u8> {
    // s' = blinded^d mod n
}

/// Removes the blinding factor to obtain a valid signature.
pub fn unblind_signature(
    n: &U3072,
    blind_sig: &[u8],
    r: &U3072,
) -> Vec<u8> {
    // s = s' · r^(-1) mod n
}

/// Verifies an unblinded signature.
pub fn verify_blind_signature(
    n: &U3072,
    e: &U3072,
    message: &[u8],
    signature: &[u8],
) -> bool {
    // signature^e mod n == H(message)
}
```

### SHA-256 (`sha256.rs`)

```rust
/// SHA-256 implementation per FIPS 180-4.
/// No external dependencies — implemented from the specification.
pub struct Sha256 {
    state: [u32; 8],     // H0..H7
    buffer: [u8; 64],    // Partial block buffer
    buffer_len: usize,
    total_len: u64,      // Total bytes processed
}

/// Round constants (first 32 bits of fractional parts of cube roots of first 64 primes)
const K: [u32; 64] = [ /* ... */ ];

/// Initial hash values (first 32 bits of fractional parts of square roots of first 8 primes)
const H_INIT: [u32; 8] = [
    0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a,
    0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19,
];

impl Sha256 {
    pub fn new() -> Self { ... }
    pub fn update(&mut self, data: &[u8]) { ... }
    pub fn finalize(self) -> [u8; 32] { ... }
}

/// Convenience function: compute SHA-256 of a byte slice.
pub fn sha256(data: &[u8]) -> [u8; 32] {
    let mut h = Sha256::new();
    h.update(data);
    h.finalize()
}
```

### Domain-Separated Hashing (`hash.rs`)

```rust
/// Computes SHA-256 with domain separation.
/// All Fiat-Shamir challenges use this to prevent cross-protocol attacks.
pub fn hash(domain: &str, data: &[&[u8]]) -> [u8; 32] {
    // h = SHA256(len(domain) as u32 || domain bytes || data[0] || data[1] || ...)
    let mut sha = Sha256::new();
    sha.update(&(domain.len() as u32).to_be_bytes());
    sha.update(domain.as_bytes());
    for d in data {
        sha.update(d);
    }
    sha.finalize()
}

/// Hashes to a scalar mod n (for Fiat-Shamir challenges in ZKPs).
pub fn hash_to_scalar(domain: &str, data: &[&[u8]]) -> Scalar {
    let h = hash(domain, data);
    Scalar::from_bytes_reduce(&h) // interpret as big-endian integer, reduce mod n
}
```

### Random Number Generation (`rng.rs`)

```rust
/// Fills a buffer with cryptographically secure random bytes.
/// 
/// - On native (std): uses `getrandom` syscall (Linux/macOS/Windows)
/// - On WASM: uses `crypto.getRandomValues()` via js-sys (injected by wasm-bindgen)
///
/// This is the ONLY platform-dependent code in the crate.
/// Configured via `#[cfg(target_arch = "wasm32")]` vs default.
pub fn random_bytes(buf: &mut [u8]) {
    #[cfg(not(target_arch = "wasm32"))]
    {
        // Use libc getrandom() or /dev/urandom
        // This is the one place we touch the OS directly
    }
    #[cfg(target_arch = "wasm32")]
    {
        // Use js_sys::crypto().get_random_values()
        // Provided by the wasm-bindgen environment
    }
}
```

> **Note on `#[no_std]`**: The crate uses `#![no_std]` at the top, with `extern crate alloc;` for `Vec` and `String`. The `std` feature enables native random number generation. On WASM, random bytes come from the JavaScript host.

## Implementation Steps

### Step 1: Big Integer Arithmetic

Implement `biguint.rs` first — this is the foundation for all modular arithmetic.

**Test**: 256-bit addition, subtraction, multiplication against known values. Verify `a * b / b == a` for random inputs. Test edge cases (overflow, zero, max value).

### Step 2: P-256 Field Arithmetic

Implement `field.rs` using Montgomery multiplication for efficiency.

**Test**: Verify known P-256 field operations. Test `a * a_inv == 1` for random elements. Test `sqrt` for known quadratic residues.

### Step 3: Point Operations

Implement `point.rs` with Jacobian coordinates. Use complete addition formulas that handle all edge cases (P + P, P + Identity, P + (-P)).

**Test**: Verify `1·G` = generator. Verify `2·G` = `G + G`. Verify `n·G` = Identity. Verify SEC1 serialization round-trips. Cross-check against known P-256 test vectors from RFC 5903.

### Step 4: Scalar Arithmetic

Implement `scalar.rs` — modular arithmetic in Z/nZ.

**Test**: Verify `a * a_inv == 1` for random scalars. Verify `random()` is in [1, n-1].

### Step 5: SHA-256

Implement `sha256.rs` per FIPS 180-4.

**Test**: Verify against NIST test vectors:
- `SHA256("")` = `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`
- `SHA256("abc")` = `ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad`
- Additional multi-block test vectors from the NIST CAVP suite.

### Step 6: Domain-Separated Hashing

Implement `hash.rs`. Critical for ZKP security — domain separation prevents cross-protocol attacks.

**Test**: Verify `hash("domain1", data) != hash("domain2", data)`. Known-answer vectors.

### Step 7: ElGamal Encryption

Implement `elgamal.rs`.

For `solve_dlog`, implement baby-step giant-step:
```
// Baby-step giant-step for solving m·G = target where 0 ≤ m ≤ max_m
// Time: O(√max_m), Space: O(√max_m)
//
// 1. s = ceil(√max_m)
// 2. Baby steps: compute table[j·G] = j for j = 0, 1, ..., s-1
// 3. Giant step: gs = (-s)·G
// 4. For i = 0, 1, ..., s-1:
//      γ = target + i·gs
//      if γ in table: return i·s + table[γ]
```

For the demo, max_m = 10,000 (max votes in a MIR) so √max_m ≈ 100. This is instant.

**Test**: Encrypt 0, decrypt → 0. Encrypt 1, decrypt → 1. Encrypt 42, decrypt → 42. Homomorphic: Enc(3) + Enc(7) decrypts to 10. Encrypt with known randomness, verify ciphertext matches hand computation.

### Step 8: Chaum-Pedersen DLOG Equality Proof

Implement `zkp.rs`.

**Test**: Generate proof, verify succeeds. Generate proof with wrong witness, verify fails. Verify Fiat-Shamir challenge is deterministic.

### Step 9: Zero-One Proof

Implement `zkp_01.rs`. This is the most complex single piece — a disjunctive proof.

**Test**: Prove m=0, verify succeeds. Prove m=1, verify succeeds. Try to prove m=2 → should fail. Verify that a proof for Enc(0) does not reveal that m=0.

### Step 10: Pedersen DKG + Threshold

Implement `threshold.rs`.

**Test full ceremony**:
1. 5-of-9 setup: 9 trustees each run `dkg_round1`
2. Each trustee receives shares from all others, verifies with `dkg_round2_verify`
3. Each computes final share with `dkg_finalize`
4. Compute combined public key with `combine_public_key`
5. Encrypt a message under the combined PK
6. 5 trustees provide partial decryptions (with proofs)
7. Combine partial decryptions → recover plaintext
8. Verify: any 5-of-9 combination works, any 4-of-9 fails

### Step 11: RSA Blind Signatures

Implement `blind.rs`.

**Test**: Full flow — blind, sign, unblind, verify. Verify the signer cannot determine the original message. Verify the unblinded signature is valid.

## Test Vectors (Generated and Committed)

Each operation produces test vectors saved to `test/vectors/`:

```
test/vectors/
  sha256.json                # NIST known-answer vectors
  p256_point_ops.json        # Known scalar multiplications on P-256
  elgamal_encrypt.json       # Known plaintext + randomness → expected ciphertext
  elgamal_homomorphic.json   # Two ciphertexts + expected sum
  zkp_dlog_equality.json     # Known witness + generators → proof components
  zkp_zero_one.json          # Enc(0) and Enc(1) proofs with known randomness
  threshold_dkg.json         # Full DKG ceremony with known seeds
  blind_signature.json       # Full blind signing flow with known blinding factor
```

These vectors are the **regression contract** — they ensure the single shared implementation remains correct across refactors. The same code runs on server and in WASM, so one test suite covers both.

## Acceptance Criteria

- [ ] `cargo test -p glasuvai-crypto` passes all tests
- [ ] `cargo tree -p glasuvai-crypto` shows zero external dependencies
- [ ] SHA-256: passes all NIST CAVP test vectors
- [ ] P-256: `n·G == Identity`, serialization round-trip, matches RFC 5903 vectors
- [ ] ElGamal: encrypt/decrypt round-trip for m = 0, 1, 100, 10000
- [ ] ElGamal: homomorphic addition correct for 100 random pairs
- [ ] ZKP DLOG equality: 1000 prove/verify cycles, all pass
- [ ] ZKP DLOG equality: forged proofs (wrong witness) rejected 100%
- [ ] ZKP 0/1: proofs for m=0 and m=1 both verify; m=2 cannot produce valid proof
- [ ] Threshold: 5-of-9 DKG + decrypt works for all C(9,5)=126 subsets
- [ ] Threshold: any 4-of-9 subset fails to decrypt
- [ ] Blind signatures: 100 blind/sign/unblind/verify cycles, all pass
- [ ] Test vectors generated and saved to `test/vectors/`
- [ ] Each function has a doc comment citing the mathematical definition
- [ ] Baby-step giant-step solves DLOG for m up to 10,000,000 in < 5 seconds
- [ ] Crate compiles with `--target wasm32-unknown-unknown` (no std dependency in core path)
