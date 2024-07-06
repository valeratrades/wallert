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
	"params": ["alchemy_minedTransactions"]
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
