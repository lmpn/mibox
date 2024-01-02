use mibox::axum_server::AxumServer;

#[tokio::main]
async fn main() -> anyhow::Result<()>{
    let server = AxumServer::default();
    server.serve().await
}
