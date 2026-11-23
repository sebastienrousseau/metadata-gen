// SPDX-FileCopyrightText: 2024 - 2026 metadata-gen contributors
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Signs front matter with ML-DSA-65, verifies it, and detects a
//! tampered copy (feature `pqc-sign`).
//!
//! Run with `cargo run --example pqc_sign_example --features pqc-sign`.

use metadata_gen::extract_metadata;
use metadata_gen::sign::{ml_dsa::Keypair, signing_key_from_seed};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // A fixed seed keeps the example reproducible. A real key comes
    // from a cryptographically secure random source and stays secret.
    let key = signing_key_from_seed(&[42u8; 32]);
    let public = key.verifying_key();

    let page = "---\ntitle: Release notes\nauthor: Ada\n---\n# Body";
    let metadata = extract_metadata(page)?;

    let signature = metadata.sign(&key)?;
    println!(
        "signed {} canonical bytes; signature is {} bytes",
        metadata.canonical_bytes().len(),
        signature.encode().len()
    );

    metadata.verify(&public, &signature)?;
    println!("original: signature verifies");

    let mut tampered = metadata.clone();
    tampered.insert("author".to_string(), "Mallory".to_string());
    match tampered.verify(&public, &signature) {
        Ok(()) => return Err("tampered metadata verified".into()),
        Err(e) => println!("tampered: rejected ({e})"),
    }
    Ok(())
}
