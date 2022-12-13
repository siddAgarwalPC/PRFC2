use std::{mem};
use borsh::{BorshDeserialize, BorshSerialize};
use pchain_types;
use pchain_sdk::collections::{IterableMap, Iterable};
use pchain_sdk::{contract, init, action, view, contract_methods, contract_field};

/// Note that this implementation is not at all gas-optimal, and is instead written for
/// terseness and clarity.
/// 
/// Known issue 1: applying #[contract(meta)] on impl EnglishCastles causes compiler to complain.
#[contract]
pub struct PRFC2Implementor {
    collection: Collection
}

#[contract_methods(meta)]
impl PRFC2Implementor {

    #[init]
    fn init(){
        let mut tokens = IterableMap::new();

        let all_tokens = 
        [   
            Token {
                id:"1".to_string(), 
                name:"Mystic duck".to_string(),
                uri:"https://cms.parallelchain.io/uploads/prfc2_kezia_duck_1a9f6c738e.png".to_string(),
                metadata:"".to_string(),
                owner: pchain_types::Base64URL::decode(&"-HcAllwug6CYTKIsATsa7yPk8wpNd4b7DBXkaD0kzko".to_string()).unwrap().try_into().unwrap(),
                spender: pchain_types::Base64URL::decode(&"-HcAllwug6CYTKIsATsa7yPk8wpNd4b7DBXkaD0kzko".to_string()).unwrap().try_into().unwrap(),
                exclusive_spender: pchain_types::Base64URL::decode(&"-HcAllwug6CYTKIsATsa7yPk8wpNd4b7DBXkaD0kzko".to_string()).unwrap().try_into().unwrap()
            },
            Token {
                id:"2".to_string(), 
                name:"Suburban dogs".to_string(),
                uri:"https://cms.parallelchain.io/uploads/prfc2_kezia_dogs_9155af2e83.png".to_string(),
                metadata:" ".to_string(),
                owner: pchain_types::Base64URL::decode(&"-HcAllwug6CYTKIsATsa7yPk8wpNd4b7DBXkaD0kzko".to_string()).unwrap().try_into().unwrap(),
                spender: pchain_types::Base64URL::decode(&"-HcAllwug6CYTKIsATsa7yPk8wpNd4b7DBXkaD0kzko".to_string()).unwrap().try_into().unwrap(),
                exclusive_spender: pchain_types::Base64URL::decode(&"-HcAllwug6CYTKIsATsa7yPk8wpNd4b7DBXkaD0kzko".to_string()).unwrap().try_into().unwrap()
            },
            Token {
                id:"3".to_string(), 
                name:"Bag'o Fish".to_string(),
                uri:"https://cms.parallelchain.io/uploads/prfc2_kezia_fish_59ff1d61a0.png".to_string(),
                metadata:"".to_string(),
                owner: pchain_types::Base64URL::decode(&"-HcAllwug6CYTKIsATsa7yPk8wpNd4b7DBXkaD0kzko".to_string()).unwrap().try_into().unwrap(),
                spender: pchain_types::Base64URL::decode(&"-HcAllwug6CYTKIsATsa7yPk8wpNd4b7DBXkaD0kzko".to_string()).unwrap().try_into().unwrap(),
                exclusive_spender: pchain_types::Base64URL::decode(&"-HcAllwug6CYTKIsATsa7yPk8wpNd4b7DBXkaD0kzko".to_string()).unwrap().try_into().unwrap()
            }
        ];
    
        for i in 0..all_tokens.len() {
            let token = all_tokens[i].clone();
            tokens.insert(&i.to_string(), token);
        }
    
        let collection = Collection{
            name:"Kezia's art collection".to_string(),
            symbol:"EKA".to_string(),
            tokens:tokens,
            uri:"".to_string()
        };
        PRFC2Implementor{
            collection: collection
        }.set()
    }

    #[view]
    fn collection(&self) -> Collection {
        let collection = self.collection.clone();
        collection
    }

    #[view]
    fn tokens_owned(&self, address: pchain_types::PublicAddress) -> Vec<Token> {
        let tokens = &self.collection.tokens;
        let iter = tokens.values().filter_map(|token| (token.owner == address).then_some(token)).collect();
        iter
    }

    #[view]
    fn get_owner(&self, token_id: String) -> pchain_types::PublicAddress {
        let tokens = &self.collection.tokens;
        tokens.get(&token_id).unwrap().owner
    }

    #[view]
    fn get_spender(&self, token_id: String) -> pchain_types::PublicAddress {
        let tokens = &self.collection.tokens;
        tokens.get(&token_id).unwrap().spender
    }

    #[view]
    fn get_exclusive_spender(&self, for_address: pchain_types::PublicAddress) -> pchain_types::PublicAddress {
        let tokens = &self.collection.tokens;
        let mut tokens_owned_by_for_address = tokens.values().filter(|token| token.owner == for_address);

        // Check if all tokens owned by for address has the same spender as first_token's
        let first_token = match tokens_owned_by_for_address.next() {
            None => return [0;32],
            Some(token) => token,
        };

        if tokens_owned_by_for_address.into_iter().all(|token| token.spender == first_token.spender) {
            return first_token.spender
        } else {
            return [0;32]
        }
    }

