use crate::processor::Processor;
use dotenv::dotenv;
use std::net::{SocketAddr, TcpListener, UdpSocket};
use tokio::io::AsyncReadExt;
use tokio::net::TcpStream;
use tokio::task;

pub async fn syslog() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    let database_url = std::env::var("DATABASE_URL")?;
    let addr: SocketAddr = "0.0.0.0:514".parse()?;
    let tcp_listener = TcpListener::bind(&addr)?;
    let udp_listener = UdpSocket::bind(&addr)?;

    let processor = match Processor::new(&database_url).await {
        Ok(processor) => processor,
        Err(e) => {
            eprintln!("Failed to create processor: {e}");
            return Ok(());
        }
    };

    let tcp_task = task::spawn({
        let processor = processor.clone();
        async move {
            loop {
                match tcp_listener.accept() {
                    Ok((socket, _)) => match TcpStream::from_std(socket) {
                        Ok(socket) => {
                            let processor = processor.clone();
                            task::spawn(async move {
                                handle_tcp_connection(socket, processor).await;
                            });
                        }
                        Err(e) => eprintln!("Failed to convert socket: {:?}", e),
                    },
                    Err(e) => eprintln!("Failed to accept connection: {:?}", e),
                }
            }
        }
    });

    let udp_task = task::spawn({
        let processor = processor.clone();
        async move {
            let mut buf = [0; 1024];
            loop {
                match udp_listener.recv_from(&mut buf) {
                    Ok((len, _)) => {
                        let text = String::from_utf8_lossy(&buf[..len]);
                        if let Err(e) = processor.process_log(&text).await {
                            eprintln!("Failed to process log: {:?}", e);
                        }
                    }
                    Err(e) => eprintln!("Failed to receive data: {:?}", e),
                }
            }
        }
    });

    tcp_task.await?;
    udp_task.await?;
    Ok(())
}

async fn handle_tcp_connection(mut socket: tokio::net::TcpStream, processor: Processor) {
    let mut buf = vec![0; 1024];
    loop {
        match socket.read(&mut buf).await {
            Ok(0) => break,
            Ok(len) => {
                let msg = String::from_utf8_lossy(&buf[..len]);
                if let Err(e) = processor.process_log(&msg).await {
                    eprintln!("Failed to process log message: {:?}", e);
                }
            }
            Err(e) => {
                eprintln!("Failed to read from socket; err = {:?}", e);
                break;
            }
        }
    }
}
