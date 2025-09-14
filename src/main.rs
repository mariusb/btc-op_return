use serde::{Deserialize};
use std::env;

#[derive(Debug, Deserialize)]
struct Transaction {
    txid: String,
    vout: Vec<TxOut>,
}

#[derive(Debug, Deserialize)]
struct TxOut {
    scriptpubkey_asm: String,
}

#[derive(Debug, Deserialize, Default)]
struct Stats {
    smallest_op_return: usize,
    largest_op_return: usize,
    smallest_op_return_hex: usize,
    largest_op_return_hex: usize,
    total_op_return: usize,
    txid: String,
    opreturn_ascii: String,
    opreturn_hex: String,
    total_transactions: usize
}

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    let client = reqwest::Client::new();

    let args: Vec<String> = env::args().collect();
    let target_height = if args.len() > 1 {
        args[1]
            .parse::<u32>()
            .expect("Please provide a valid block number.")
    } else {
        // 1. Get the latest block height if no argument is provided
        let tip_height_url = "https://mempool.space/api/blocks/tip/height";
        client
            .get(tip_height_url)
            .send()
            .await?
            .text()
            .await?
            .parse()
            .unwrap()
    };
    println!("Fetching transactions for block number: {}", target_height);

    // 2. Get the hash for the target block
    let block_hash_url = format!("https://mempool.space/api/block-height/{}", target_height);
    let block_hash = client.get(&block_hash_url).send().await?.text().await?;
    println!("Block hash: {}", block_hash);

    // 3. Get all transactions for that block by paginating
    let mut all_transactions = Vec::new();
    let mut start_index = 0;
    loop {
        let txs_url = format!(
            "https://mempool.space/api/block/{}/txs/{}",
            block_hash, start_index
        );
        let response_text = client.get(&txs_url).send().await?.text().await?;
        if response_text.trim().is_empty() || response_text == "Block not found" {
            break;
        }

        let transactions: Vec<Transaction> = match serde_json::from_str(&response_text) {
            Ok(txs) => txs,
            Err(_) => {
                // If parsing fails, we've likely hit the end or an error
                break;
            }
        };

        if transactions.is_empty() {
            break;
        }

        all_transactions.extend(transactions);
        start_index += 25;
    }

    let mut txnumber = 0;
    let mut stats = Stats::default();
    stats.total_transactions = all_transactions.len();
    // 4. Check each transaction for OP_RETURN
    for tx in all_transactions {
        txnumber += 1;
        let mut headerprinted = false;
        for output in tx.vout {
            if output.scriptpubkey_asm.starts_with("OP_RETURN") {
                // 5. If OP_RETURN is found, print the tx hash and the data
                let (hex_data, ascii_data) = extract_op_return_data(&output.scriptpubkey_asm);
                if !headerprinted {
                    println!("--------------------------------------------------");
                    println!("Transaction: #{} <--> {}", txnumber, tx.txid);
                    headerprinted = true;
                }
                println!("  OP_RETURN hex:   {}", hex_data);
                println!("  OP_RETURN ascii: {}", ascii_data);
                stats.total_op_return += 1;
                let data_length  = ascii_data.len(); // Each byte is represented by two hex characters
                if stats.smallest_op_return == 0 || data_length < stats.smallest_op_return {
                    stats.smallest_op_return = data_length;
                    stats.smallest_op_return_hex = hex_data.len();
                }
                if data_length > stats.largest_op_return {
                    stats.largest_op_return = data_length;
                    stats.largest_op_return_hex = hex_data.len();
                    stats.txid = tx.txid.clone();
                    stats.opreturn_ascii = ascii_data.clone();
                    stats.opreturn_hex = hex_data.clone();
                }
            }
        }
    }
    println!("--------------------------------------------------");
    println!("Statistics for block number:      {}", target_height);
    println!("  Total transactions processed:   {}", stats.total_transactions);
    println!("  Total OP_RETURN occurrences:    {}", stats.total_op_return);
    println!("  Smallest OP_RETURN hex length:  {} ({})", stats.smallest_op_return_hex, stats.smallest_op_return_hex/2);
    println!("  Largest OP_RETURN hex length:   {} ({})", stats.largest_op_return_hex, stats.largest_op_return_hex/2);
    println!("  Smallest OP_RETURN data length: {} characters", stats.smallest_op_return);
    println!("  Largest OP_RETURN data length:  {} characters", stats.largest_op_return);
    println!("    (Transaction ID:         {})", stats.txid);
    println!("    (OP_RETURN Data - ASCII: {})", stats.opreturn_ascii);
    println!("    (OP_RETURN Data - HEX:   {})", stats.opreturn_hex);
    Ok(())
}

fn extract_op_return_data(script_asm: &str) -> (String, String) {
    let parts: Vec<&str> = script_asm.split(' ').collect();
    if let Some(hex_data) = parts.last() {
        if let Ok(bytes) = hex::decode(hex_data) {
            let ascii_data = String::from_utf8_lossy(&bytes).to_string();
            return (hex_data.to_string(), ascii_data);
        } else {
            // It's possible the last part is not the data, but an opcode.
            // In that case, we can consider the second to last part.
            // This is getting complicated, so for now, we'll just indicate failure.
            if parts.len() > 1 {
                return (parts[parts.len()-1].to_string(), "Invalid hex data".to_string());
            }
        }
    }
    ("No data found".to_string(), "".to_string())
}