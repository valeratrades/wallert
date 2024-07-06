use anyhow::Result;
use futures_util::{SinkExt, StreamExt};
use serde_json::{json, Value};
use tokio::time::{Duration, Instant};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use serde::{Serialize, Deserialize};
use futures_util::future::join_all;

async fn websocket_connection(url: &str, network: &str) -> Result<()> {
    let (ws_stream, _) = connect_async(url).await?;
    let (mut write, mut read) = ws_stream.split();

    println!("{} WebSocket connected", network);

    // Subscribe to alchemy_minedTransactions
    let subscribe_message = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "eth_subscribe",
        "params": ["alchemy_minedTransactions"]
    });

    write.send(Message::Text(subscribe_message.to_string())).await?;
    println!("{} Subscription request sent", network);

    let mut last_ping_t = Instant::now();
    let mut n = 0;

    while let Some(msg) = read.next().await {
        let msg = msg?;
        if let Message::Ping(payload) = msg {
            last_ping_t = Instant::now();
            write.send(Message::Pong(payload)).await?;
            continue;
        }
        if last_ping_t + Duration::from_secs(10 * 60) < Instant::now() {
            eprintln!("{} No ping for 10m. Reconnecting...", network);
            return Ok(());
        }

        if let Message::Text(text) = msg {
            if let Ok(parsed) = serde_json::from_str::<WebsocketMessage>(&text) {
                println!("{} {}", network, serde_json::to_string_pretty(&parsed).unwrap());
                n += 1;
                println!("{} {}", network, n);
            } else {
                println!("{} Failed to parse.\n{}", network, text);
            }
        }
    }

    Ok(())
}

pub async fn run() -> Result<()> {
    let websockets = [
        ("wss://eth-mainnet.g.alchemy.com/v2/ta8waMJZgIdW8z6UoxDAYDfduqX0_lRb", "Ethereum"),
        ("wss://polygon-mainnet.g.alchemy.com/v2/ta8waMJZgIdW8z6UoxDAYDfduqX0_lRb", "Polygon"),
        ("wss://arb-mainnet.g.alchemy.com/v2/ta8waMJZgIdW8z6UoxDAYDfduqX0_lRb", "Arbitrum"),
        ("wss://opt-mainnet.g.alchemy.com/v2/ta8waMJZgIdW8z6UoxDAYDfduqX0_lRb", "Optimism"),
        ("wss://base-mainnet.g.alchemy.com/v2/ta8waMJZgIdW8z6UoxDAYDfduqX0_lRb", "Base"),
    ];

    loop {
        let tasks: Vec<_> = websockets.iter()
            .map(|(url, network)| websocket_connection(url, network))
            .collect();

        join_all(tasks).await;

        println!("All WebSocket connections closed. Reconnecting...");
        tokio::time::sleep(Duration::from_secs(5)).await;
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct WebsocketMessage {
    jsonrpc: String,
    method: String,
    params: WebSocketParams,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct WebSocketParams {
    result: MinedTransactions,
    subscription: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct MinedTransactions {
    removed: bool,
    transaction: Transaction,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Transaction {
    block_hash: String,
    block_number: String,
    from: String,
    gas: String,
    gas_price: String,
    hash: String,
    input: String,
    nonce: String,
    to: Option<String>,
    transaction_index: String,
    value: String,
    #[serde(rename = "type")]
    transaction_type: String,
    v: String,
    r: String,
    s: String,
    chain_id: String,
    max_fee_per_gas: Option<String>,
    max_priority_fee_per_gas: Option<String>,
    access_list: Option<Vec<Value>>,
    y_parity: Option<String>,
}
