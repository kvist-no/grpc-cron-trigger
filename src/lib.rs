pub use crate::main::*;

pub mod main {
    use crate::sentry::cron_monitor::{CronJobStatus, CronMonitor};
    use crate::service::service_client::ServiceClient;
    use crate::service::CommandRequest;
    use anyhow::{anyhow, Result};
    use log::{error, info, warn};
    use std::env;
    use uuid::Uuid;

    pub async fn run() -> Result<()> {
        // Initialise logger
        env_logger::init();

        // Set up required environment variables
        let environment = get_env_var("ENVIRONMENT")?;
        let cron_url = get_env_var("CRON_URL")?;
        let notification_service_url = get_env_var("NOTIFICATION_SERVICE_URL")?;

        // Bootstrap out Sentry Cron monitor
        let check_in_id = Uuid::new_v4().hyphenated().to_string();

        let cron_monitor = CronMonitor {
            environment: &environment,
            cron_url: &cron_url,
            check_in_id: &check_in_id,
        };

        match cron_monitor.report(CronJobStatus::InProgress).await {
            Ok(_) => {
                info!("CronJob successfully reported as in progress")
            }
            Err(error) => {
                warn!(
                    "Failed to report status of CronJob as in progress: {}",
                    error
                );
            }
        }

        // Perform all work under here
        match trigger_command(notification_service_url).await {
            Ok(_) => {
                match cron_monitor.report(CronJobStatus::Ok).await {
                    Ok(_) => {
                        info!("CronJob successfully reported as succeeded")
                    }
                    Err(cron_monitor_error) => {
                        warn!(
                            "Failed to report status of CronJob as succeeded: {}",
                            cron_monitor_error
                        )
                    }
                }
                Ok(())
            }
            Err(error) => {
                match cron_monitor.report(CronJobStatus::Error).await {
                    Ok(_) => {
                        info!("CronJob successfully reported as failed")
                    }
                    Err(cron_monitor_error) => {
                        warn!(
                            "Failed to report status of CronJob as failed: {}",
                            cron_monitor_error
                        )
                    }
                }
                error!("CronJob failed: {}", error);
                Err(error)
            }
        }
    }

    async fn trigger_command(notification_service_url: String) -> Result<()> {
        let mut service_client = ServiceClient::connect(notification_service_url.clone()).await?;

        let from =
            get_env_var("COMMAND_FROM").unwrap_or(String::from("Kubernetes Cron Trigger"));

        let command = get_env_var("COMMAND_COMMAND")?;

        let data = get_env_var("COMMAND_DATA").unwrap_or(String::from("{}"));

        let requester = get_env_var("COMMAND_REQUESTER").unwrap_or(String::from(""));

        info!(
            "Sending gRPC command {} to {}",
            command,
            notification_service_url.clone()
        );
        let response = service_client
            .command(CommandRequest {
                from,
                command,
                data,
                requester,
                saga: None,
                correlation: None,
            })
            .await?;

        let backend_response = response.into_inner();

        info!("Backend succeeded and returned {}", backend_response.data);
        Ok(())
    }

    fn get_env_var(name: &str) -> Result<String> {
        match env::var(name) {
            Ok(value) => Ok(value),
            Err(_) => Err(anyhow!("Environment variable {} not set", name)),
        }
    }
}

pub mod sentry;

pub mod service {
    tonic::include_proto!("kvist.v1");
}
