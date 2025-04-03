#![allow(unused)]
use std::collections::HashMap;

pub struct VirtualMachine;

impl VirtualMachine {

}

pub struct SmartContract {
    pub owner: String,
    pub balance: u64,
}

impl SmartContract {
    pub fn new(owner: String) -> SmartContract {
        SmartContract { 
            owner, 
            balance: 0 
        }
    }

    pub fn deposit(&mut self, amount: u64) {
        self.balance += amount;
        println!("Deposited {} to the contract. New balance: {}", amount, self.balance);
    }

    pub fn get_balance(&self) -> u64 {
        self.balance
    }
       
}