use anyhow::Result;
use grpc_cron_trigger::run;

#[tokio::main]
async fn main() -> Result<()> {
    run().await
}
