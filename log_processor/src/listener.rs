use sqlx::mysql::MySqlPool;
use std::net::{SocketAddr, TcpListener, UdpSocket};
use std::str;
use std::sync::Arc;
use tokio::io::AsyncReadExt;

#[derive(Clone)]
pub struct Processor {
    db_pool: Arc<MySqlPool>,
}

impl Processor {
    pub async fn new(database_url: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let db_pool = MySqlPool::connect(database_url).await?;

        Ok(Processor {
            db_pool: Arc::new(db_pool),
        })
    }

    pub async fn process_log(&self, log: &str) -> Result<(), Box<dyn std::error::Error>> {
        sqlx::query("INSERT INTO syslogs (message) VALUES (?)")
            .bind(log)
            .execute(&*self.db_pool)
            .await?;
        Ok(())
    }
}

pub async fn syslog() -> Result<(), Box<dyn std::error::Error>> {
    let addr: SocketAddr = "0.0.0.0:514".parse()?;
    let tcp_listener = TcpListener::bind(&addr)?;
    let udp_listener = UdpSocket::bind(&addr)?;

    let processor = match Processor::new("mysql://user:password@localhost/database").await {
        Ok(processor) => processor,
        Err(e) => {
            eprintln!("Failed to create processor: {e}");
            return Ok(());
        }
    };

    let tcp_task = tokio::spawn({
        let processor = processor.clone();
        async move {
            loop {
                let (socket, _) = tcp_listener.accept()?;
                let processor = processor.clone();
                tokio::spawn(async move {
                    handle_tcp_connection(socket, processor).await;
                });
            }
        }
    });

    let udp_task = tokio::spawn({
        let processor = processor.clone();
        async move {
            let mut buf = [0; 1024];
            loop {
                let (len, addr) = udp_listener.recv_from(&mut buf)?;
                let text = str::from_utf8_lossy(&buf[..len]);
                processor.process_log(&text).await.unwrap();
            }
        }
    });

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
