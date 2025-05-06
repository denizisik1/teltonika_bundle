use tokio::{
    net::TcpListener,
    io::{AsyncReadExt, AsyncWriteExt},
    signal,
};
use tracing::{info, error};
use tracing_subscriber;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt::init();

    let listener = TcpListener::bind("127.0.0.1:12345").await?;
    info!("Server running on 0.0.0.0:12345");

    let shutdown_signal = async {
        signal::ctrl_c().await.expect("Failed to listen for shutdown signal");
        info!("Shutdown signal received");
    };

    tokio::select! {
        _ = accept_loop(listener) => {},
        _ = shutdown_signal => {
            info!("Shutting down server");
        },
    }

    Ok(())
}

async fn accept_loop(listener: TcpListener) {
    loop {
        match listener.accept().await {
            Ok((mut socket, addr)) => {
                info!("New connection from {}", addr);

                tokio::spawn(async move {
                    let mut buf = [0u8; 1024];

                    loop {
                        match socket.read(&mut buf).await {
                            Ok(0) => {
                                info!("Connection closed: {}", addr);
                                return;
                            }
                            Ok(n) => {
                                if let Err(e) = socket.write_all(&buf[..n]).await {
                                    error!("Write error to {}: {}", addr, e);
                                    return;
                                }
                            }
                            Err(e) => {
                                error!("Read error from {}: {}", addr, e);
                                return;
                            }
                        }
                    }
                });
            }
            Err(e) => {
                error!("Accept error: {}", e);
            }
        }
    }
}
