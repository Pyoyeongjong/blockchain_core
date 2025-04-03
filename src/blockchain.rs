#![allow(unused)]
use std::collections::{HashMap, HashSet};
use sha2::{Sha256, Digest};
use p256::ecdsa::{signature::{self, Signer, Verifier}, Signature, SigningKey, VerifyingKey};
use rand::Rng;
use hex;

use crate::{merkle_tree::MerkleTree, transaction::{self, Transaction}, utils::current_timestamp};

#[derive(Debug, Clone)]
pub struct BlockHeader {
    pub previous_hash: String,
    pub block_hash: String,
    pub merkle_tree: MerkleTree,
    pub timestamp: u128,
    pub nonce: u64,
    pub difficulty: usize,
}

impl BlockHeader {
    pub fn calculate_hash(&self) -> String {
        let mut hasher = Sha256::new();
        hasher.update(self.previous_hash.as_bytes());
        hasher.update(self.merkle_tree.root.as_bytes());
        hasher.update(self.timestamp.to_string().as_bytes());
        hasher.update(self.nonce.to_string().as_bytes());
        hex::encode(hasher.finalize())
    }
}

#[derive(Debug, Clone)]
pub struct Block {
    pub header: BlockHeader,
    pub transactions: Vec<Transaction>,
}

impl Block {
    pub fn new(previous_hash: String, transactions: Vec<Transaction>, difficulty: usize) -> Block {

        let timestamp = current_timestamp();
        let merkle_tree = MerkleTree::new(&transactions);
        let nonce = 0;

        let mut header = BlockHeader {
            previous_hash,
            block_hash: String::new(),
            merkle_tree,
            timestamp,
            nonce,
            difficulty
        };

        header.block_hash = header.calculate_hash();

        Block {
            header,
            transactions
        }
    }

    // PoW
    pub fn mine_block(&mut self) {

        let difficulty = self.header.difficulty;
        let target = "0".repeat(difficulty);

        while &self.header.block_hash[..difficulty] != target {
            self.header.nonce += 1;
            self.header.block_hash = self.header.calculate_hash();
        }

        println!("블록이 성공적으로 채굴되었습니다. Nonce: {}, Hash: {}",
            self.header.nonce, self.header.calculate_hash());

        self.header.block_hash = self.header.calculate_hash();
    }
}

#[derive(Debug)]
pub struct BlockChain {
    pub chain: Vec<Block>,
    pub accounts: HashMap<String, u64>,
    pub transaction_history: HashSet<String>,

    pub difficulty: usize,
    pub block_time: u128,
    pub adjustment_interval: usize,
}

impl BlockChain {
    pub fn new() -> BlockChain {
        let mut blockchain = BlockChain {
            chain: Vec::new(),
            accounts: HashMap::new(),
            transaction_history: HashSet::new(),
            difficulty: 3,
            block_time: 60000,
            adjustment_interval: 10
        };
        blockchain.add_genesis_block();
        blockchain
    }

    pub fn add_genesis_block(&mut self) {
        let root_tx = Transaction::new(
            "".to_string(),
            "".to_string(),
            0,
            0,
        );
        let genesis_block = Block::new(
            "0".to_string(),
            vec![root_tx],
            self.difficulty
        );
        self.chain.push(genesis_block);
    }

    pub fn add_block(&mut self, transactions: &[Transaction]) {
        let previous_block = self.chain.last().unwrap();
        let previous_hash = previous_block.header.calculate_hash();

        let mut new_block = Block::new(
            previous_block.header.block_hash.clone(),
            transactions.to_vec(),
            self.difficulty
        );

        new_block.mine_block();
        self.chain.push(new_block);

        println!("New block added: {:?}", self.chain.last());

        if self.chain.len() % self.adjustment_interval == 0 {
            self.adjust_difficulty();
        }
    }

    pub fn is_chain_valid(&self) -> bool {
        for i in 1..self.chain.len() {
            let cur_block = &self.chain[i];
            let prev_block = &self.chain[i-1];

            if cur_block.header.previous_hash != prev_block.header.block_hash {
                println!("Blockchain is invalid at block {}.", i);
                return false;
            }

            if cur_block.header.block_hash != cur_block.header.calculate_hash() {
                println!("Blockchain hash is invalid at block {}.",i);
                return false;
            }
        }
        println!("Blockchain is valid.");
        true
    }

    pub fn validate_transaction(&mut self, transaction: &Transaction, public_key: &VerifyingKey) -> bool {

        if receive_transaction(transaction).is_err() {
            println!("invalid signature.");
            return false;
        }

        if let Some(sender_balance) = self.accounts.get(&transaction.sender) {
            if sender_balance < &transaction.amount {
                println!("Insufficient funds.");
                return false;
            }
        } else {
            println!("Sender account not found.");
            return false;
        }

        let transaction_hash = transaction.calculate_hash().expect("The transaction is not signed.");

        if self.transaction_history.contains(&transaction_hash) {
            println!("Duplicate transaction.");
            return false;
        }

        self.transaction_history.insert(transaction_hash);
        println!("Transaction is valid.");
        true
    }

    pub fn adjust_difficulty(&mut self) {
        let expected_time = self.block_time;
        let actual_time = self.chain.last().unwrap().header.timestamp - self.chain[self.chain.len() - self.adjustment_interval].header.timestamp;

        if actual_time < expected_time {
            self.difficulty += 1;
        } else if actual_time > expected_time {
            self.difficulty -= 1;
        }

        println!("Adjusted difficulty: {}", self.difficulty);
    }

    pub fn apply_block(&mut self, block: &Block) {
        for tx in block.transactions.iter() {
            if let Some(sender_balance) = self.accounts.get_mut(&tx.sender) {
                *sender_balance -= tx.amount;
            }
            if let Some(receiver_balance) = self.accounts.get_mut(&tx.receiver) {
                *receiver_balance += tx.amount;
            }
        }
        println!("Blockchain state updated.");
    }
}

// 외부에서 signature가 정확한지 확인.
pub fn receive_transaction(transaction: &Transaction) -> Result<(), p256::ecdsa::Error> {
    let transaction_hash = transaction.calculate_hash_sign();

    let signature_bytes = hex::decode(
        transaction.signature.as_ref().expect("The transaction is not signed")
    ).expect("Invalid signature format.");

    let signature = Signature::from_slice(&signature_bytes).unwrap();
    transaction.public_key.expect("The transaction is not signed")
        .verify(transaction_hash.as_bytes(), &signature)
}


#[cfg(test)]
mod test {

    use crate::wallet::Wallet;

    use super::*;
    use p256::elliptic_curve::rand_core::OsRng;

    #[test]
    fn test_mine_block() {
        let mut blockchain = BlockChain::new();

        // 트랜잭션 생성
        let tx1 = Transaction {
            sender: "Alice".to_string(),
            receiver: "Bob".to_string(),
            amount: 50,
            fee: 0,
            signature: None,
            public_key: None,
            timestamp: current_timestamp(),
        };
    
        let txs = vec![tx1];
    
        // 20개의 블록을 추가하며 난이도 조정 테스트
        for _ in 0..20 {
            blockchain.add_block(&txs.clone());
        }
        // 블록체인 상태 확인
        for (i, block) in blockchain.chain.iter().enumerate() {
            println!("Block {}: {:?}", i, block);
        }
    }    

    fn test_receive_transaction() {
        let wallet = Wallet::new();
        let mut transaction = Transaction::new(String::from("A"), String::from("B"), 1000,1);
        wallet.sign_transaction(&mut transaction);

        assert!(receive_transaction(&transaction).is_ok());
    }

}