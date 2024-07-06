use anyhow::Result;
use futures_util::{SinkExt, StreamExt};
use serde_json::{json, Value};
use tokio::time::{Duration, Instant};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use serde::{Serialize, Deserialize};

async fn eth_websocket() -> Result<()> {
	let url = "wss://eth-mainnet.g.alchemy.com/v2/ta8waMJZgIdW8z6UoxDAYDfduqX0_lRb";
	let (ws_stream, _) = connect_async(url).await?;
	let (mut write, mut read) = ws_stream.split();

	println!("WebSocket connected");

	// Before first block is received, we get:
	//- WebSocket connected
	//- Subscription request sent
	//- {"id":1,"result":"0xbf525752be994fd9d0992b49ca8662cc","jsonrpc":"2.0"}


	// Subscribe to newHeads
	let subscribe_message = json!({
	"jsonrpc": "2.0",
	"id": 1,
	"method": "eth_subscribe",
	"params": ["newHeads"]
	});

	write.send(Message::Text(subscribe_message.to_string())).await?;
	println!("Subscription request sent");

	let mut last_ping_t = Instant::now();

	while let Some(msg) = read.next().await {
		let msg = msg?;
		if let Message::Ping(payload) = msg {
			last_ping_t = Instant::now();
			write.send(Message::Pong(payload)).await?;
			continue;
		}
		if last_ping_t + Duration::from_secs(10 * 60) < Instant::now() {
			eprintln!("No ping for 10m. Reconnecting...");
			return Ok(()); // This will cause the function to exit and can be restarted by the caller
		}

		if let Message::Text(text) = msg {
			if let Ok(parsed) = serde_json::from_str::<WebsocketMessage>(&text) {
				println!("{}", serde_json::to_string_pretty(&parsed).unwrap());
			} else {
				println!("Failed to parse.\n{}", text);
			}
		}
	}

	Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
	loop {
		match eth_websocket().await {
			Ok(_) => println!("WebSocket connection closed. Reconnecting..."),
			Err(e) => eprintln!("WebSocket error: {}. Reconnecting...", e),
		}
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
    result: EthereumBlock,
    subscription: String,
}

//TODO!!!!!: utilize everything below this line
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct EthereumBlock {
    base_fee_per_gas: String,
    blob_gas_used: String,
    difficulty: String,
    excess_blob_gas: String,
    extra_data: String,
    gas_limit: String,
    gas_used: String,
    hash: String,
    logs_bloom: String,
    miner: String,
    mix_hash: String,
    nonce: String,
    number: String,
    parent_beacon_block_root: String,
    parent_hash: String,
    receipts_root: String,
    sha3_uncles: String,
    size: String,
    state_root: String,
    timestamp: String,
    transactions_root: String,
    withdrawals: Vec<Withdrawal>,
    withdrawals_root: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Withdrawal {
    address: String,
    amount: String,
    index: String,
    validator_index: String,
}

// You can use this function to parse the JSON string into the EthereumBlock struct
pub fn parse_ethereum_block(json_str: &str) -> Result<EthereumBlock, serde_json::Error> {
    serde_json::from_str(json_str)
}
