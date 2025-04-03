mod merkle_tree;
mod blockchain;
mod transaction;
mod utils;
mod wallet;
mod node;
mod smart_contract;

//use merkle_tree::*;
use blockchain::*;
use transaction::*;
use wallet::Wallet;

// 아 self에 의존하는게 안 좋은 이유 -> self 내 필드를 다른 구조체로 이동해야 할 때, 수고로움이 크다!
fn main() {

    let wallet = Wallet::new();

    let mut transaction = Transaction::new(String::from("A"), String::from("B"), 1000,1);
    wallet.sign_transaction(&mut transaction);

    println!("{}", receive_transaction(&transaction).is_ok());
    
}

/* Node.rs 로 나눠질 애들 같음
----------------------------------------------------------------------------------------------- */


