extern crate time;
extern crate serde;
extern crate sha2;

use self::sha2::{Sha256, Digest};
use std::fmt::Write;
use serde::{Serialize};


#[derive(Serialize, Clone, Debug)]
struct Transaction{
    sender: String,
    reciever: String,
    amount: f32,
}

#[derive(Debug, Serialize)]
pub struct BlockHeader{
    timestamp: i64,
    nonce: u32,
    previous_hash: String,
    merkle: String,
    difficulty: u32,
}

#[derive(Debug, Serialize)]
pub struct Block{
    header: BlockHeader,
    count: u32,
    transactions: Vec<Transaction>,
}

pub struct Chain{
    chain: Vec<Block>,
    current_transaction: Vec<Transaction>,
    difficulty: u32,
    miner_address: String,
    reward: f32,
}

impl Chain{
    pub fn new(miner_address: String, difficulty: u32) -> Chain{
        let mut chain = Chain {
            chain: Vec::new(),
            current_transaction: Vec::new(),
            difficulty,
            miner_address,
            reward: 100.0,
        };

        chain.generate_new_block();
        chain
    }

    pub fn new_transaction(&mut self, sender: String, reciever: String, amount: f32) -> bool{
        self.current_transaction.push(
            Transaction{
                sender,
                reciever,
                amount,
            }
        );
        true
    }

    pub fn last_hash(&self) -> String{
        let block = match self.chain.last(){
            Some(block) => block,
            None =>  return String::from_utf8(vec![48; 64]).unwrap()
        };
        Chain::hash(&block.header)
    }

    pub fn set_difficulty(&mut self, difficulty: u32) -> bool{
        self.difficulty = difficulty;
        true
    }

    pub fn set_reward(&mut self, reward: f32) -> bool{
        self.reward = reward;
        true
    }

    pub fn generate_new_block(&mut self) -> bool{
        let header = BlockHeader{
            timestamp: time::now().to_timespec().sec,
            nonce: 0,
            previous_hash: self.last_hash(),
            merkle: String::new(),
            difficulty: self.difficulty,
        };

        let reward_transaction = Transaction{
            sender: String::from("Root"),
            reciever: self.miner_address.clone(),
            amount: self.reward,
        };

        let mut block = Block{
            header,
            count: 0,
            transactions: vec![]
        };

        block.transactions.push(reward_transaction);
        block.transactions.append(&mut self.current_transaction);
        block.count = block .transactions.len() as u32;
        block.header.merkle = Chain::get_merkle(block.transactions.clone());

        Chain::proof_of_work(&mut block.header);

        println!("{:#?}", &block);
        self.chain.push(block);
        
        true
    }

    fn get_merkle(current_transaction: Vec<Transaction>) -> String{
        let mut merkle = Vec::new();

        for t in &current_transaction{
            let hash = Chain::hash(t);
            merkle.push(hash);
        }

        if merkle.len()  & 1 == 1 {
            let last = merkle.last().cloned().unwrap();
            merkle.push(last);
        }

        while merkle.len() > 1{
            let mut h1 = merkle.remove(0);
            let mut h2 = merkle.remove(0);

            h1.push_str(&mut h2);
            let new_hash = Chain::hash(&h1);
            merkle.push(new_hash);
        }
        merkle.pop().unwrap()
    }


    // will keep generating hashes until we have as many leading zeroes.
    // i.e. if we have a difficulty of 2, then there need to be 2 leading zeros
    pub fn proof_of_work(header: &mut BlockHeader){
        loop{
            let hash = Chain::hash(header);
            let slice = &hash[..header.difficulty as usize];

            match slice.parse::<u32>(){
                Ok(val) => {
                    if val != 0 {
                        header.nonce += 1;
                    } else {
                        println!("Block hash: {}", hash);
                        break;
                    }
                },
                Err(_) => {
                    header.nonce += 1;
                    continue;
                }
            };
        }
    }

    pub fn hash<T: serde::Serialize>(item: &T) -> String{
        let input  = serde_json::to_string(&item).unwrap();
        let mut hasher = Sha256::default();

        hasher.input(input.as_bytes());
        let res = hasher.result();
        let vec_res = res.to_vec();

        Chain::hex_to_string(vec_res.as_slice())
    }

    pub fn hex_to_string(vec_res: &[u8]) -> String{
        let mut s = String::new();

        for b in vec_res{
            write!(&mut s, "{:x}", b).expect("unable to write");
        }
        s
    }
}