pub use {
    toml,
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
        rpc_request::{self, TokenAccountsFilter},
        client_error::{self, ClientError},
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
    std::fmt,
    std::fs::OpenOptions,
    std::io::prelude::*,
    std::io::BufReader,
    chrono::{DateTime, prelude::*},
};

pub const CONFIG_FILE_PATH: &str = "./src/config.toml"; // <---- User may need to modify this.

pub const USDC_UNIT: u64 = 1000000; //u64 to avoid casting.
pub const SOL_UNIT: u64 = 1000000000;

pub const MAXIMUM_USDC_TRADE_GLOBAL: u64 = 0;//60000; // REMOVE

pub const RPC_CLIENT_LINK: &str = RPC_SERUM_LINK;
//pub const RPC_CLIENT_LINK: &str = RPC_SSO_LINK;
//pub const RPC_CLIENT_LINK: &str = RPC_QUICKNODE_LINK;
//pub const RPC_CLIENT_LINK: &str = RPC_QUICKNODE_WSS_LINK;

pub const RPC_SSO_LINK: &str = "https://ssc-dao.genesysgo.net";
pub const RPC_SOLANA_LINK: &str = "https://api.mainnet-beta.solana.com";
pub const RPC_SERUM_LINK: &str = "https://solana-api.projectserum.com";
pub const RPC_QUICKNODE_LINK: &str = "https://ssc-dao.genesysgo.net"; // <---- Value modified for public repo.

pub const USER_PUBKEY: &str = "5jxxqUUQNCmDfpeHwdqioUeFnLFi37TepeSvum2xdKXH"; // REMOVE

pub const USDC_LOT_SIZE: u64 = 1u64 * 10u64.pow(6);
pub const QUOTE_LINK: [&str; 4] = ["https://quote-api.jup.ag/v1/quote?inputMint=", "&outputMint=", "&amount=", "&slippage=0.09&feeBps=4"];
pub const INSTRUCTION_LINK: &str = "https://quote-api.jup.ag/v1/swap";

// REMOVE below
pub const TOKENS: [(&str, &str, &str); 14] = [ 
    ("usdc", "USDC", "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",),
    ("usdt", "USDT", "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB",),
    ("sol", "SOL", "So11111111111111111111111111111111111111112"), // Wrapped Sol
    ("crystal", "CRY", "HbrmyoumgcK6sDFBi6EZQDi4i4ZgoN16eRB2JseKc7Hi",),
    ("bonfida", "FIDA", "EchesyfXePKdLtoiZSL8pBe8Myagyy8ZRqsACNCFGnvp"),
    ("step", "STEP", "StepAscQoEioFxxWGnh2sLBDFp9d8rvKz2Yp39iDpyT",),
    ("1sol", "1SOL", "4ThReWAbAVZjNVgs5Ui9Pk3cZ5TYaD9u6Y89fp6EFzoF"),
    ("raydium", "RAY", "4k3Dyjzvzp8eMZWUXbBCjEvwSkkk59S5iCNLY3QrkX6R"),
    ("avax", "AVAX", "KgV1GvrHQmRBY8sHQQeUKwTm2r2h8t4C8qt12Cw1HVE"),
    ("samo", "SAMO", "7xKXtg2CW87d97TXJSDpbD5jBkheTqA83TZRuJosgAsU"),
    ("cropper finance", "CRP", "DubwWZNWiNGMMeeQHPnMATNj77YZPZSAz2WVR5WjLJqz"),
    ("mongoose", "MONGOOSE", "J7WYVzFNynk9D28eBCccw2EYkygygiLDCVCabV7CupWL"),
    ("aurory", "AURY", "AURYydfxJib1ZkTir1Jn1J9ECYUtjb6rKQVmtYaixWPP"),
    ("FUMoney", "FUM", "7DcNieNEcC7UqPhaqyBU7a6H2CeWREHpTjw1sKiPqsDc"),
];
