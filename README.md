# ParallelChain F Request for Comments 2 (PRFC 2)

| PRFC | Title | Author | Version | Date First Published |
| --- | ----- | ---- | --- | --- |
| 2   | Non-Fungible Token Standard | ParallelChain Lab | 2 | July 23rd, 2022 | 

## Summary 
  
ParallelChain F Request for Comments 2 defines a standard interface for non-fungible tokens implemented as ParallelChain F smart contracts. "Non-Fungible Tokens" or NFTs is taken here to have the same meaning as in Ethereum's ERC-721, namely a set of transferable entities on a blockchain with identification metadata unique to its creator. For example, a deed for a plot of land is a non-fungible token, since a deed identifies a singular, unique plot of land (i.e., no two deeds identifies the same plot of land).

A standard contract interface for non-fungible tokens allows more seamless interoperability, since applications can make the simplifying assumption that all PRFC 2-implementing contracts always export the same, named set of Methods (they may export more).

The 'Required Methods' section lists the set of methods that all smart contracts that want to be PRFC 2-compliant must implement, as well as the behavior that each defined method must exhibit. Required behavior involves emitting certain events. These are also listed and described.

## Glossary

**Collection**: A set of Tokens with shared properties. A single PRFC 2-implementing contract represents a single Collection. For example, 'CryptoKitties' is a collection, whilst a single CryptoKitty is a Token.  

**Token**: An instance of a Collection.

**Spender**: An account that approved (using method `set_spender` or `set_exclusive_spender`) to transfer tokens on behalf of an owner. Any token can have at most one Spender.

**Exclusive Spender**: A spender that is approved to transfer *all* tokens owned by an owner. An account can be made an Exclusive Spender either through a single call to `set_exclusive_spender` (preferred), or multiple calls to `set_spender`.  

## Required types

```rust
type TokenID = String;
```

```rust
struct Collection {
    name: String,
    symbol: String,

    // Recommendation: uri should be an Internet URL, viewable on a browser.
    uri: Option<String>,
}
```

```rust
struct Token {
    id: TokenID,

    // Recommendation: uri should be an Internet URL, viewable on a browser.
    uri: Option<String>,
    owner: Option<PublicAddress>,
    spender: Option<PublicAddress>,
    exclusive_spender: Option<PublicAddress>
}
```

## Required Views 

The following uses syntax from Rust (version 1.59.0).

### `fn collection() -> Collection`

Returns information about the Collection represented by this contract.

### `fn token_ids() -> Vec<TokenID>` 

Returns the IDs of *all* tokens in this Collection. 

### `fn token_ids_of(owner: PublicAddress) -> Vec<TokenID>`

Returns the IDs of *all* tokens owned by `owner`.

### `fn tokens(ids: Vec<TokenID>) -> Vec<Token>`

Returns information about the Tokens identified by `ids`. 

If an ID does not identify a token, it must not appear in the returned vector. 

## Required Actions

### `fn transfer(token_id: TokenID, to_address: Option<PublicAddress>)`

'transfer' transfers the token identified by `token_id` from the account identified by `txn.from_address` (its owner), to the account identified by `to_address`. If `to_address` is None, this burns the token.

'transfer' must panic if get_owner(token_id) != `txn.from_address`.

Event `Transfer` must trigger if 'transfer' is successful.

### `fn transfer_from(from_address: PublicAddress, to_address: Option<PublicAddress>, token_id: TokenID)`

'transfer_from' transfers the token identified by `token_id` from the account identified by `from_address`, to the account identified by `to_address`, on behalf of the account identified by get_owner(token_id). If `to_address` is None, this burns the Token.

'transfer_from' must panic if: 
1. get_spender(token_id) != `txn.from_address`.
2. get_owner(token_id) != Some(`from_address`).
3. Or, if evaluating 1. or 2. causes a panic.

Event `Transfer` must trigger if ‘transfer_from’ is successful. 

### `fn set_spender(token_id: TokenID, spender_address: Option<PublicAddress>)`

If `spender_address` is Some, 'set_spender' gives the account identified by `spender_address` the right to transfer the token identified by `token_id` on behalf of its owner. Otherwise, it revokes `get_spender(token_id)` of its right.

'set_spender' must panic if:
1. get_owner(token_id) != `txn.from_address`.
2. get_exclusive_spender(`txn.from_address`) is Some and != `spender_address`.
3. Or, if evaluating 1. causes a panic.

Event `SetSpender` must trigger if 'set_spender' is successful.

### `fn set_exclusive_spender(spender_address: Option<PublicAddress>)`

If `spender_address` is Some, 'set_exclusive_spender' gives the account identified by `spender_address` the right to transfer *all* tokens owned by `txn.from_address`. Otherwise, it revokes `get_exclusive_spender(token_id)` of its right. Calling this method MUST have the same side effects (except events emitted) as calling `set_spender` for every `token_id` owned by `txn.from_address`, with the same `spender_address`.

Event `SetExclusiveSpender` must trigger if 'set_exclusive_spender' is successful.
     
## Required Events

In this section, `++` denotes bytes concatenation.

### `Transfer`

| Field | Value |
| ----- | ----- |
| Topic | `0u8 ++ token_id ++ owner_address ++ recipient_address: Option<PublicAddress>` |
| Value | Empty. |

Gets trigerred on successful call to methods 'transfer', or 'transfer_from'.

### `SetSpender`

| Field | Value |
| ----- | ----- |
| Topic | `1u8 ++ token_id + spender_address: Option<PublicAddress>` |
| Value | Empty. |

Gets triggered on successful call to method 'set_spender'.

### `SetExclusiveSpender`

| Field | Value |
| ----- | ----- |
| Topic | `2u8 ++ owner_address: Option<PublicAddress> ++ spender_address: Option<PublicAddress>` |
| Value | Empty. |

Gets triggered on successful call to method 'set_exclusive_spender'. 
