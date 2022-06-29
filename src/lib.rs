pub mod prelude;
pub mod schemas;
use { prelude::*, schemas::* };

pub struct CruelSummer{
    pub config: Config,
    pub user_credentials: UserCredentials,
    pub arb_paths: Vec<ArbPath>,
}

impl CruelSummer{
    pub fn new(arb_options: ArbOptions) -> Result<Self>{
        let config = Self::load_config().unwrap();
        let arb_paths = Self::load_arb_paths(&config.tokens, &arb_options).unwrap();
        let user_credentials = UserCredentials::new(&config.user.pubkey,
            &config.user.keypair_path);
        return Ok ( Self{ config, user_credentials, arb_paths} );
    }
    
    pub fn launch_searchers(&self, recover_crashed_thread: bool){
        let mut thread_counter = 1u8;
        let mut thread_handles = vec![];  // `Token` below needs to be a reference. 
        let active_tokens: Vec<(Token, bool)> = (self.config).tokens.clone().into_iter().map(|token| (token, false)).collect::<Vec<(Token, bool)>>();
        // ^ above is useless atm.
        let lock_other_ops = Arc::new(Mutex::new(false));
        let active_tokens = Arc::new(Mutex::new(active_tokens));
        let last_active_tx_quote: Arc<Mutex<Option<(Token, u64)>>> = Arc::new(Mutex::new(None));
        let failed_recovery_loop_count = Arc::new(Mutex::new(0u16));

        for arb_path in &self.arb_paths{
            thread::sleep(Duration::from_millis(rand::thread_rng().gen_range(700..3000))); // Staggered launches to keep within rate limits.
            let mut thread_name = format!("{}/", arb_path.path[0].0.symbol);
            thread_name.push_str(&arb_path.path.clone().into_iter().map(|p| p.1.symbol.clone()).collect::<Vec<String>>().join("/"));
            {
                let thread_name = thread_name.clone();
                let config = self.config.clone();
                let arb = arb_path.clone();
                let user_cred = self.user_credentials.gen_copy();
                let active_tokens = Arc::clone(&active_tokens);
                let lock_others = Arc::clone(&lock_other_ops); // altered name for reachability. Refactor.
                let last_active_tx_quote = Arc::clone(&last_active_tx_quote); // (last quote in a failed arb, initial in amount)
                let failed_recovery_loop_count = Arc::clone(&failed_recovery_loop_count);
                thread_handles.push(
                    thread::Builder::new().name(thread_name.clone()).spawn(move ||{
                        loop{
                            let config = config.clone();
                            let arb = arb.clone();
                            let user_cred = user_cred.gen_copy();
                            let active_tokens = Arc::clone(&active_tokens);
                            let lock_other_ops = Arc::clone(&lock_others);
                            let last_active_tx_quote = Arc::clone(&last_active_tx_quote);
                            let failed_recovery_loop_count = Arc::clone(&failed_recovery_loop_count);
                            _ = panic::catch_unwind(move ||{
                            Self::kanye_west(thread_counter, config, arb, user_cred, active_tokens, lock_other_ops, last_active_tx_quote,
                                failed_recovery_loop_count);
                            });
                            if recover_crashed_thread {
                                let stmt = format!("Thread `{} - {}` crashed. Recovering in 10s", thread_counter, thread_name);
                                show_statement(StatementType::Warning, &stmt);
                                thread::sleep(Duration::from_secs(rand::thread_rng().gen_range(8..12)));
                                *lock_others.lock().unwrap() = false;
                                continue;
                            }
                            break;
                        }
            }))};
            thread_counter += 1; 
        }
        for thread_handle in thread_handles{
            thread_handle.unwrap().join().unwrap();
        }
    }
    
