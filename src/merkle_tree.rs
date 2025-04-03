#![allow(unused)]
use sha2::{Sha256, Digest};
use crate::transaction::Transaction;

#[derive(Debug, Clone)]
pub struct MerkleTree {
    pub root: String,
    pub leaf_nodes: Vec<String>,
}

impl MerkleTree {
    // &[] -> 슬라이스 참조
    pub fn new(transaction: &[Transaction]) -> MerkleTree {
        let leaf_nodes = transaction.iter().map(|tx| {
            Self::hash(&tx.calculate_hash().expect("The transactions is not signed"))
        }).collect::<Vec<String>>();

        let root = MerkleTree::calculate_root(&leaf_nodes);

        MerkleTree {
            root,
            leaf_nodes
        }
    }

    pub fn hash(data: &str) -> String{
        let mut hasher = Sha256::new();
        hasher.update(data.as_bytes());
        hex::encode(hasher.finalize())
    }

    pub fn calculate_root(leat_nodes: &[String]) -> String {
        let mut nodes = leat_nodes.to_vec();

        while nodes.len() > 1 {
            if nodes.len() % 2 != 0 {
                nodes.push(nodes.last().unwrap().clone());
            }

            let mut new_level = Vec::new();
            for i in (0..nodes.len()).step_by(2) {
                let combined = format!("{}{}", nodes[i], nodes[i + 1]);
                new_level.push(Self::hash(&combined));
            }
            nodes = new_level;
        }
        nodes[0].clone()
    }

    pub fn get_merkle_path(&self, tx_index: usize) -> Vec<String> {
        let mut path = Vec::new();
        let mut index = tx_index;
        let mut nodes = self.leaf_nodes.clone();

        while nodes.len() > 1 {
            if nodes.len() % 2 != 0 {
                nodes.push(nodes.last().unwrap().clone());
            }

            let sibling_index = if index % 2 == 0 { index + 1 } else { index - 1 };
            path.push(nodes[sibling_index].clone());

            index /= 2;
            nodes = nodes.chunks(2).map(|chunk| {
                let combined = format!("{}{}", chunk[0], chunk[1]);
                Self::hash(&combined)
            }).collect();
        }

        path
    }

    pub fn verify_transaction(&self, transaction: &Transaction, tx_index: usize, path: Vec<String>) -> bool {
        let mut hash = MerkleTree::hash(&transaction.calculate_hash().expect("The transactions is not signed"));
        let mut index = tx_index;

        for sibling_hash in path {
            if index % 2 == 0 {
                hash = Self::hash(&format!("{}{}", hash, sibling_hash));
            } else {
                hash = Self::hash(&format!("{}{}", sibling_hash, hash));
            }
            index /= 2;
        }

        println!("{:?} {:?}",hash, self.root);
        hash == self.root
    }
}

#[cfg(test)]
mod test {
    use p256::{ecdsa::{SigningKey, VerifyingKey}, elliptic_curve::rand_core::OsRng};

    use super::*;

    #[test]
    fn test_verify_transaction() {

        let private_key = SigningKey::random(&mut OsRng);
        let _public_key = VerifyingKey::from(&private_key);
        
        let mut tx1 = Transaction::new(String::from("A"), String::from("B"), 100, 0);
        let mut tx2 = Transaction::new(String::from("C"), String::from("D"), 10, 0);
        let mut tx3 = Transaction::new(String::from("E"), String::from("F"), 1, 0);

        let transactions = vec![tx1.clone(), tx2];

        let merkle_tree = MerkleTree::new(&transactions);
        let merkle_path = merkle_tree.get_merkle_path(0);

        assert!(merkle_tree.verify_transaction(&tx1, 0, merkle_path));
        
    }
}