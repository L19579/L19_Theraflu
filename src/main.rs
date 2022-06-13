use l19_jup_fixed_three_leg as l19;

use std::thread::sleep;
use std::time::Duration;
use std::fs::{self, OpenOptions};
use std::io::{prelude::*, BufReader};

fn main() {
    //uncomment to allow route.
    let arb_paths = vec![
        //("usdc", "cry", "usdc"),
        //("usdc", "usdt", "usdc"),
        //("usdc", "fida", "usdc"),
        //("fida", "usdc", "fida"),
        ("usdc", "step", "usdc"),
        ("usdc", "samo", "usdc"),
        ("usdc", "mongoose", "usdc"),
        ("usdc", "orca", "usdc"), 
        ("usdc", "solape", "usdc"), 
        //("usdc", "ftt", "usdc"),
        //("usdc", "crp", "usdc"), // 2c lost that 1 time.
        ("usdc", "eth", "usdc"), 
        //("usdc", "1sol", "usdc"), 
        //("usdc", "aury", "usdc"), 
        //("usdc", "liq", "usdc"), 
        //("usdc", "cope", "usdc"), 
        ("usdc", "dfl", "usdc"), 
        //("usdc", "trpy", "usdc"), 
        //("usdc", "slim", "usdc"), 
        //("usdc", "nirv", "usdc"), 
        //("usdc", "sunny", "usdc"), 
        //("usdc", "aca", "usdc"), 
        //("usdc", "fum", "usdc"), 
    ];
    
    let arb_options = l19::schemas::ArbOptions::SelectedTwoLeg(arb_paths);
    let cruel_summer = l19::CruelSummer::new(arb_options).unwrap();
    cruel_summer.launch_searchers(true);
}
