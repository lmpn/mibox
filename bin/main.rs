use mibox::server::Server;

#[tokio::main]
async fn main() -> anyhow::Result<()>{
    let server = Server::default();
    server.serve().await
}
