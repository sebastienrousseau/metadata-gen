// SPDX-FileCopyrightText: 2024 - 2026 metadata-gen contributors
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Post-quantum signatures over front matter (feature `pqc-sign`).
//!
//! A [`Metadata`] map is turned into a deterministic byte string, its
//! *canonical encoding* (see [`canonical_bytes`] and
//! `docs/spec/canonical-encoding.md`), and that string is signed with
//! ML-DSA-65 as specified in FIPS 204, using the RustCrypto
//! [`ml_dsa`] crate. Signing uses FIPS 204's deterministic variant, so
//! the same key and the same metadata always give the same signature,
//! and no random number generator is needed. Every signature is bound
//! to the context string [`CONTEXT`], so it cannot be replayed as a
//! signature over some other kind of message under the same key.
//!
//! The feature is off by default; without it, no ML-DSA crate is in the
//! dependency tree.
//!
//! # Example
//!
//! ```
//! use metadata_gen::extract_metadata;
//! use metadata_gen::sign::{ml_dsa::Keypair, signing_key_from_seed};
//!
//! let key = signing_key_from_seed(&[7u8; 32]);
//! let public = key.verifying_key();
//! let metadata = extract_metadata("---\ntitle: Signed\n---\nbody").unwrap();
//!
//! let signature = metadata.sign(&key).unwrap();
//! assert!(metadata.verify(&public, &signature).is_ok());
//!
//! let mut tampered = metadata.clone();
//! tampered.insert("title".into(), "Changed".into());
//! assert!(tampered.verify(&public, &signature).is_err());
//! ```

use crate::{Metadata, MetadataMap};
use alloc::vec::Vec;
use core::fmt;

pub use ml_dsa;

/// The FIPS 204 parameter set used here: ML-DSA-65 (NIST category 3).
pub type Params = ml_dsa::MlDsa65;
/// An ML-DSA-65 signing (private) key.
pub type SigningKey = ml_dsa::SigningKey<Params>;
/// An ML-DSA-65 verifying (public) key.
pub type VerifyingKey = ml_dsa::VerifyingKey<Params>;
/// An ML-DSA-65 signature (3,309 bytes when encoded).
pub type Signature = ml_dsa::Signature<Params>;

/// The FIPS 204 context string every signature is bound to.
///
/// It names the encoding version: a future encoding gets a new context,
/// so a signature over one version never verifies as the other.
pub const CONTEXT: &[u8] = b"metadata-gen/canonical/v1";

/// The eight bytes every canonical encoding starts with.
pub const MAGIC: &[u8; 8] = b"MDGCAN01";

/// Signing failed. With the fixed, short [`CONTEXT`] this happens only
/// if ML-DSA's rejection sampling gives up, which FIPS 204 bounds to a
/// negligible probability.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SignError;

impl fmt::Display for SignError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ML-DSA signing failed")
    }
}

impl core::error::Error for SignError {}

/// The signature does not match the metadata and key: the metadata was
/// changed, the signature was made by another key, or it is corrupt.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VerifyError;

impl fmt::Display for VerifyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ML-DSA signature does not verify")
    }
}

impl core::error::Error for VerifyError {}

/// Derives a signing key from a 32-byte seed (FIPS 204
/// `ML-DSA.KeyGen_internal`).
///
/// The seed is the whole secret: keep it as you would the key. Generate
/// it from a cryptographically secure source; this crate does not
/// generate randomness.
///
/// # Example
///
/// ```
/// use metadata_gen::sign::{ml_dsa::Keypair, signing_key_from_seed};
///
/// let a = signing_key_from_seed(&[1u8; 32]);
/// let b = signing_key_from_seed(&[1u8; 32]);
/// assert_eq!(a.verifying_key(), b.verifying_key());
/// ```
#[must_use]
pub fn signing_key_from_seed(seed: &[u8; 32]) -> SigningKey {
    SigningKey::from_seed(&ml_dsa::Seed::from(*seed))
}