    pub fn kanye_west(searcher_id: u8, config: Config, arb_path: ArbPath, 
        user_credentials: UserCredentials, active_tokens: Arc<Mutex<Vec<(Token, bool)>>>,
        lock_other_ops: Arc<Mutex<bool>>, last_active_tx_quote: Arc<Mutex<Option<(Token, u64)>>>,
        failed_recovery_loop_count: Arc<Mutex<u16>>){
        show_statement(StatementType::ThreadOnline, &format!("{:<15} Searcher at work.", 
            thread::current().name().unwrap_or("Unamed thread")));
        let rpc_client = RpcClient::new(RPC_CLIENT_LINK);
        let in_token = arb_path.path[0].0.clone();
        let out_token = arb_path.path.last().unwrap().1.clone();
        let max_usdc_per_trade = config.route_preferences.max_usdc_per_trade;
        'main_loop: loop{
            if *lock_other_ops.lock().unwrap() { continue };
            let maximum_in_amount = match Self::approx_token_price(None, &arb_path.path[0].0, 
                max_usdc_per_trade){
                Ok(p) => p,
                Err(_) => {
                    thread::sleep(Duration::from_secs(4));
                    continue; 
                },
            };
            thread::sleep(Duration::from_secs(rand::thread_rng().gen_range(2..6)));
            let mut quotes = Vec::<JupiterQuote>::new();
            let mut best_quote_data = Vec::<QuoteData>::new();
            let date_time = Utc::now();
            let token_account_balance = match Self::token_account_balance(&rpc_client, &in_token){
                Ok(b)  => b,
                _ => {
                    continue 'main_loop
                },
            };
            
            if *failed_recovery_loop_count.lock().unwrap() > 4 {
                show_statement(StatementType::General, "Helping another bot recover. Checking and transfering from all active token accounts."); 
                *lock_other_ops.lock().unwrap() = true;
                for token in &*active_tokens.lock().unwrap(){
                    println!("\tchecking {} token account", token.0.symbol);
                    if token.0.mint_key.to_lowercase() == USDC_MINT_KEY.to_lowercase(){
                        continue;
                    }
                    let token_acc_balance = match Self::token_account_balance(&rpc_client, &token.0){
                        Ok(b) => b,
                        _ => continue,
                    };  
                    let token_usdc_price = match Self::approx_usdc_price(Some(&token.0), token_acc_balance){ 
                        Ok(p) => p,
                        _ => continue,
                    };  
                    if (token_usdc_price as f64/max_usdc_per_trade as f64) > 0.93f64{
                        let _ = Self::send_from_token_to_usdc_account(&config, &user_credentials, 
                            &token.0, Some(token_acc_balance));
                    }  
                } 
                *failed_recovery_loop_count.lock().unwrap() = 0;
                *lock_other_ops.lock().unwrap() = false;
                thread::sleep(Duration::from_secs(10));  
                continue 'main_loop;
            } 
            
            if token_account_balance < maximum_in_amount{ // Blocks non USDC starts.
                if last_active_tx_quote.lock().unwrap().is_none(){
                    show_statement(StatementType::General, "last_active_tx_quote is empty. Recovered coins likely unsettled. Pausing for 15s."); 
                    thread::sleep(Duration::from_secs(15));  
                    *failed_recovery_loop_count.lock().unwrap() += 1;
                    continue;
                }
                let result: Result<()>;
                match &*last_active_tx_quote.lock().unwrap()
                {
                    Some(q) => result = {
                        *lock_other_ops.lock().unwrap() = true;
                        let r = Self::send_from_token_to_usdc_account(&config, &user_credentials, &q.0, Some(q.1));
                        *lock_other_ops.lock().unwrap() = false;
                        r
                    }, // < - This loop continues forever. Impl hard kill after x recovery attempts.
                    None => result = Err(anyhow!("")),
                };
                match result {
                    Ok(()) => { *last_active_tx_quote.lock().unwrap() = None; }
                    _ => (),
                };

                let stmt = format!("{} : {} token account below required amount ({}). Rechecking.", 
                    thread::current().name().unwrap_or("Unamed thread"), in_token.symbol, maximum_in_amount);
                show_statement(StatementType::Warning, &stmt);
                continue;
            };

            if last_active_tx_quote.lock().unwrap().is_some() {
                let last_tx_token_acc_balance = match Self::token_account_balance(&rpc_client, &in_token){
                    Ok(b)  => b,
                    _ => {
                        continue 'main_loop
                    },
                };
                let q = match last_active_tx_quote.lock().unwrap().as_ref(){
                    Some(q) => q.clone(),
                    None => continue 'main_loop, // unlocked @ if checker. This applies to everything else tho. Review.
                };

                if last_active_tx_quote.lock().unwrap().as_ref().unwrap().1 >= last_tx_token_acc_balance{
                    *lock_other_ops.lock().unwrap() = true;
                    //let r = Self::recover_last_tx(&user_credentials, q.0.clone(), q.1);
                    let result = Self::send_from_token_to_usdc_account(&config, &user_credentials, &q.0, Some(q.1));
                    *lock_other_ops.lock().unwrap() = false;
                    
                    match result {
                        Ok(()) => { *last_active_tx_quote.lock().unwrap() = None; }
                        _ => (),
                    };
                }
            }

            quotes.push( match Self::get_quote(&in_token.mint_key, &arb_path.path[0].1.mint_key, maximum_in_amount){
                Ok(q) => q,
                _ => continue 'main_loop,
            }); 
            for i in 1..arb_path.path.len(){
                let path = &arb_path.path[i];
                best_quote_data.push(Self::best_quote(&quotes[i-1]));
                quotes.push(match Self::get_quote(&path.0.mint_key, &path.1.mint_key, best_quote_data.last().unwrap().outAmountWithSlippage){
                    Ok(q) => q,
                    _ => continue 'main_loop,
                }); 
            }
            best_quote_data.push(Self::best_quote(&quotes.last().unwrap()));
            let final_out_amount = best_quote_data.last().unwrap().outAmountWithSlippage;
            let arb_in_tokens: Vec<&Token> = arb_path.path.iter().map(|p| &p.0).collect();
            let in_amounts: Vec<u64> = best_quote_data.iter().map(|q| q.inAmount).collect();
            let approx_fees_in_usdc = match Self::load_max_fees_as_usdc(&(arb_in_tokens.into_iter().zip(in_amounts.into_iter())
                .collect::<Vec<(&Token, u64)>>())){
                Ok(f) => f,
                Err(_) => continue 'main_loop, 
            }; 
            let pnl = (match Self::approx_usdc_price(Some(&out_token), final_out_amount){
                Ok(p) => p as i64,
                Err(_) => continue 'main_loop, 
            }) - (max_usdc_per_trade as i64 + approx_fees_in_usdc as i64);
            let statement = format!("{} ; in_amount: {} {}, final_out_amount: {} {}; fee: {} USDC ; pnl {} USDC", date_time, 
                maximum_in_amount, in_token.symbol, final_out_amount, out_token.symbol, approx_fees_in_usdc, pnl);
            show_statement(StatementType::General, &format!("{} --- {}", thread::current().name().unwrap_or("Unamed thread"), statement));
            if pnl > 10000 { 
                if *lock_other_ops.lock().unwrap(){ continue };
                *lock_other_ops.lock().unwrap() = true;

                let date_time = Utc::now();
                let mut path_str_desc = String::new();
                path_str_desc.push_str(&format!("{}", arb_path.path[0].0.symbol));
                for p_s in 0..arb_path.path.len(){
                    let path = &arb_path.path[p_s].1.symbol;
                    path_str_desc.push_str(&format!("-> {}", path));
                }
                let statement = format!("{} ; in_amount: {} {}, final_out_amount: {} {}; fee: {} USDC ; pnl {} USDC", date_time, 
                    maximum_in_amount, in_token.symbol, final_out_amount, out_token.symbol, approx_fees_in_usdc, pnl);
                show_statement(StatementType::Opportunity, &format!("{} --- {}", thread::current().name().unwrap_or("Unamed thread"), statement));
                //show_statement(StatementType::Opportunity, &statement);
                let mut fetch_times_str: String = String::from("Fetch times: ");
                fetch_times_str.push_str(&quotes.clone().into_iter().enumerate().map(|(i, q)| format!("\n\tquote #{}: {} seconds", i, q.timeTaken))
                    .collect::<Vec<String>>().join(" ")); 
                println!("{}", fetch_times_str);
                
                for (i, quote) in quotes.into_iter().enumerate(){
                    println!("\tquote {}: {} ms", i+1, quote.timeTaken);
                }                                                                              
                *last_active_tx_quote.lock().unwrap() = Some((arb_path.path[1].0.clone(), best_quote_data.last().unwrap().inAmount));
                let quote_data_sequence = QuoteDataSequence { quotes: best_quote_data, arb_path: arb_path.clone(), date_time };
                _ = Self::capture_v3(&user_credentials, quote_data_sequence, &in_token);
                thread::sleep(Duration::from_secs(10));
                *lock_other_ops.lock().unwrap() = false;
                continue;
            } 
            show_statement(StatementType::General, &format!("{:<18} ; {} --- No opportunity found", 
                thread::current().name().unwrap_or("Unamed thread"), date_time));
        } 
    } 
    
    pub fn token_is_locked(active_tokens: &Arc<Mutex<Vec<(Token, bool)>>>, token: &Token) -> bool{
        let mut account_inactive = true;
        for active_token in &*active_tokens.lock().unwrap(){
            if active_token.0.mint_key == token.mint_key{
                account_inactive = active_token.1;
                break;
        }} 
        return account_inactive;
    }

    pub fn token_account_balance(rpc_client: &RpcClient, token: &Token) -> Result<u64>{
        let  ui_amount = match rpc_client.get_token_account_balance_with_commitment(
            &b58_to_pubkey(&token.token_account_key.as_ref().unwrap()), CommitmentConfig::confirmed()){
            Ok(p) => p,
            Err(e) => return Err(anyhow!("{}", e)),
        };
        return Ok((ui_amount.value.ui_amount.unwrap() * 10f64.powi(ui_amount.value.decimals as i32))as u64);
    }

    pub fn non_dec_amount(token: &Token, amount: f64) -> u64{
        (amount * token.unit_decimals as f64) as u64
    }

    pub fn load_max_fees_as_usdc(t_and_a: &[(&Token, u64)]) -> Result<u64>{
        // 000_000_100 + 000_000_500 lamports
        let expected_tx_fee_per_leg = t_and_a.len() as u64 * match Self::approx_usdc_price(None, 100 + 500){
            Ok(f) => f,
            Err(e) => return Err(anyhow!("{}", e)),
        };  

        match panic::catch_unwind (|| {
            let jup_fees: u64 = t_and_a.iter().map(|(token, in_amount)| Self::approx_usdc_price(Some(&token), 
            (0.0004 * *in_amount as f64) as u64).unwrap()).sum(); 
            expected_tx_fee_per_leg + jup_fees
        }){
            Ok(f) => Ok(f),
            Err(_) => {
                let stmt = format!("{:<18} : Unable to load fees", thread::current().name().unwrap_or("Unamed thread"));
                show_statement(StatementType::Warning, &stmt);
                return Err(anyhow!(""));
            },
        } 
    }
   
    /// Input token defaults to `SOL` 
    pub fn approx_usdc_price(token: Option<&Token>, token_amount: u64) -> Result<u64>{
        if token.is_some() && token.unwrap().symbol.to_lowercase() == "usdc"{
            return Ok(token_amount);
        };

        match panic::catch_unwind(||{
            let formatted_price: u64 = Runtime::new().unwrap().block_on(async {
                let https_client = Client::builder().build::<_,hyper::Body>(HttpsConnector::new());
                let url: *const str = &*format!("https://price.jup.ag/v1/price?id={}", 
                   token.map_or("SOL", |t| &t.symbol));
                let response = unsafe{
                    https_client.get(Uri::from_static(&*url)).await.unwrap()
                };
                let buf = hyper::body::to_bytes(response).await.unwrap();
                let price_check = serde_json::from_slice::<PriceCheck>(&buf).unwrap(); 
                let one_token_usdc_p = price_check.data.price * 10f64.powi(6);
                (one_token_usdc_p * (token_amount as f64 / 10f64.powi(token
                    .map_or(9, |t| t.unit_decimals as i32)))) as u64 
            });
            formatted_price
        }){
            Ok(formatted_price) => Ok(formatted_price),
            Err(_) => {
                let error_str = format!("Could not retrieve price data from Jupiter. (Error code intentionally hidden)");
                show_statement(StatementType::Warning, &error_str);
                Err(anyhow!("{}", error_str))
            },
        }
    }
    // Combine this and above fns.
    /// Input token defaults to `USDC`
    pub fn approx_token_price(input_t: Option<&Token>, output_t: &Token, input_t_amount: u64) -> Result<u64>{
        if input_t.is_some() && input_t.unwrap().mint_key == output_t.mint_key
        || input_t.is_none() && output_t.symbol.to_lowercase() == "usdc"{
            return Ok(input_t_amount);
        }; 

        match panic::catch_unwind(||{
            let formatted_price = Runtime::new().unwrap().block_on(async {
                let https_client = Client::builder().build::<_,hyper::Body>(HttpsConnector::new());
                let url: *const str = &*format!("https://price.jup.ag/v1/price?id={}&vsToken={}",
                    input_t.map_or("USDC", |t| &t.symbol), output_t.symbol);
                let response = unsafe{
                    https_client.get(Uri::from_static(&*url)).await.unwrap()
                };
                let buf = hyper::body::to_bytes(response).await.unwrap();
                let price_check = serde_json::from_slice::<PriceCheck>(&buf).unwrap(); 
                (price_check.data.price * input_t_amount as f64 
                 * 10f64.powi(output_t.unit_decimals as i32 - input_t.map_or(6, |t| t.unit_decimals) as i32)) as u64  
            });
            formatted_price 
        }){
            Ok(formatted_price) => Ok(formatted_price),
            Err(_) => {
                let error_str = format!("Could not retrieve price data from Jupiter");
                show_statement(StatementType::Warning, &error_str);
                Err(anyhow!("{}", error_str))
            },
        }
    }
    
    pub fn best_quote(quotes: &JupiterQuote) -> QuoteData{
        let mut highest_out_amount = 0;
        let mut best_quote = QuoteData{
            inAmount: 0,
            outAmount: 0,
            amount: 0,
            outAmountWithSlippage: 0,
            otherAmountThreshold: 0,
            swapMode: String::new(),
            priceImpactPct: 0f64,
            marketInfos: Vec::<MarketInfos>::new(),
        };
        for q in &quotes.data{
            if q.outAmountWithSlippage> highest_out_amount{
               highest_out_amount = q.outAmountWithSlippage;
               best_quote = q.clone();
            }
        }
        return best_quote;    
    } 

    pub fn get_quote(input_mint: &str, output_mint: &str, amount: u64) -> Result<JupiterQuote> {
        let quote_url = Self::generate_quote_link(input_mint, output_mint, amount);
        
        let data: String = match Runtime::new().unwrap().block_on(
            Self::pull_via_https(&*quote_url)){
            Ok(d)  => d,
            Err(e) => return Err(anyhow!("{}", e)),
        };
        //println!("TRACE ---- {}", data);
        let quote: JupiterQuote = match serde_json::from_str(&data){
            Ok(q) => q,
            Err(e) => return Err(anyhow!("{}", e)),
        }; 
        
        return Ok(quote);
    }

    pub async fn pull_via_https(target_url: *const str) -> Result<String>{
        let https = HttpsConnector::new();
        let https_client = Client::builder().build::<_, hyper::Body>(https);
        let mut data = String::new();
        unsafe {
            for _ in 0..4{
                let response = match https_client.get(Uri::from_static(&*target_url))
                .await{
                    Ok(r) => r,
                    Err(e) => return Err(anyhow!("{}", e)),
                };
                //println!("status: \n{}\n\n", response.status());
                let buf = match hyper::body::to_bytes(response).await{
                    Ok(b) => b,
                    Err(e) => return Err(anyhow!("{}", e)),
                };
                data = String::from_utf8_lossy(&buf).into_owned();
                if data.len() > 40 { break }; 
                thread::sleep(Duration::from_secs(2));
            }
        }
        return Ok(data);
    }
    
    pub fn capture_v3(user_creds: &UserCredentials, route: QuoteDataSequence, in_token: &Token) -> Result<()>{
        let in_token_balance = match Self::token_account_balance(&RpcClient::new(RPC_SERUM_LINK), &in_token){
            Ok(b)  => b,
            Err(e) => return Err(anyhow!("{}", e)),
        };
        if in_token_balance < route.quotes[0].inAmount{ 
                show_statement(StatementType::LowBalance, "attempt cancelled");
                return Err(anyhow!(""));
        }
        let serialized_legs = Self::serialized_transactions(route.clone(), user_creds).unwrap();//{
        
        for (a, serialized_transactions) in  serialized_legs.transactions.iter().enumerate(){
    println!("TRACE --- serialized transactions: {}", serialized_transactions);
            let txs_b64: B64Transaction = match serde_json::from_str(&serialized_transactions){
                Ok(t) => t,
                _ => {
                    show_statement(StatementType::Error, "Couldn't build Tx object from B64. Skipping Arb");
                    return Err(anyhow!("Failed to parse transactions"));
            }};
            let txs: [Option<Transaction>; 3] = [
                match txs_b64.setupTransaction {
                    Some(t) => {
                        if a == 0 && route.quotes.len() > 1{ 
                            show_statement(StatementType::ExcessRisk, "Starting leg includes a setup Tx. Attempt cancelled");
                            return Err(anyhow!(""));
                        };
                        Some(Self::base64_to_transaction(t).unwrap())
                    },
                    None => None,
                },
                Some(Self::base64_to_transaction(txs_b64.swapTransaction).unwrap()),
                match txs_b64.cleanupTransaction {
                    Some(t) => Some(Self::base64_to_transaction(t).unwrap()),
                    None => None,
                },
            ];  
            let (std_wait, swap_wait) = (12, 14);
            let commitment_config = CommitmentConfig::confirmed();
            let tpu_config = TpuClientConfig{ fanout_slots: DEFAULT_FANOUT_SLOTS };
            let transaction_config = rpc_config::RpcSendTransactionConfig{
                skip_preflight: true, preflight_commitment: None, encoding: None,
                max_retries: None, min_context_slot: None,
            };
            for (b, tx) in txs.into_iter().enumerate(){
                if tx.is_none() { continue };
                let mut sigs: Vec<Signature> = Vec::new();
                let mut status: Vec<bool> = Vec::new();

                for _ in 0..1 {
        /*  tpu 1/3: Uncomment for access to TPU clients. WSS links reqd. This is likely to be ineffective
            when validators implement QUIC.
                    let tpu_clients = [
                        match TpuClient::new(
                        Arc::new(RpcClient::new(RPC_SERUM_LINK)),
                        RPC_QUICKNODE_WSS_LINK,
                        tpu_config.clone(),
                        ){
                            Ok(t) => t,
                            Err(_) => return Err(anyhow!("Failed to create TPU client")),
                        }, 
                        match TpuClient::new(
                        Arc::new(RpcClient::new(RPC_SSO_LINK)),
                        RPC_QUICKNODE_WSS_LINK,
                        tpu_config.clone(),
                        ){
                            Ok(t) => t,
                            Err(_) => return Err(anyhow!("Failed to create TPU client")),
                        }, 
                    ];
            */
                    // User should replace duplicate RPCs here. Will cause simple errors if deleted.
                    let rpc_clients: [RpcClient; 4] = [
                        RpcClient::new_with_commitment(RPC_SSO_LINK, commitment_config),
                        RpcClient::new_with_commitment(RPC_SERUM_LINK, commitment_config),
                        RpcClient::new_with_commitment(RPC_SERUM_LINK, commitment_config),
                        RpcClient::new_with_commitment(RPC_SSO_LINK, commitment_config),
                    ]; 
                    let blockhash = match Self::get_valid_blockhash(&rpc_clients[0]){
                        Ok(h) => h,
                        Err(e) => {
                            let error_str = format!("Unable to fetch valid block hash, skipping opportunity. Error: {}", e);
                            show_statement(StatementType::Error, &error_str);
                            return Err(anyhow!("{}", e));
                        },
                    };
                    let mut tx = tx.clone().unwrap();
                    tx.message.recent_blockhash = blockhash;
                    tx.sign(&[&user_creds.keypair], blockhash); 
                    
                   // tpu 2/3: _ = tpu_clients.iter().map(|tpu| tpu.try_send_transaction(&tx)); // Send 2 txs to leader. 
                    sigs.append(&mut rpc_clients.iter().map(|rpc| rpc.send_transaction_with_config(&tx, transaction_config).unwrap()).collect::<Vec<Signature>>()); // Send 4 txs to various RPCs.
                    // tpu 3/3: let sigs: Vec<Signature> = alt_rpcs.iter().map(|rpc| rpc.send_and_confirm_transaction_with_spinner_and_config(&tx, commitment_config, transaction_config).unwrap()).collect(); 
                    let sigs_str = sigs.iter().map(|s| format!("\n\t{}", s)).collect::<Vec<String>>().join("");
                    println!("{} \n", sigs_str);
                    let wait_time = if b == 1 { swap_wait } else { std_wait };
                    thread::sleep(Duration::from_secs(wait_time));
                    status.append(&mut sigs.iter().map(|s| rpc_clients[0].confirm_transaction(&s).unwrap() ).collect::<Vec<bool>>());
                }
                let final_balance = match Self::token_account_balance(&RpcClient::new(RPC_SSO_LINK), &route.arb_path.path[a].1){
                    Ok(b) => b,
                    Err(e) => return Err(anyhow!("{}", e)),
                }; 
                if final_balance < route.quotes[0].outAmountWithSlippage || &status[..] == [false; 8]{ // Flaw here. Account could be emptied by another bot. ------------------------------------------------------ FIX!
                    let stmt = format!("{:<18} : Couldn't confirm tx, dropping attempt.", thread::current().name().unwrap_or("Unamed thread"));
                    show_statement(StatementType::Error, &stmt);
                    return Err(anyhow!(""));
                }
                show_statement(StatementType::Success, &format!("Transaction confirmed. Leg {}/{}, Tx {}/1-3", 
                    a+1, serialized_legs.transactions.len(), b+1));
        }}
        return Ok(());
    }
    
    /// Sends full token account balance if `amount = None`
    pub fn send_from_token_to_usdc_account(config: &Config, user_creds: &UserCredentials, in_token: &Token, amount: Option<u64>)
    -> Result<()>{
        let stmt = format!("{:<18}: Attempting to recover last tx. Likely at a loss",
            thread::current().name().unwrap_or("Unanmed thread"));
        show_statement(StatementType::General, &stmt);
        let rpc_client = RpcClient::new(RPC_SERUM_LINK);
        let date_time = Utc::now();
        let mut usdc_token: Option<Token> = None;
        for token in &config.tokens{
            if token.mint_key == USDC_MINT_KEY{ usdc_token = Some(token.clone()); }
        }
        if usdc_token.is_none(){
            return Err(anyhow!("Couldn't derive usdc token")); 
        }
        let starting_account_balance = match Self::token_account_balance(&rpc_client, in_token){
            Ok(b)  => b,
            Err(e) => return Err(anyhow!("{}", e)),
        };
        if starting_account_balance == 0 {
            show_statement(StatementType::Warning, "Attempt to recover from an account w/ a zero balance");  
            return Ok(()); 
        }
        if amount.is_some() && starting_account_balance < amount.unwrap() {
            show_statement(StatementType::Warning, "Attempt to recover from an account w/ a balance lower than requested amount");  
            return Ok(()); 
        }
        let in_amount = amount.unwrap_or_else(|| starting_account_balance); 
        let quote = match Self::get_quote(&in_token.mint_key, USDC_MINT_KEY, in_amount) {
            Ok(q) => q,
            _ => return Err(anyhow!("")),
        };
        let best_quote_data = Self::best_quote(&quote);
        //let out_amount = best_quote_data.outAmountWithSlippage;
        let arb_path = ArbPath{ path: vec![(in_token.clone(), usdc_token.unwrap())] };
        let quote_data_sequence = QuoteDataSequence{ quotes: vec![best_quote_data], arb_path: arb_path.clone(), date_time};
        match Self::capture_v3(user_creds, quote_data_sequence, in_token){
            Ok(_)  => (),
            Err(e) => return Err(anyhow!("{}", e)),
        };

        //balance recheck.
        thread::sleep(Duration::from_secs(15));
        let updated_balance = match Self::token_account_balance(&rpc_client, in_token){
            Ok(b)  => b,
            Err(e) => return Err(anyhow!("{}", e)),
        };
        if starting_account_balance - updated_balance >= in_amount{
        println!("TRACE NEW RECOVERY -----------------------------------------------------  END RECOVERED");
            return Ok(()); // Again not 100% reliable. Another bot may mod account.
        }
        println!("TRACE NEW RECOVERY -----------------------------------------------------  END RECOVERY FAILED");
        return Err(anyhow!(""));
    }

    ///Deprecated. Use `send_from_token_to_usdc_account()`
    pub fn recover_last_tx(user_creds: &UserCredentials, quote: QuoteData, original_in_amount: u64)
    -> Result<()>{
        let stmt = format!("{:<18}: Attempting to recover last tx. Likely at a loss",
            thread::current().name().unwrap_or("Unanmed thread"));
        show_statement(StatementType::General, &stmt);
        let rpc_client = RpcClient::new(RPC_CLIENT_LINK); 
        let mut out_decr_pct = 0.02f32;
        let mut updtd_out_w_slip_amnt = quote.outAmountWithSlippage;
        let mut updated_quote = quote.clone();
        
        let commitment_config = CommitmentConfig::confirmed();
        let transaction_config = rpc_config::RpcSendTransactionConfig{
            skip_preflight: true, preflight_commitment: None, encoding: None,
            max_retries: Some(2), min_context_slot: None,
        };
       
        let mut i = 0;
        'top_loop: loop {
            i += 1;
            println!("Attempt #{}", i);
            if (updated_quote.outAmountWithSlippage as f64 / original_in_amount as f64) < 0.93f64 {
            let stmt = format!("{:>18}: Below acceptable out amount loss. Stopping auto recovery attempt.",
                thread::current().name().unwrap_or("Unanmed thread"));
                show_statement(StatementType::Error, &stmt);
            }
            out_decr_pct += 0.02f32;
            updtd_out_w_slip_amnt = (updtd_out_w_slip_amnt as f32 * (1f32 - out_decr_pct)) as u64;
            updated_quote.outAmountWithSlippage = updtd_out_w_slip_amnt;
            let serialized_transactions = Runtime::new().unwrap().block_on(
                Self::pull_serialized_transaction_https(updated_quote.clone(), &user_creds)); 
            let serialized_transactions = match serialized_transactions {
                Ok(s_t) => s_t,
                Err(e) => return Err(anyhow!("{}", e)),
            };
            let txs_b64: B64Transaction = match serde_json::from_str(&serialized_transactions){
                Ok(f) => f,
                Err(e) => { 
                    println!("{}", e);
                    return Err(anyhow!("{}", e))
                },
            };

            let txs_b64_arr = [
                txs_b64.setupTransaction,
                Some(txs_b64.swapTransaction),
                txs_b64.cleanupTransaction,
            ];
            for (i, tx_b64) in txs_b64_arr.iter().enumerate(){
                match tx_b64{
                    Some(tx) => {
                        
                        let mut tx = match Self::base64_to_transaction(tx.clone()){
                            Ok(tx) => tx,
                            Err(e) => return Err(anyhow!("{}", e)),
                        };
                        let blockhash = match Self::get_valid_blockhash(&rpc_client){
                            Ok(b) => b,
                            _ => continue 'top_loop
                        };
                        tx.message.recent_blockhash = blockhash;
                        tx.sign(&[&user_creds.keypair], blockhash);
                        match panic::catch_unwind(||{
                            let rpc_client = RpcClient::new(RPC_CLIENT_LINK); // <------ Find a way to use Arc/Mutex
                            for i in 1..{
                                if i > 3 { panic!("") }
                                match rpc_client.send_and_confirm_transaction_with_spinner_and_config(
                                    &tx, commitment_config, transaction_config){
                                    Ok(tx_sig)  => {
                                        show_statement(StatementType::Success, &format!("Recovery transaction confirmed. Signature: {}", tx_sig));
                                        break;
                                    }, 
                                    Err(_) => {
                                        show_statement(StatementType::Success, &format!("Couldn't push recovery transaction, reattempting."));
                                        continue; 
                            }}}
                        }){
                            Ok(_) => {
                                if (txs_b64_arr[0].is_none() && txs_b64_arr[2].is_none() && i > 0){
                                    return Ok(());
                                };
                                if (txs_b64_arr[0].is_none() || txs_b64_arr[2].is_none()) && i > 1{
                                    return Ok(());     
                            }},
                            _ => continue 'top_loop,
                    }},
                    None => continue,
        }}}
    }

    pub fn get_valid_blockhash(rpc_client: &RpcClient) -> Result<Hash>{
        let commitment_config = CommitmentConfig::confirmed();

        for _ in 0..15 {
            //let (blockhash, _ ) = rpc_client.get_latest_blockhash_with_commitment(commitment_config).unwrap(); 
            let (blockhash, _ ) = match rpc_client.get_latest_blockhash_with_commitment(commitment_config){
                Ok(h) => h,
                Err(e) => return Err(anyhow!("{}", e)),
            }; 
            if rpc_client.is_blockhash_valid(&blockhash, commitment_config).unwrap(){
                println!("Valid blockhash found: {}", blockhash);
                return Ok(blockhash);
            }
        }
        return Err(anyhow!("Unable to get valid blockhash"));
    }
	
	pub fn base64_to_transaction(base64_transaction: String) -> Result<Transaction> {
        bincode::deserialize(&base64::decode(base64_transaction)?).map_err(|err| err.into())
    }
    // Below needs to be more async. FIX!
    pub fn serialized_transactions(route: QuoteDataSequence, user_credentials: &UserCredentials) -> Result<SerializedTransactions>{ 
        let mut transactions = Vec::<String>::new();
        let mut date_time: Option<DateTime<Utc>> = None;
        for quote in route.quotes{
            if date_time.is_none(){
                date_time = Some(Utc::now()); 
            }
            let serialized_transaction = match Runtime::new().unwrap().block_on(
            Self::pull_serialized_transaction_https(quote.clone(), &user_credentials)){
                Ok(t) => t,
                Err(e) => return Err(anyhow!("{}", e)),
            };
            
            transactions.push(serialized_transaction) 
        }
        return Ok(SerializedTransactions{ transactions, date_time: date_time.unwrap() });
    }
    
    async fn pull_serialized_transaction_https(route: QuoteData, user_cred: &UserCredentials) -> Result<String>{
        let swap_form = SwapForm{
            route,
            wrapUnWrapSOL: true,
            feeAccount: String::from(""),
            tokenLedger: String::from(""),
            userPublicKey: format!("{}", user_cred.pubkey), 
        }; 
        let serialized_form = match serde_json::ser::to_string(&swap_form){
            Ok(f) => f,
            Err(e) => return Err(anyhow!("{}", e)),
        };
        let https = HttpsConnector::new();
        let https_client = Client::builder().build::<_, hyper::Body>(https);
        let mut req = Request::builder().method(Method::POST).uri(INSTRUCTION_LINK).body(Body::from(serialized_form))
            .expect("request builder");
        // Post request will fail without header.
        req.headers_mut().insert(header::CONTENT_TYPE, header::HeaderValue::from_static("application/json"));
        let response = match https_client.request(req).await{
            Ok(r) => r,
            Err(e) => return Err(anyhow!("{}", e)),
        };
        //println!("status: \n{}\n\n", response.status());
        let buf = match hyper::body::to_bytes(response).await{
            Ok(b) => b,
            Err(e) => return Err(anyhow!("{}", e)),
        };
        let buf_str = String::from_utf8_lossy(&buf).into_owned();
        return Ok(buf_str);
    }

    pub fn generate_quote_link(input_mint: &str, output_mint: &str, amount: u64) -> String {
        let link = format!("https://quote-api.jup.ag/v1/quote?inputMint={}&outputMint={}&amount={}&slippage=0.07&feeBps=4", 
                       input_mint, output_mint, amount);
        return link;
    }
    
    #[inline(always)]
    pub fn load_config() -> Result<Config>{
        let mut config_file = OpenOptions::new().read(true).open(CONFIG_FILE_PATH).unwrap();
        let mut buf_reader = BufReader::new(config_file);
        let mut content_buf = Vec::with_capacity(MAXIMUM_FILE_LEN);
        buf_reader.read_to_end(&mut content_buf);
        let config: Config = toml::de::from_str(&String::from_utf8(content_buf).unwrap()).unwrap();
        return Ok(config);
    }
   
    //Stopped dev early here. There are better ways to do this. Highly recc modifying.
    #[inline(always)]
    pub fn load_arb_paths(tokens: &[Token], arb_options: &ArbOptions) -> Result<Vec<ArbPath>>{ 
        let mut arb_paths = Vec::<ArbPath>::new();
        
        for token in tokens{
            println!("{}", token.symbol);
        }

        for left_token in tokens{
            for center_token in tokens{
                for right_token in tokens{
                    //if l == c || c == r { continue };
                    if left_token.mint_key == center_token.mint_key ||
                        center_token.mint_key == right_token.mint_key { continue }
                    match arb_options {
                        ArbOptions::All => {
                        println!("---------------LOADING ARB_ALL");
                            arb_paths.push(ArbPath{
                                    path: vec![(left_token.clone(), center_token.clone()), (center_token.clone(), right_token.clone())],
                            }); 
                            let ok_statement = format!("{} -> {} -> {}", left_token.symbol, center_token.symbol, 
                                right_token.symbol);
                            show_statement(StatementType::General, &ok_statement);
                                //println!("TRACE --- Not comparable");
                        },
                        ArbOptions::SelectedTwoLeg(arb_selection) => {
                        //println!("---------------LOADING ARB_SELECT");
                            for (l_s, c_s, r_s) in arb_selection{
                                //println!("comparing ({}, {}, {}) to ({}, {}, {})",
                                //left_token.symbol, center_token.symbol, right_token.symbol,
                                //l_s, c_s, r_s);
                                   // || left_token.symbol.to_lowercase() == l_s.to_lowercase()
                                   // || left_token.mint_key == *l_s
                                    
                                   //|| center_token.symbol.to_lowercase() == c_s.to_lowercase()
                                    //|| center_token.mint_key == *c_s

                                    // || right_token.symbol.to_lowercase() == r_s.to_lowercase()
                                    //|| right_token.mint_key == *r_s{
                                
                                if left_token.symbol.to_lowercase() == l_s.to_lowercase()
                                    && center_token.symbol.to_lowercase() == c_s.to_lowercase()
                                    && right_token.symbol.to_lowercase() == r_s.to_lowercase()
                                    {    arb_paths.push(
                                            ArbPath{path: vec![(left_token.clone(), center_token.clone()), (center_token.clone(), right_token.clone())],
                                        }); 
                        //println!("TRACE --- comparable");
                                        let ok_statement = format!("{} -> {} -> {}", left_token.symbol, center_token.symbol, 
                                           right_token.symbol);
                                        show_statement(StatementType::General, &ok_statement);
                                };
                            }
                        
                        },
                    };
        }}}
        return Ok(arb_paths);
    }
} 

