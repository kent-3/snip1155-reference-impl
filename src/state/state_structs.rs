use super::*;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{
    Uint128, Addr,
};

use self::metadata::Metadata;

#[cfg(test)]
use crate::state::metadata::Extension;


/////////////////////////////////////////////////////////////////////////////////
// Contract and Token Id configs
/////////////////////////////////////////////////////////////////////////////////

/// contract configuration, spanning all `token_ids` generated by this contract
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ContractConfig {
    pub admin: Option<Addr>,
    /// These are contract-level curators that can curate new token_ids and mint initial balances. They cannot 
    /// mint additional tokens of existing token_ids, unless they are also minters of the specific
    /// fungible token
    pub curators: Vec<Addr>,
    pub token_id_list: Vec<String>,
    pub tx_cnt: u64,
    pub prng_seed: Vec<u8>,
    pub contract_address: Addr,
}

/// message sent my instantiator and curators for a specific `token_id`'s token info
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct TokenInfoMsg {
    pub token_id: String,
    pub name: String,
    pub symbol: String,
    pub token_config: TknConfig,
    pub public_metadata: Option<Metadata>,
    pub private_metadata: Option<Metadata>,
}

impl TokenInfoMsg {
    pub fn to_store(&self, curator: &Addr) -> StoredTokenInfo {
        StoredTokenInfo { 
            token_id: self.token_id.clone(), 
            name: self.name.clone(), 
            symbol: self.symbol.clone(), 
            token_config: self.token_config.clone(), 
            public_metadata: self.public_metadata.clone(), 
            private_metadata: self.private_metadata.clone(),
            curator: curator.clone(),
        }
    }
}

/// information for a specific `token_id`
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct StoredTokenInfo {
    pub token_id: String,
    pub name: String,
    pub symbol: String,
    pub token_config: TknConfig,
    pub public_metadata: Option<Metadata>,
    pub private_metadata: Option<Metadata>,
    pub curator: Addr,
}



#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum TknConfig {
    /// no `owner_may_update_metadata`because there can be multiple owners
    Fungible {
        minters: Vec<Addr>,
        /// Decimals play no part in the contract logic of the base specification of SNIP1155, 
        /// as there are no `deposit` and `redeem` features as seen in SNIP20. The UI application
        /// has discretion in handling decimals
        decimals: u8,
        public_total_supply: bool,
        enable_mint: bool,
        enable_burn: bool,
        minter_may_update_metadata: bool,
    },
    /// no `enable_mint` option because NFT can be minted only once using `CurateTokenIds`
    Nft {
        /// NFTs' minters cannot mint additional tokens, but may be able to change metadata
        minters: Vec<Addr>,
        /// total supply can be zero if the token has been burnt
        public_total_supply: bool,
        owner_is_public: bool,
        enable_burn: bool,
        owner_may_update_metadata: bool,
        minter_may_update_metadata: bool,
    }
}

impl TknConfig {
    /// Combines variables in the TknConfig enum into a single struct for easier handling in contract logic.
    pub fn flatten(&self) -> TknConfigFlat {
        match self {
            TknConfig::Fungible { 
                minters,
                decimals, 
                public_total_supply, 
                enable_mint, 
                enable_burn, 
                minter_may_update_metadata 
            } => {
                TknConfigFlat {
                    is_nft: false,
                    minters: minters.clone(),
                    decimals: *decimals,
                    public_total_supply: *public_total_supply,
                    owner_is_public: false,
                    enable_mint: *enable_mint,
                    enable_burn: *enable_burn,
                    minter_may_update_metadata: *minter_may_update_metadata,
                    /// there can be multiple owners, so owners cannot update metadata
                    owner_may_update_metadata: false,
                }
            },
            TknConfig::Nft { 
                minters,
                public_total_supply, 
                owner_is_public, 
                enable_burn, 
                owner_may_update_metadata, 
                minter_may_update_metadata
            } => {
                TknConfigFlat {
                    is_nft: true,
                    /// NFTs' minters cannot mint additional tokens, but may be able to change metadata
                    minters: minters.clone(),
                    decimals: 0_u8,
                    public_total_supply: *public_total_supply,
                    owner_is_public: *owner_is_public,
                    /// NFT can be minted only once using `CurateTokenIds`
                    enable_mint: false,
                    enable_burn: *enable_burn,
                    minter_may_update_metadata: *minter_may_update_metadata,
                    owner_may_update_metadata: *owner_may_update_metadata,
                }
            },
        } 
    }

