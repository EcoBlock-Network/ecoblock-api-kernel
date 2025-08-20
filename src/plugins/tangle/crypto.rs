use ed25519_dalek::{PublicKey, Signature, Verifier};

pub fn verify_ed25519_signature(public_key_bytes: &[u8], message: &[u8], signature_bytes: &[u8]) -> Result<bool, ed25519_dalek::SignatureError> {
    let pk = PublicKey::from_bytes(public_key_bytes)?;
    let sig = Signature::from_bytes(signature_bytes)?;
    match pk.verify(message, &sig) {
        Ok(_) => Ok(true),
        Err(e) => Err(e),
    }
}
