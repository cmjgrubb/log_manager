use sqlx::mysql::MySqlPool;
use std::net::{SocketAddr, TcpListener, UdpSocket};
use std::str;
use std::sync::Arc;
use tokio::io::AsyncReadExt;
use tokio::net::TcpStream;
use tokio::task;

// #[derive(Clone)]
// pub struct Processor {
//     db_pool: Arc<MySqlPool>,
// }

// impl Processor {
//     pub async fn new(database_url: &str) -> Result<Self, Box<dyn std::error::Error>> {
//         let db_pool = MySqlPool::connect(database_url).await?;

//         Ok(Processor {
//             db_pool: Arc::new(db_pool),
//         })
//     }

//     pub async fn process_log(&self, log: &str) -> Result<(), Box<dyn std::error::Error>> {
//         sqlx::query("INSERT INTO syslogs (message) VALUES (?)")
//             .bind(log)
//             .execute(&*self.db_pool)
//             .await?;
//         Ok(())
//     }
// }

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
