use std::{hash::Hash, mem};
use std::collections::HashMap;
use std::marker::PhantomData;
use borsh::{BorshDeserialize, BorshSerialize};
use pchain_types;
use pchain_sdk::{contract, action, view};

/// Note that this implementation is not at all gas-optimal, and is instead written for
/// terseness and clarity.
/// 
/// Known issue 1: applying #[contract(meta)] on impl EnglishCastles causes compiler to complain.
#[contract]
pub struct PRFC2Implementor {
    tokens: Map<String, Token>,
}

#[contract(meta)]
impl PRFC2Implementor {
    #[view]
    fn tokens_owned(&self, address: pchain_types::PublicAddress) -> u64 {
        let tokens = self.tokens.to_hash_map();
        tokens.values().filter(|token| token.owner == address).count() as u64
    }

    #[view]
    fn get_owner(&self, token_id: String) -> pchain_types::PublicAddress {
        let tokens = self.tokens.to_hash_map();
        tokens.get(&token_id).unwrap().owner
    }

    #[view]
    fn get_spender(&self, token_id: String) -> Option<pchain_types::PublicAddress> {
        let tokens = self.tokens.to_hash_map();
        tokens.get(&token_id).unwrap().spender
    }

    #[view]
    fn get_exclusive_spender(&self, for_address: pchain_types::PublicAddress) -> Option<pchain_types::PublicAddress> {
        let tokens = self.tokens.to_hash_map();
        let tokens_owned_by_for_address = tokens.values().filter(|token| token.owner == for_address);

        // Check if all tokens owned by for address has the same spender as first_token's
        let first_token = match tokens_owned_by_for_address.next() {
            None => return None,
            Some(token) => token,
        };

        if tokens_owned_by_for_address.into_iter().all(|token| Some(token.spender) == first_token.spender) {
            return first_token.spender
        } else {
            return None
        }
    }

    #[action]
    fn transfer(&self, token_id: String, to_address: pchain_types::PublicAddress) {
        let txn = pchain_sdk::Transaction::new();
        assert_eq!(txn.from_address, self.get_owner(token_id));

        let tokens = self.tokens.to_hash_map();
        let token = tokens.get_mut(&token_id).unwrap();
        token.owner = to_address;

        let event = TransferEvent {
            owner_address: txn.from_address,
            recipient_address: to_address,
            token_id,
        };
        pchain_sdk::emit_event(&event.topic(), &event.into_value());

        Self::set_tokens(Map::from_hash_map(&tokens));
    }

    #[action]
    fn transfer_from(&self, from_address: pchain_types::PublicAddress, to_address: pchain_types::PublicAddress, token_id: String) {
        let txn = pchain_sdk::Transaction::new();
        assert_eq!(txn.from_address, self.get_spender(token_id).unwrap());
        assert_eq!(from_address, self.get_owner(token_id));

        let tokens = self.tokens.to_hash_map();
        let token = tokens.get_mut(&token_id).unwrap();
        token.owner = to_address;

        let event = TransferEvent {
            owner_address: from_address,
            recipient_address: to_address,
            token_id
        };
        pchain_sdk::emit_event(&event.topic(), &event.into_value());

        Self::set_tokens(Map::from_hash_map(&tokens));
    }

    #[action]
    fn set_spender(&self, token_id: String, spender_address: Option<pchain_types::PublicAddress>) {
        let txn = pchain_sdk::Transaction::new();
        assert_eq!(txn.from_address, self.get_owner(token_id));
        assert!(self.get_exclusive_spender(txn.from_address).is_some() 
            && self.get_exclusive_spender(txn.from_address).unwrap() != spender_address);

        let tokens = self.tokens.to_hash_map();
        let token = tokens.get_mut(&token_id).unwrap();
        token.spender = spender_address;
        let event = SetSpenderEvent {
            spender_address,
            token_id,
            approved,
        };
        pchain_sdk::emit_event(&event.topic(), &event.into_value());

        Self::set_tokens(Map::from_hash_map(&tokens))
    
    }

