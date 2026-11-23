// SPDX-FileCopyrightText: 2024 - 2026 metadata-gen contributors
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! ML-DSA-65 signing over the canonical encoding (feature `pqc-sign`).

#![cfg(feature = "pqc-sign")]

use metadata_gen::sign::{
    canonical_bytes, ml_dsa::Keypair, signing_key_from_seed, Signature,
    VerifyError, MAGIC,
};
use metadata_gen::{extract_metadata, Metadata, MetadataMap};

/// Decodes the base64 body of a PEM file (test-only, no padding checks
/// beyond what the fixtures need).
fn pem_der(pem: &str) -> Vec<u8> {
    let body: String = pem
        .lines()
        .filter(|l| !l.starts_with("-----"))
        .collect::<String>();
    let value = |c: u8| -> u32 {
        match c {
            b'A'..=b'Z' => u32::from(c - b'A'),
            b'a'..=b'z' => u32::from(c - b'a') + 26,
            b'0'..=b'9' => u32::from(c - b'0') + 52,
            b'+' => 62,
            b'/' => 63,
            other => panic!("not base64: {other}"),
        }
    };
    let mut out = Vec::new();
    for chunk in body.trim_end_matches('=').as_bytes().chunks(4) {
        let n = chunk.iter().enumerate().fold(0u32, |acc, (i, &c)| {
            acc | (value(c) << (18 - 6 * i))
        });
        let bytes = n.to_be_bytes();
        out.extend_from_slice(&bytes[1..chunk.len()]);
    }
    out
}

#[test]
fn key_from_lamps_seed_matches_published_public_key() {
    let seed_der =
        pem_der(include_str!("data/ml-dsa/ML-DSA-65-seed.priv"));
    let pub_der = pem_der(include_str!("data/ml-dsa/ML-DSA-65.pub"));

    let seed: [u8; 32] =
        seed_der[seed_der.len() - 32..].try_into().unwrap();
    let expected_seed: Vec<u8> = (0u8..32).collect();
    assert_eq!(seed.as_slice(), expected_seed.as_slice());

    let derived = signing_key_from_seed(&seed).verifying_key().encode();
    assert_eq!(
        derived.len(),
        1952,
        "ML-DSA-65 public keys are 1952 bytes"
    );
    assert_eq!(derived.as_slice(), &pub_der[pub_der.len() - 1952..]);
}

fn sample() -> Metadata {
    extract_metadata(
        "---\ntitle: Signed\nauthor:\n  name: Ada\n---\nbody",
    )
    .unwrap()
}

#[test]
fn sign_then_verify_round_trips() {
    let key = signing_key_from_seed(&[3u8; 32]);
    let metadata = sample();
    let signature = metadata.sign(&key).unwrap();
    assert_eq!(signature.encode().len(), 3309);
    assert_eq!(
        metadata.verify(&key.verifying_key(), &signature),
        Ok(())
    );
}

#[test]
fn signing_is_deterministic() {
    let key = signing_key_from_seed(&[3u8; 32]);
    let a = sample().sign(&key).unwrap();
    let b = sample().sign(&key).unwrap();
    assert_eq!(a.encode(), b.encode());
}

#[test]
fn tampering_is_detected() {
    let key = signing_key_from_seed(&[3u8; 32]);
    let public = key.verifying_key();
    let signature = sample().sign(&key).unwrap();

    let mut changed = sample();
    changed.insert("title".into(), "Signed!".into());
    assert_eq!(changed.verify(&public, &signature), Err(VerifyError));

    let mut added = sample();
    added.insert("draft".into(), "true".into());
    assert_eq!(added.verify(&public, &signature), Err(VerifyError));

    let other = signing_key_from_seed(&[4u8; 32]).verifying_key();
    assert_eq!(sample().verify(&other, &signature), Err(VerifyError));

    let mut bytes = signature.encode();
    bytes[100] ^= 0x01;
    if let Some(flipped) = Signature::decode(&bytes) {
        assert_eq!(
            sample().verify(&public, &flipped),
            Err(VerifyError)
        );
    }
}

#[test]
fn canonical_encoding_is_exact() {
    let mut map = MetadataMap::new();
    map.insert("b".to_string(), "xy".to_string());
    map.insert("a".to_string(), String::new());
    let mut expected = MAGIC.to_vec();
    expected.extend_from_slice(&2u64.to_be_bytes());
    expected.extend_from_slice(&1u64.to_be_bytes());
    expected.extend_from_slice(b"a");
    expected.extend_from_slice(&0u64.to_be_bytes());
    expected.extend_from_slice(&1u64.to_be_bytes());
    expected.extend_from_slice(b"b");
    expected.extend_from_slice(&2u64.to_be_bytes());
    expected.extend_from_slice(b"xy");
    assert_eq!(canonical_bytes(&map), expected);
}

#[test]
fn canonical_encoding_ignores_source_format_and_order() {
    let yaml = extract_metadata("---\nb: 2\na: 1\n---\n").unwrap();
    let toml = extract_metadata("+++\na = 1\nb = 2\n+++\n").unwrap();
    let json = extract_metadata("{\"b\": 2, \"a\": 1}").unwrap();
    assert_eq!(yaml.canonical_bytes(), toml.canonical_bytes());
    assert_eq!(yaml.canonical_bytes(), json.canonical_bytes());
}

#[test]
fn length_prefixes_prevent_boundary_collisions() {
    let mut one = MetadataMap::new();
    one.insert("ab".to_string(), "c".to_string());
    let mut two = MetadataMap::new();
    two.insert("a".to_string(), "bc".to_string());
    assert_ne!(canonical_bytes(&one), canonical_bytes(&two));
}