    // note that default is normally `false`. These default to `true` is for unit testing purposes
    #[cfg(test)]
    pub fn default_fungible() -> Self {
        TknConfig::Fungible { 
            minters: vec![Addr::unchecked("addr0".to_string())],
            decimals: 6_u8,
            public_total_supply: true, 
            enable_mint: true,
            enable_burn: true, 
            minter_may_update_metadata: true, 
        }
    }

    #[cfg(test)]
    pub fn default_nft() -> Self {
        TknConfig::Nft { 
            minters: vec![],
            public_total_supply: true, 
            owner_is_public: true,
            enable_burn: true, 
            owner_may_update_metadata: true, 
            minter_may_update_metadata: true,
        }
    }
}

/// Constructed from input enum `TknConfig`. Flattened for easier handling in contract logic  
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct TknConfigFlat {
    pub is_nft: bool,
    pub minters: Vec<Addr>,
    pub decimals: u8,
    pub public_total_supply: bool,
    pub owner_is_public: bool,
    pub enable_mint: bool,
    pub enable_burn: bool,
    pub minter_may_update_metadata: bool,
    pub owner_may_update_metadata: bool,
}

impl TknConfigFlat {
    pub fn to_enum(&self) -> TknConfig {
        match self.is_nft {
            true => {
                TknConfig::Nft { 
                    minters: self.minters.clone(),
                    public_total_supply: self.public_total_supply, 
                    owner_is_public: self.owner_is_public, 
                    enable_burn: self.enable_burn, 
                    owner_may_update_metadata: self.owner_may_update_metadata,
                    minter_may_update_metadata: self.minter_may_update_metadata,
                }
            },
            false => {
                TknConfig::Fungible { 
                    minters: self.minters.clone(), 
                    decimals: self.decimals, 
                    public_total_supply: self.public_total_supply, 
                    enable_mint: self.enable_mint, 
                    enable_burn: self.enable_burn, 
                    minter_may_update_metadata: self.minter_may_update_metadata 
                }
            },
        }
    }
}


/////////////////////////////////////////////////////////////////////////////////
// Other structs, enums and functions
/////////////////////////////////////////////////////////////////////////////////

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CurateTokenId {
    pub token_info: TokenInfoMsg,
    pub balances: Vec<TokenIdBalance>,
}

#[cfg(test)]
impl Default for CurateTokenId {
    fn default() -> Self {
        Self { 
            token_info: TokenInfoMsg { 
                token_id: "0".to_string(), 
                name: "token0".to_string(), 
                symbol: "TKN".to_string(), 
                token_config: TknConfig::default_fungible(),
                public_metadata: Some(Metadata {
                    token_uri: Some("public uri".to_string()),
                    extension: Some(Extension::default()),
                }), 
                private_metadata: Some(Metadata {
                    token_uri: Some("private uri".to_string()),
                    extension: Some(Extension::default()),
                }),  
            }, 
            balances: vec![TokenIdBalance { 
                address: Addr::unchecked("addr0".to_string()), 
                amount: Uint128::from(1000_u64) 
            }],
        }
    }
}

/// used for MintToken and BurnToken in the base specifications
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct TokenAmount {
    pub token_id: String,
    /// For BurnToken, only `Balance.amount` is relevant. `Balance.address` need to be the 
    /// owner's address. This design decision is to allow `BurnToken` to apply to other addresses, 
    /// possible in the additional specifications
    pub balances: Vec<TokenIdBalance>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct TokenIdBalance {
    /// For BurnToken, `address` needs to be the owner's address. This design decision is 
    /// to allow `BurnToken` to apply to other addresses, possible in the additional 
    /// specifications
    pub address: Addr,
    pub amount: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct OwnerBalance {
    pub token_id: String,
    pub amount: Uint128,
}
