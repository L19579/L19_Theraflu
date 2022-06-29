use l19_theraflu as l19;

use std::thread::sleep;
use std::time::Duration;
use std::fs::{self, OpenOptions};
use std::io::{prelude::*, BufReader};

fn main() {
    let keypair = "/home/jojo/partition_3/programming_2/FULL/Solana_Arb/side_active/test2.json";

    let arb_paths_one = vec![
        ("usdc", "cry", "usdc"),
        ("usdc", "fida", "usdc"),
        ("usdc", "step", "usdc"),
        ("usdc", "samo", "usdc"),
        ("usdc", "mongoose", "usdc"),
        ("usdc", "orca", "usdc"), 
        ("usdc", "solape", "usdc"), 
        ("usdc", "wsftt", "usdc"),
        ("usdc", "crp", "usdc"),
        //("usdc", "wseth", "usdc"), 
        //("usdc", "wpeth", "usdc"), 
        ("usdc", "1sol", "usdc"), 
        ("usdc", "aury", "usdc"), 
        ("usdc", "liq", "usdc"), 
        ("usdc", "cope", "usdc"), 
        ("usdc", "dfl", "usdc"), 
        ("usdc", "trpy", "usdc"), 
        ("usdc", "slim", "usdc"), 
        ("usdc", "nirv", "usdc"), 
        ("usdc", "sunny", "usdc"), 
        ("usdc", "aca", "usdc"), 
        ("usdc", "fum", "usdc"), 
        ("usdc", "axax", "usdc"), 
        ("usdc", "ray", "usdc"), 
    ]; 
    // Create a config_2.toml w/ accs to enable these.
    // Run through alternate IP address to avoid rate
    // limits.
    /* 
    let arb_paths_two = vec![
        ("usdc", "dinoegg", "usdc"), 
        ("usdc", "msol", "usdc"), 
        ("usdc", "polis", "usdc"), 
        ("usdc", "trtls", "usdc"), 
        ("usdc", "solpad", "usdc"), 
        ("usdc", "wsbtc", "usdc"), 
        ("usdc", "gene", "usdc"), 
        ("usdc", "stsol", "usdc"), 
        ("usdc", "forge", "usdc"), 
        ("usdc", "shdw", "usdc"), 
        ("usdc", "crwny", "usdc"), 
        ("usdc", "slb", "usdc"), 
        ("usdc", "larix", "usdc"), 
        ("usdc", "kin", "usdc"), 
        ("usdc", "srm", "usdc"), 
        ("usdc", "dust", "usdc"), 
        ("usdc", "woof", "usdc"), 
        ("usdc", "wag", "usdc"), 
        ("usdc", "tulip", "usdc"), 
        ("usdc", "bitxbit", "usdc"), 
        ("usdc", "atlas", "usdc"), 
        ("usdc", "syp", "usdc"), 
        ("usdc", "bop", "usdc"), 
        ("usdc", "sny", "usdc"), 
        ("usdc", "ring", "usdc"), 
        ("usdc", "aart", "usdc"), 
    ];
    */ 
    
    let arb_options = l19::schemas::ArbOptions::SelectedTwoLeg(arb_paths_one);
    let cruel_summer = l19::CruelSummer::new(arb_options).unwrap();
    cruel_summer.launch_searchers(true);
}
