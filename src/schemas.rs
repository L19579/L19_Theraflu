use crate::prelude::*;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct USDCConversion(i128);

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PriceInfo{
    pub quantity: u64,
    pub usdc_conversion: USDCConversion,
    pub symbol: String,
}

pub struct UserCredentials{ // Temp
    pub pubkey: Pubkey,
    pub keypair: Keypair,
    pub usdc_account_pubkey: Pubkey,
    pub keypair_path_str: String,
}
impl UserCredentials{
    pub fn new<T: fmt::Display>(pubkey: T, keypair_path_str: &str) -> Self {
        let keypair_path = Path::new(&keypair_path_str);
        let keypair: Keypair = keypair::read_keypair_file(keypair_path).unwrap(); 
        let pubkey = Pubkey::new(&bs58::decode(format!("{}", pubkey)).into_vec().unwrap()); 
        let usdc_account_pubkey = Pubkey::new(
            &bs58::decode(USDC_TOKEN_ACCOUNT).into_vec().unwrap());
        return Self{
            pubkey, keypair, usdc_account_pubkey, keypair_path_str: keypair_path_str.to_string()
        }; 
    }

    pub fn gen_copy(&self) -> Self{
        return Self::new(self.pubkey, &self.keypair_path_str);
    }
}

pub enum ArbOptions<'searcher>{
    All,
    SelectedTwoLeg(Vec<(&'searcher str, &'searcher str, &'searcher str)>),
}

// ------------------------------------------------- config
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Config{
    pub user: User,
    pub route_preferences: RoutePreferences,
    pub tokens: Vec<Token>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct User{
    pub pubkey: String,
    pub keypair_path: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RoutePreferences{
    pub random_route_with_settlement_at_ends: bool,
    pub maximum_jupiter_hops: u64,
    pub max_slippage: f64,
    pub max_usdc_per_trade: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Token{
    pub name: String,
    pub symbol: String,
    pub mint_key: String,
    pub token_account_key: Option<String>,
    pub unit_decimals: u64,
    pub settlement: bool,
}
impl Token{
    pub fn show_values(&self){
        println!("symbol: {}, mint_key: {}", self.symbol, self.mint_key);
    }
}

// ------------------------------------------------- Arbitrage
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ArbPath{
    pub path: Vec<(Token, Token)>,
}
// ------------------------------------------------- Jupiter
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct JupiterQuoteWrapper{
    pub base_mint: String,
    pub quote_mint: String,
    pub jupiter_quote: JupiterQuote,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct JupiterQuote{
    pub data: Vec<QuoteData>,
    pub timeTaken: f64,
}
impl JupiterQuote{
    pub fn show_data(&self){
        println!("Quote Data: ");
        for d in &self.data{
            println!("\n{:-<60}", "-");
            println!("\n\tinAmount: {}\n\toutAmount: {}\n\toutAmountWithSlippage: {}\n\tpriceImpactPct: {} marketInfos:",
            d.inAmount, d.outAmount, d.outAmountWithSlippage, d. priceImpactPct);
            for m in &d.marketInfos{
                let lp = &m.lpFee; 
                let pf = &m.platformFee; 
                println!("\t\tid: {}\n\t\tlabel: {}\n\t\tinputMint: {}\n\t\toutputMint: {}\n\t\tnotEnoughLiquidity: {}\n\t\tinAmount: {}",
                         m.id, m.label, m.inputMint, m.outputMint, m.notEnoughLiquidity, m.inAmount);
                println!("\t\toutAmount: {}\n\t\tpriceImpactPct: {}\n\t\tlpFee:",
                         m.outAmount, m.priceImpactPct);
                println!("\t\t\tamount: {}\n\t\t\tmint: {}\n\t\t\tpct: {}",
                         lp.amount, lp.mint, lp.pct);
                println!("\t\t\tamount: {}\n\t\t\tmint: {}\n\t\t\tpct: {}",
                         pf.amount, pf.mint, pf.pct);
            }
        }
    println!("\ttimeTaken: {}\n", self.timeTaken)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SerializedTransactions{ //Typo fix.
    pub transactions: Vec<String>,
    pub date_time: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct B64Transaction{
    pub setupTransaction: Option<String>,
    pub swapTransaction: String,
    pub cleanupTransaction: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SwapFormWrapper{
    pub swap_form: SwapForm,
    pub date_time: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SwapForm{
    pub route: QuoteData,
    pub wrapUnWrapSOL: bool,
    pub feeAccount: String,
    pub tokenLedger: String,
    pub userPublicKey: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PriceCheckData{
    pub id: String,
    pub mintSymbol: String,
    pub vsToken: String,
    pub vsTokenSymbol: String,
    pub price: f64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PriceCheck{
    pub data: PriceCheckData,
    pub timeTaken: f64,
}

impl PriceCheck{
    pub fn show_values(&self){
        let d = &self.data;
        println!("\tid: {}\n\tmintSymbol: {}\n\tvsToken: {}\n\tvsTokenSymbol: {}\n\tprice: {}\n\ttimeTaken: {}\n\n",
            d.id, d.mintSymbol, d.vsToken, d.vsTokenSymbol, d.price, self.timeTaken  
        ); 
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct QuoteDataSequence{
    pub quotes: Vec<QuoteData>,
    pub date_time: DateTime<Utc>,
}
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct QuoteData{
    pub inAmount: u64,
    pub outAmount: u64,
    pub amount: u64,
    pub outAmountWithSlippage: u64,
    pub otherAmountThreshold: u64,
    pub swapMode: String,
    pub priceImpactPct: f64,
    pub marketInfos: Vec<MarketInfos>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MarketInfos{
    pub id: String,
    pub label: String,
    pub inputMint: String,
    pub outputMint: String,
    pub notEnoughLiquidity: bool,
    pub inAmount: u64,
    pub outAmount: u64,
    pub priceImpactPct: f64,
    pub lpFee: LpFee,
    pub platformFee: PlatformFee,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LpFee{
    pub amount: f64,
    pub mint: String,
    pub pct: f64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PlatformFee{
    pub amount: f64,
    pub mint: String,
    pub pct: f64,
}
