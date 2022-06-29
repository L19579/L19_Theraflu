pub use {
    bs58,
	base64,
	bincode,
    colored::Colorize,
    rand::Rng,
    anyhow::{anyhow, Result},
//    ed25519_dalek::{self, Keypair},
    solana_sdk::{
        hash::Hash,
        pubkey::Pubkey,
        message::Message,
		transaction::Transaction,
        commitment_config::CommitmentConfig,
        signature::*,
        signer::{
            Signer, 
            keypair::Keypair,
            signers::Signers,
        }, 
    },
    solana_client::{
        rpc_config,
        rpc_client::RpcClient,
        tpu_client::{TpuClient, TpuClientConfig, DEFAULT_FANOUT_SLOTS},
        client_error::{self, ClientError},
        rpc_request::{self, TokenAccountsFilter},
    },
    hyper::{Client, Uri, header, Request, Method, Body, body::HttpBody as _},
    hyper_tls::HttpsConnector,
    serde::{Deserialize, Serialize},
    serde_json::{self, Value}, //self for serde_json::Result
    tokio::{self, time, runtime::Runtime},
    std::sync::{mpsc, Arc, Mutex},
    std::thread::{self, JoinHandle},
    std::time::Duration,
    std::collections::VecDeque,
    std::path::Path,
    std::panic,
    std::ops,
    toml,
    std::fmt,
    std::fs::OpenOptions,
    std::io::prelude::*,
    std::io::BufReader,
    chrono::{DateTime, prelude::*},
};

pub const CONFIG_FILE_PATH: &str = ""; // <--- insert direct path to config.toml here.
pub const MAXIMUM_FILE_LEN: usize = 15_000;

pub const USDC_MINT_KEY: &str = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";
pub const USDC_UNIT: u64 = 1000000; 
pub const SOL_UNIT: u64 = 1000000000;

pub const MAXIMUM_USDC_TRADE_GLOBAL: u64 = 0;//60000; // REMOVE

pub const RPC_CLIENT_LINK: &str = RPC_SERUM_LINK; // <-- std for low req rate.
//pub const RPC_CLIENT_LINK: &str = RPC_SSO_LINK;
//pub const RPC_CLIENT_LINK: &str = RPC_QUICKNODE_LINK;
//pub const RPC_CLIENT_LINK: &str = RPC_QUICKNODE_WSS_LINK;

pub const RPC_SSO_LINK: &str = "https://ssc-dao.genesysgo.net";
pub const RPC_SOLANA_LINK: &str = "https://api.mainnet-beta.solana.com";
pub const RPC_SERUM_LINK: &str = "https://solana-api.projectserum.com";
/*
pub const RPC_QUICKNODE_LINK: &str = "https://not-active.solana-mainnet.quiknode.pro/123/";
pub const RPC_QUICKNODE_WSS_LINK: &str = "wss://not-active.solana-mainnet.quiknode.pro/123/";
pub const RPC_QUICKNODE_LINK_TWO: &str = "https://not-active.solana-mainnet.quiknode.pro/123/";
pub const RPC_QUICKNODE_WSS_LINK_TWO: &str = "wss://not-active.solana-mainnet.quiknode.pro/123/";
*/

pub const USDC_LOT_SIZE: u64 = 1u64 * 10u64.pow(6);
pub const INSTRUCTION_LINK: &str = "https://quote-api.jup.ag/v1/swap";

pub const USDC_TOKEN_ACCOUNT: &str = "<<>>"; // User must have their USDC token account pubkkey here or mod Schemas, & config read
                                             // so the static is not required.
