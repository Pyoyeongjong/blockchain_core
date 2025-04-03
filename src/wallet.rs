#![allow(unused)]
use sha2::{Sha256, Digest};
use p256::{ecdsa::{signature::{Signer, Verifier}, Signature, SigningKey, VerifyingKey}, elliptic_curve::rand_core::OsRng};

use crate::transaction::Transaction;

pub struct Wallet {
    private_key: SigningKey,
    pub public_key: VerifyingKey,
}

impl Wallet {
    pub fn new() -> Wallet {
        let private_key = SigningKey::random(&mut OsRng);
        let public_key = VerifyingKey::from(&private_key);

        Wallet { private_key, public_key }
    }

    pub fn generate_address(&self) -> String {
        // TODO: binding 왜 써야 했었지??
        let binding = self.public_key.to_encoded_point(false);
        let public_key_bytes = binding.as_bytes();
        let hashed_public_key = Wallet::hash_public_key(public_key_bytes);
        hex::encode(hashed_public_key)
    }

    pub fn hash_public_key(public_key: &[u8]) -> Vec<u8> {
        let mut hasher = Sha256::new();
        hasher.update(public_key);
        hasher.finalize().to_vec()
    }

    // 생성자의 개인키로 서명함으로써 생성자가 이 트랜잭션을 만들었다고 알린다.
    pub fn sign_transaction(&self, transaction: &mut Transaction) {
        let transaction_hash = transaction.calculate_hash_sign();
        let signature: Signature = self.private_key.sign(transaction_hash.as_bytes());

        transaction.signature = Some(hex::encode(signature.to_bytes()));
        transaction.public_key = Some(self.public_key.clone());
    }

    pub fn verify_signature(&self, transaction: &Transaction) -> bool {
        let transaction_hash = transaction.calculate_hash_sign();
        let signature_bytes = hex::decode(transaction.signature.clone().expect("Not signed transaction."))
            .expect("Invalid signature.");
        let signature = Signature::from_slice(&signature_bytes).unwrap();
        self.public_key.verify(transaction_hash.as_bytes(), &signature).is_ok()
    }

}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_create_wallet() {
        let wallet = Wallet::new();
        let address = wallet.generate_address();

        println!("Your wallet address: {}", address);
    }

    #[test]
    fn test_verify_transaction() {
        let wallet = Wallet::new();

        let mut transaction = Transaction::new(String::from("A"), String::from("B"), 1000,1);
        wallet.sign_transaction(&mut transaction);

        let is_valid = wallet.verify_signature(&transaction);
        assert!(is_valid);
    }
}