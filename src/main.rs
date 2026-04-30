use anyhow::Result;
use cron_trigger::run;

#[tokio::main]
async fn main() -> Result<()> {
    run().await
}
