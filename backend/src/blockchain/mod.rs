use crate::crypto::hash::sha256_hex;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub index: u64,
    pub timestamp: DateTime<Utc>,
    pub data: String,
    pub previous_hash: String,
    pub hash: String,
    pub nonce: u64,
    pub difficulty: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Blockchain {
    pub chain: Vec<Block>,
    pub difficulty: u8,
}

impl Block {
    /// Create a new block
    pub fn new(
        index: u64,
        data: String,
        previous_hash: String,
        difficulty: u8,
    ) -> Self {
        let timestamp = Utc::now();
        let mut block = Block {
            index,
            timestamp,
            data,
            previous_hash,
            hash: String::new(),
            nonce: 0,
            difficulty,
        };
        block.hash = block.calculate_hash();
        block
    }

    /// Calculate block hash
    pub fn calculate_hash(&self) -> String {
        let block_data = format!(
            "{}{}{}{}{:?}{}",
            self.index,
            self.timestamp,
            self.data,
            self.previous_hash,
            self.nonce,
            self.difficulty
        );
        sha256_hex(block_data.as_bytes())
    }

    /// Mine the block (proof-of-work)
    pub fn mine_block(&mut self) {
        let target = "0".repeat(self.difficulty as usize);
        
        while !self.hash.starts_with(&target) {
            self.nonce += 1;
            self.hash = self.calculate_hash();
        }
        
        tracing::info!(
            "Block mined: {} (nonce: {}, difficulty: {})",
            self.hash,
            self.nonce,
            self.difficulty
        );
    }

    /// Validate block integrity
    pub fn is_valid(&self) -> bool {
        let calculated_hash = self.calculate_hash();
        if calculated_hash != self.hash {
            tracing::warn!("Block hash mismatch: expected {}, got {}", self.hash, calculated_hash);
            return false;
        }

        let target = "0".repeat(self.difficulty as usize);
        if !self.hash.starts_with(&target) {
            tracing::warn!("Block hash does not meet difficulty requirement");
            return false;
        }

        true
    }
}

impl Blockchain {
    /// Create a new blockchain with genesis block
    pub fn new(difficulty: u8) -> Self {
        let mut blockchain = Blockchain {
            chain: Vec::new(),
            difficulty,
        };
        blockchain.create_genesis_block();
        blockchain
    }

    /// Create the genesis block
    fn create_genesis_block(&mut self) {
        let mut genesis = Block::new(
            0,
            "Genesis Block".to_string(),
            "0".to_string(),
            self.difficulty,
        );
        genesis.mine_block();
        self.chain.push(genesis);
        tracing::info!("Genesis block created");
    }

    /// Get the last block in the chain
    pub fn get_last_block(&self) -> Option<&Block> {
        self.chain.last()
    }

    /// Add a new block to the chain
    pub fn add_block(&mut self, mut block: Block) -> Result<(), String> {
        if let Some(last_block) = self.get_last_block() {
            if block.previous_hash != last_block.hash {
                return Err("Previous hash mismatch".to_string());
            }
            if block.index != last_block.index + 1 {
                return Err("Invalid block index".to_string());
            }
        }

        block.mine_block();

        if !block.is_valid() {
            return Err("Block validation failed".to_string());
        }

        self.chain.push(block);
        Ok(())
    }

    /// Validate the entire blockchain
    pub fn is_valid(&self) -> bool {
        if self.chain.is_empty() {
            return false;
        }

        // Check genesis block
        if !self.chain[0].is_valid() {
            tracing::warn!("Genesis block is invalid");
            return false;
        }

        if self.chain[0].previous_hash != "0" {
            tracing::warn!("Genesis block previous hash should be '0'");
            return false;
        }

        // Check all subsequent blocks
        for i in 1..self.chain.len() {
            let current_block = &self.chain[i];
            let previous_block = &self.chain[i - 1];

            if !current_block.is_valid() {
                tracing::warn!("Block {} is invalid", i);
                return false;
            }

            if current_block.previous_hash != previous_block.hash {
                tracing::warn!(
                    "Block {} has incorrect previous hash: expected {}, got {}",
                    i,
                    previous_block.hash,
                    current_block.previous_hash
                );
                return false;
            }

            if current_block.index != previous_block.index + 1 {
                tracing::warn!("Block {} has invalid index", i);
                return false;
            }
        }

        true
    }

    /// Get chain length
    pub fn length(&self) -> u64 {
        self.chain.len() as u64
    }

    /// Get block by index
    pub fn get_block(&self, index: u64) -> Option<&Block> {
        self.chain.get(index as usize)
    }

    /// Get all blocks
    pub fn get_chain(&self) -> &[Block] {
        &self.chain
    }
}

impl fmt::Display for Blockchain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Blockchain {{ length: {}, valid: {}, difficulty: {} }}",
            self.length(),
            self.is_valid(),
            self.difficulty
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_blockchain() {
        let blockchain = Blockchain::new(2);
        assert_eq!(blockchain.length(), 1);
        assert!(blockchain.is_valid());
    }

    #[test]
    fn test_add_block() {
        let mut blockchain = Blockchain::new(2);
        let last_hash = blockchain.get_last_block().unwrap().hash.clone();

        let mut new_block = Block::new(
            1,
            "Test Block 1".to_string(),
            last_hash,
            2,
        );
        
        assert!(blockchain.add_block(new_block).is_ok());
        assert_eq!(blockchain.length(), 2);
        assert!(blockchain.is_valid());
    }

    #[test]
    fn test_blockchain_validation() {
        let blockchain = Blockchain::new(1);
        assert!(blockchain.is_valid());
    }

    #[test]
    fn test_tampering_detection() {
        let mut blockchain = Blockchain::new(1);
        
        // Tamper with genesis block
        if let Some(block) = blockchain.chain.first_mut() {
            block.data = "Tampered".to_string();
        }

        assert!(!blockchain.is_valid());
    }
}
