mod websockets;
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
	websockets::run().await?;
	Ok(())
}
