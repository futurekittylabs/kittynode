use eyre::Result;

#[tokio::main]
async fn main() -> Result<()> {
    kittynode_web::run().await
}
