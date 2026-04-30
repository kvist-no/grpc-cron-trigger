use grpc_cron_trigger::service::{
    service_server::*, CommandRequest, CommandResponse, QueryRequest, QueryResponse,
};
use std::env;
use tokio::net::TcpListener;
use tonic::{transport::Server, Request, Response, Status};

#[derive(Debug, Default)]
pub struct MockService {
    pub received_requests: std::sync::Arc<std::sync::Mutex<Vec<CommandRequest>>>,
}

#[tonic::async_trait]
impl Service for MockService {
    async fn command(
        &self,
        request: Request<CommandRequest>,
    ) -> Result<Response<CommandResponse>, Status> {
        let req = request.into_inner();

        // Store the request for verification
        self.received_requests.lock().unwrap().push(req.clone());

        let response = CommandResponse {
            data: "Mock response".to_string(),
        };

        Ok(Response::new(response))
    }

    async fn query(
        &self,
        _request: Request<QueryRequest>,
    ) -> Result<Response<QueryResponse>, Status> {
        unimplemented!("Query not needed for this test")
    }
}

#[tokio::test]
async fn test_program_sends_grpc_message_on_startup() {
    // Set up mock gRPC server
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let service = MockService::default();
    let received_requests = service.received_requests.clone();

    // Start the mock server
    tokio::spawn(async move {
        Server::builder()
            .add_service(ServiceServer::new(service))
            .serve_with_incoming(tokio_stream::wrappers::TcpListenerStream::new(listener))
            .await
            .unwrap();
    });

    // Give server time to start
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Set environment variables for the test
    env::set_var("ENVIRONMENT", "test");
    env::set_var("CRON_URL", "http://test-sentry.com/cron");
    env::set_var(
        "NOTIFICATION_SERVICE_URL",
        format!("http://127.0.0.1:{}", addr.port()),
    );
    env::set_var("COMMAND_FROM", "Test From");
    env::set_var("COMMAND_COMMAND", "TestCommand");
    env::set_var("COMMAND_DATA", r#"{"test": "data"}"#);
    env::set_var("COMMAND_REQUESTER", "test-requester");

    // Run the main function (this will send the gRPC message)
    let result = grpc_cron_trigger::run().await;

    // Verify the result
    assert!(result.is_ok(), "Main function should complete successfully");

    // Verify that a gRPC request was received
    let requests = received_requests.lock().unwrap();
    assert_eq!(
        requests.len(),
        1,
        "Should have received exactly one gRPC request"
    );

    let request = &requests[0];
    assert_eq!(request.from, "Test From");
    assert_eq!(request.command, "TestCommand");
    assert_eq!(request.data, r#"{"test": "data"}"#);
    assert_eq!(request.requester, "test-requester");
}
