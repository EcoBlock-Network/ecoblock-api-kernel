#[cfg(test)]
mod tests {
    use super::super::crypto::verify_ed25519_signature;
    use ed25519_dalek::{Keypair, Signer};

    #[test]
    fn verify_valid_signature() {
        let mut csprng = rand::rngs::OsRng{};
        let keypair = Keypair::generate(&mut csprng);
        let message = b"hello";
        let sig = keypair.sign(message);
        let res = verify_ed25519_signature(&keypair.public.to_bytes(), message, &sig.to_bytes());
        assert!(res.is_ok());
        assert!(res.unwrap());
    }

    #[test]
    fn verify_invalid_signature() {
        let mut csprng = rand::rngs::OsRng{};
        let keypair = Keypair::generate(&mut csprng);
        let message = b"hello";
        let mut sig_bytes = keypair.sign(message).to_bytes();
        sig_bytes[0] ^= 0xff;
        let res = verify_ed25519_signature(&keypair.public.to_bytes(), message, &sig_bytes);
        assert!(res.is_err());
    }
}