/// The canonical encoding of a metadata map: the bytes that are signed.
///
/// [`MAGIC`], then the entry count as a big-endian `u64`, then each
/// entry in ascending byte order of its key, as the key's length
/// (big-endian `u64`) and UTF-8 bytes followed by the value's length
/// and bytes. Keys and values are taken as they are, with no
/// normalisation, so two maps encode the same exactly when they hold
/// the same entries.
///
/// # Example
///
/// ```
/// use metadata_gen::sign::canonical_bytes;
/// use metadata_gen::MetadataMap;
///
/// let mut map = MetadataMap::new();
/// map.insert("a".to_string(), "1".to_string());
/// let bytes = canonical_bytes(&map);
/// assert_eq!(&bytes[..8], b"MDGCAN01");
/// assert_eq!(bytes.len(), 8 + 8 + (8 + 1) + (8 + 1));
/// ```
#[must_use]
pub fn canonical_bytes(map: &MetadataMap) -> Vec<u8> {
    let mut entries: Vec<(&str, &str)> =
        map.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect();
    entries.sort_unstable_by(|a, b| a.0.as_bytes().cmp(b.0.as_bytes()));
    let size = entries
        .iter()
        .map(|(k, v)| 16 + k.len() + v.len())
        .sum::<usize>();
    let mut out = Vec::with_capacity(MAGIC.len() + 8 + size);
    out.extend_from_slice(MAGIC);
    push_len(&mut out, entries.len());
    for (key, value) in entries {
        push_field(&mut out, key.as_bytes());
        push_field(&mut out, value.as_bytes());
    }
    out
}

fn push_len(out: &mut Vec<u8>, len: usize) {
    // usize is at most 64 bits on every target Rust supports.
    out.extend_from_slice(&(len as u64).to_be_bytes());
}

fn push_field(out: &mut Vec<u8>, bytes: &[u8]) {
    push_len(out, bytes.len());
    out.extend_from_slice(bytes);
}

impl Metadata {
    /// The canonical encoding of this metadata; see [`canonical_bytes`]
    /// (feature `pqc-sign`).
    ///
    /// # Example
    ///
    /// ```
    /// use metadata_gen::extract_metadata;
    ///
    /// let a = extract_metadata("---\nb: 2\na: 1\n---\n").unwrap();
    /// let b = extract_metadata("{\"a\": 1, \"b\": 2}").unwrap();
    /// assert_eq!(a.canonical_bytes(), b.canonical_bytes());
    /// ```
    #[must_use]
    pub fn canonical_bytes(&self) -> Vec<u8> {
        canonical_bytes(self.as_map())
    }

    /// Signs the canonical encoding with ML-DSA-65 (deterministic
    /// variant, context [`CONTEXT`]) (feature `pqc-sign`).
    ///
    /// # Errors
    ///
    /// [`SignError`] if ML-DSA's rejection sampling gives up, which
    /// FIPS 204 bounds to a negligible probability.
    ///
    /// # Example
    ///
    /// ```
    /// use metadata_gen::extract_metadata;
    /// use metadata_gen::sign::signing_key_from_seed;
    ///
    /// let key = signing_key_from_seed(&[9u8; 32]);
    /// let metadata = extract_metadata("---\ntitle: T\n---\n").unwrap();
    /// // Deterministic: the same key and metadata give the same signature.
    /// assert_eq!(metadata.sign(&key).unwrap(), metadata.sign(&key).unwrap());
    /// ```
    pub fn sign(
        &self,
        key: &SigningKey,
    ) -> Result<Signature, SignError> {
        key.expanded_key()
            .sign_deterministic(&self.canonical_bytes(), CONTEXT)
            .map_err(|_| SignError)
    }

    /// Checks `signature` against this metadata and `key` (feature
    /// `pqc-sign`).
    ///
    /// # Errors
    ///
    /// [`VerifyError`] if the signature does not match: the metadata
    /// changed, another key signed it, or the signature is corrupt.
    ///
    /// # Example
    ///
    /// ```
    /// use metadata_gen::extract_metadata;
    /// use metadata_gen::sign::{ml_dsa::Keypair, signing_key_from_seed};
    ///
    /// let key = signing_key_from_seed(&[9u8; 32]);
    /// let other = signing_key_from_seed(&[8u8; 32]);
    /// let metadata = extract_metadata("---\ntitle: T\n---\n").unwrap();
    /// let signature = metadata.sign(&key).unwrap();
    /// assert!(metadata.verify(&key.verifying_key(), &signature).is_ok());
    /// assert!(metadata.verify(&other.verifying_key(), &signature).is_err());
    /// ```
    pub fn verify(
        &self,
        key: &VerifyingKey,
        signature: &Signature,
    ) -> Result<(), VerifyError> {
        if key.verify_with_context(
            &self.canonical_bytes(),
            CONTEXT,
            signature,
        ) {
            Ok(())
        } else {
            Err(VerifyError)
        }
    }
}