pub fn b58_to_pubkey(key: &str) -> Pubkey{
    let pubkey = Pubkey::new(&bs58::decode(key).into_vec().unwrap());
    return pubkey;
}

pub fn show_tx_str_details(tx: &Transaction){
    let mh = tx.message.header;
    println!("num_req_sigs: {} num_read_only_signed_acc: {} num_read_only_unsigned_acc: {}",
    mh.num_required_signatures, mh.num_readonly_signed_accounts, mh.num_readonly_unsigned_accounts);
    let m_keys = &tx.message.account_keys;
    println!("Keys in message: ");
    for k in m_keys{
        println!("{}", k);
    }
    println!("Blockhash: {}", tx.message.recent_blockhash);
    println!("Instructions: ");
    for ins in &tx.message.instructions{
        println!("\tprogram_id_index: {}", ins.program_id_index);
        for aac in &ins.accounts{
            println!("\t{}", aac);
        }
        println!("data: {:?}", &ins.data[..]);
    }
}

pub enum StatementType{
    Opportunity,
    ThreadOnline,
    Success,
    General,
    Warning,
    LowWarning,
    LowBalance,
    ExcessRisk,
    Error,
}

//fix these.
pub fn show_statement(statement_type: StatementType, statement: &str){
    let prompt: colored::ColoredString = match statement_type {
        StatementType::Opportunity => "Opportunity Found: ".green().bold(),  
        StatementType::ThreadOnline => "Thread Online: ".green().bold(),  
        StatementType::Success => "Success: ".green().bold(),  
        StatementType::General => "General: ".cyan().bold(),
        StatementType::Warning => "Warning: ".yellow().bold(),  
        StatementType::LowWarning => "Warning: ".yellow(),  
        StatementType::LowBalance => "Low Balance: ".yellow(),  
        StatementType::ExcessRisk => "Excessive Risk: ".yellow(),  
        StatementType::Error => "Error: ".red().bold(),
    };
    println!("{}: {}", prompt, statement);
}