    #[action]
    fn set_exclusive_spender(&self, spender_address: Option<pchain_types::PublicAddress>) {
        let txn = pchain_sdk::Transaction::new();

        let owners_tokens = self.tokens.to_hash_map();
        owners_tokens.values_mut().filter(|token| token.owner == txn.from_address).for_each(|token| token.spender = spender_address),

        let event = SetExclusiveSpenderEvent {
            owner_address: txn.from_address,
            spender_address,
        };

        pchain_sdk::emit_event(&event.topic(), &event.into_value())
    }
}

#[derive(BorshSerialize, BorshDeserialize, PartialEq, PartialOrd)]
struct Token {
    pub owner: pchain_types::PublicAddress,
    pub spender: Option<pchain_types::PublicAddress>,
}

struct Tokens {
    data: Vec<u8>,
} 

struct TransferEvent {
    owner_address: pchain_types::PublicAddress,
    recipient_address: pchain_types::PublicAddress,
    token_id: String,
}

struct SetSpenderEvent {
    spender_address: Option<pchain_types::PublicAddress>,
    token_id: String,
}

struct SetExclusiveSpenderEvent {
    owner_address: pchain_types::PublicAddress,
    spender_address: Option<pchain_types::PublicAddress>,
}

impl TransferEvent {
    fn topic(&self) -> [u8; 1] {
        [0]
    }

    fn into_value(self) -> Vec<u8> {
        let res = Vec::with_capacity(
            mem::size_of::<pchain_types::PublicAddress>() 
            + mem::size_of::<pchain_types::PublicAddress>() 
            + self.token_id.len());
        res.extend(self.owner_address);
        res.extend(self.recipient_address);
        res.extend(self.token_id.as_bytes());

        res
    }
}

impl SetSpenderEvent {
    fn topic(&self) -> [u8; 1] {
        [1]
    }

    fn into_value(self) -> Vec<u8> {
        let res = Vec::with_capacity(
            mem::size_of::<pchain_types::PublicAddress>() 
            + self.token_id.len()
            + 1
        );
        res.extend(self.spender_address);
        res.extend(self.token_id.as_bytes());
        res.extend(if self.approved { &[1] } else { &[0] });

        res
    }
}

impl SetExclusiveSpenderEvent {
    fn topic(&self) -> [u8; 1] {
        [2]
    }

    fn into_value(self) -> Vec<u8> {
        let res = Vec::with_capacity(
            mem::size_of::<pchain_types::PublicAddress>()
            + mem::size_of::<pchain_types::PublicAddress>()
            + 1
        );
        res.extend(self.owner_address);
        res.extend(self.spender_address);
        res.extend(if self.approved { &[1] } else { &[0] });

        res
    }

}

struct Map<K, V>
where K: BorshSerialize + BorshDeserialize + PartialOrd + Eq + Hash,
      V: BorshSerialize + BorshDeserialize {
    data: Vec<u8>,
    _phantom: PhantomData<K>,
    _phantom2: PhantomData<V>,
}

impl<K, V> Map<K, V>
where K: BorshSerialize + BorshDeserialize + PartialOrd + Eq + Hash,
      V: BorshSerialize + BorshDeserialize + PartialOrd {
    fn data(&self) -> &Vec<u8> { 
        &self.data
    }

    fn wrap(data: Vec<u8>) -> Self {
        Self {
            data,
            _phantom: PhantomData,
            _phantom2: PhantomData,
        }
    }

    fn new() -> Self {
        let mut data = vec![];
        HashMap::<K, V>::new().serialize(&mut data).unwrap();
        Self::wrap(data)
    }

    fn to_hash_map(&self) -> HashMap<K, V> {
        let mut data = self.data().as_slice();
        BorshDeserialize::deserialize(&mut data).unwrap()
    }

    fn from_hash_map(data: &HashMap<K, V>) -> Self {
        let mut buf = vec![];
        data.serialize(&mut buf).unwrap();
        Self::wrap(buf)
    }
} 