    #[action]
    fn transfer(&mut self, token_id: String, to_address: pchain_types::PublicAddress) {
        let txn_from_address = pchain_sdk::transaction::from_address();

        let mut token = self.collection.tokens.get_mut(&token_id).unwrap();
        token.owner = to_address;
        token.spender = to_address;
        token.exclusive_spender = to_address;

        let event = TransferEvent {
            owner_address: txn_from_address,
            recipient_address: to_address,
            token_id,
        };
        pchain_sdk::emit_event(&event.topic(), &event.into_value());
    }

    #[action]
    fn transfer_from(&mut self, from_address: pchain_types::PublicAddress, to_address: pchain_types::PublicAddress, token_id: String) {
        let txn_from_address = pchain_sdk::transaction::from_address();
        // assert_eq!(txn_from_address, self.get_spender(token_id).unwrap());
        // assert_eq!(from_address, self.get_owner(token_id));
        let token = self.collection.tokens.get_mut(&token_id).unwrap();
        token.owner = to_address;
        token.spender = to_address;
        token.exclusive_spender = to_address;

        let event = TransferEvent {
            owner_address: from_address,
            recipient_address: to_address,
            token_id
        };
        pchain_sdk::emit_event(&event.topic(), &event.into_value());
    }

    #[action]
    fn set_spender(&mut self, token_id: String, spender_address: pchain_types::PublicAddress) {
        let txn_from_address = pchain_sdk::transaction::from_address();
        // assert_eq!(txn_from_address, self.get_owner(token_id));
        // assert!(self.get_exclusive_spender(txn_from_address).is_some() 
        //     && self.get_exclusive_spender(txn_from_address).unwrap() != spender_address);
        let token = self.collection.tokens.get_mut(&token_id).unwrap();
        if token.owner == txn_from_address {
            token.spender = spender_address;
            let event = SetSpenderEvent {
                spender_address,
                token_id
            };
            pchain_sdk::emit_event(&event.topic(), &event.into_value());
        }
        
    
    }

    #[action]
    fn set_exclusive_spender(&mut self, spender_address: pchain_types::PublicAddress) {
        let txn_from_address = pchain_sdk::transaction::from_address();

        self.collection.tokens.values_mut().filter(|token| token.owner == txn_from_address).for_each(|token| token.spender = spender_address);

        let event = SetExclusiveSpenderEvent {
            owner_address: txn_from_address,
            spender_address,
        };

        pchain_sdk::emit_event(&event.topic(), &event.into_value());
    }
}

#[derive(BorshSerialize, BorshDeserialize, PartialEq, PartialOrd, Clone)]
struct Token {
    pub owner: pchain_types::PublicAddress,
    pub spender: pchain_types::PublicAddress,
    pub uri: String,
    pub metadata: String,
    pub name: String,
    pub id: String,
    pub exclusive_spender: pchain_types::PublicAddress
}

impl Iterable for Token {}

#[contract_field]
#[derive(BorshSerialize, BorshDeserialize, Clone)]
struct Collection {
    name: String,
    symbol: String,
    tokens: IterableMap<String, Token>,
    uri: String
} 

struct TransferEvent {
    owner_address: pchain_types::PublicAddress,
    recipient_address: pchain_types::PublicAddress,
    token_id: String,
}

struct SetSpenderEvent {
    spender_address: pchain_types::PublicAddress,
    token_id: String,
}

struct SetExclusiveSpenderEvent {
    owner_address: pchain_types::PublicAddress,
    spender_address: pchain_types::PublicAddress,
}

impl TransferEvent {
    fn topic(&self) -> Vec<u8> {
        let mut res = Vec::with_capacity(
            mem::size_of::<pchain_types::PublicAddress>() 
            + mem::size_of::<pchain_types::PublicAddress>() 
            + self.token_id.len());
        res.extend(self.owner_address);
        res.extend(self.recipient_address);
        res.extend(self.token_id.as_bytes());
        res
    }

    fn into_value(self) -> [u8; 1] {
        [0]
    }
}

impl SetSpenderEvent {
    fn topic(&self) -> Vec<u8> {
        let mut res = Vec::with_capacity(
            mem::size_of::<pchain_types::PublicAddress>() 
            + self.token_id.len()
        );
        res.extend(self.spender_address);
        res.extend(self.token_id.as_bytes());
        res
    }

    fn into_value(self) -> [u8; 1] {
        [1]
    }
}

impl SetExclusiveSpenderEvent {
    fn topic(&self) -> Vec<u8> {
        let mut res = Vec::with_capacity(
            mem::size_of::<pchain_types::PublicAddress>()
            + mem::size_of::<pchain_types::PublicAddress>()
        );
        res.extend(self.owner_address);
        res.extend(self.spender_address);
        res
    }

    fn into_value(self) ->  [u8; 1]{
        [2]
    }
}