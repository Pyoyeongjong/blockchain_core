#![allow(unused)]
use p256::ecdsa::{signature::{Signer, Verifier}, Signature, SigningKey, VerifyingKey};
use sha2::{Sha256, Digest};
use crate::utils::current_timestamp;

#[derive(Debug, Clone)]
pub struct Transaction {
    pub sender: String,         // 지갑 주소 해시값
    pub receiver: String,
    pub amount: u64,
    pub fee: u64,
    pub signature: Option<String>,
    pub public_key: Option<VerifyingKey>,
    pub timestamp: u128
}

impl Transaction {
    pub fn new(sender: String, receiver: String, amount: u64, fee: u64) -> Transaction {
        Transaction { 
            sender, 
            receiver, 
            amount,
            fee,
            signature: None,
            public_key: None,
            timestamp: current_timestamp()
        }
    }

    // 서명을 포함하지 않아야 verify할 때 true가 나올 것.
    pub fn calculate_hash_sign(&self) -> String {
        let mut hasher = Sha256::new();
        hasher.update(self.sender.as_bytes());
        hasher.update(self.receiver.as_bytes());
        hasher.update(self.amount.to_string().as_bytes());
        hasher.update(self.timestamp.to_string().as_bytes());
        hex::encode(hasher.finalize())
    }

    pub fn calculate_hash(&self) -> Result<String, Box<dyn std::error::Error>> {

        // 서명한 transaction hash만 가능하게
        let mut hasher = Sha256::new();
        hasher.update(self.sender.as_bytes());
        hasher.update(self.receiver.as_bytes());
        hasher.update(self.amount.to_string().as_bytes());
        if let Some(signature) = self.signature.as_ref() {
            hasher.update(signature.to_string().as_bytes());
        } else {
            return Err("Not signed transaction".into());
        }
        hasher.update(self.timestamp.to_string().as_bytes());
        Ok(hex::encode(hasher.finalize()))
    }

    pub fn cmp_by_fee(&self, other: &Transaction) -> std::cmp::Ordering {
        other.fee.cmp(&self.fee)
    }
}

pub struct TransactionPool {
    // 검증되지 않은 트랜잭션들
    pub transactions: Vec<Transaction>,
}

impl TransactionPool {
    pub fn new() -> TransactionPool {
        TransactionPool { transactions: Vec::new() }
    }

    pub fn add_transaction(&mut self, tx: Transaction) {
        self.transactions.push(tx.clone());
        println!("Transaction added to pool: {:?}", tx);
    }

    pub fn select_transcations(&mut self, limit: usize) -> Vec<Transaction> {
        self.transactions.drain(..limit.min(self.transactions.len())).collect()
    }

    pub fn select_transcations_by_fee(&mut self, limit: usize) -> Vec<Transaction> {
        let mut transactions: Vec<Transaction> = self.transactions.drain(..).collect();
        transactions.sort_by(|a, b| a.cmp_by_fee(b));
        transactions.into_iter().take(limit).collect()
    }

    // 네트워크 혼잡도에 따른 동적 수수료
    pub fn dynamic_fee(&self, base_fee: u64, congestion_level: u64) -> u64 {
        base_fee + congestion_level * 2
    }
}