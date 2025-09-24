use chrono::Utc;
use orderbook::{client_handler::Client, orders::*};
use tokio::{
    io::{self, AsyncBufReadExt, AsyncWriteExt, BufReader}, net::{TcpListener, TcpStream}, sync::mpsc
};
use std::{net::SocketAddr, sync::{atomic::{AtomicU64, Ordering::Relaxed}, Arc}, time::Instant};

pub fn create_order(input: &str, client: Client) -> Option<Orders> {
    let parts: Vec<&str> = input.trim().split_whitespace().collect();

    let side = match parts[0].to_lowercase().as_str() {
        "buy"  => Some(MarketSide::Bid),
        "sell" => Some(MarketSide::Ask),
        _ => None,
    };

    match parts[1].to_lowercase().as_str() {
        "market" => {
            let qty: usize = parts[2].parse().unwrap();
            Some(MarketOrder::new(Utc::now(), qty, 0, side.unwrap(), client, parts[3].to_string()).into())
        }

        "limit" => {
            let price: usize = parts[2].parse().unwrap();
            let qty: usize = parts[3].parse().unwrap();
            Some(LimitOrder::new(Utc::now(), qty, 0, side.unwrap(), price, client, parts[4].to_string()).into())
        },

        _ => None,
    }
}

async fn handle_client(stream: TcpStream, sockaddr: SocketAddr, tx_ob: mpsc::UnboundedSender<Orders>) -> io::Result<()> {
    let (reader, mut writer) = stream.into_split();

    let mut buf = BufReader::new(reader);

    let (tx, mut rx) = mpsc::channel::<String>(100);

    let client = Client::new(tx, sockaddr);

    let socket_reader = async move {
        loop {
            let mut line = String::new();
            match buf.read_line(&mut line).await {
                Ok(0) => {
                    println!("Connection terminated by client {sockaddr}");
                    break;
                },
                Ok(_) => {
                    let order = create_order(&line, client.clone()).unwrap();
                    if let Err(e) = tx_ob.send(order) {
                        eprintln!("Error sending order to OrderBook: {e}");
                    }
                    line.clear();
                },
                Err(e) => {
                    use std::io::ErrorKind;
                    if e.kind() == ErrorKind::ConnectionReset {
                        println!("Client {sockaddr} disconnected");
                    } else {
                        eprintln!("Error reading from client {sockaddr}: {e}");
                    }
                    break;
                }
            }
        }
    };

    let channel_reader_socket_writer = async move {
        loop {
            match rx.recv().await {
                Some(msg) => {
                    if let Err(e) = writer.write_all(format!("{msg}\n").as_bytes()).await {
                        eprintln!("Error writing to socket: {e}");
                        break;
                    } else {
                        println!("msg written to socket");
                    }
                    
                },
                None => {}
            }
        }
    };

    tokio::select! {
        _ = socket_reader => {},
        _ = channel_reader_socket_writer => {},
    }

    Ok(())
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;

    println!("Server listening on 127.0.0.1:8080");

    let (tx, mut rx) = mpsc::unbounded_channel::<Orders>();
    let (tx_price, mut rx_price) = mpsc::unbounded_channel::<usize>();

    let counter = Arc::new(AtomicU64::new(0));
    let c = counter.clone();

    let start = Instant::now();

    let client_handler_future = async move {
        loop {
            match listener.accept().await {
                Ok((stream, sockaddr)) => {
                    let tx_ob = tx.clone();
                    println!("New client connected from {sockaddr}");
                    tokio::spawn(handle_client(stream, sockaddr, tx_ob));
                },
                Err(e) => {
                    eprintln!("Error accepting connection: {e}");
                    break;
                }
            }
        }
    };

    let orderbook_handler_future = async move {
        let mut orderbook = orderbook::orderbook::OrderBook::new();
        loop {
            if let Some(order) = rx.recv().await {
                orderbook.handle_order(order, tx_price.clone(), c.clone());
                println!("{orderbook}");
            }
        }
    };

    let _price_showcase_future = async move {
        let mut stream = match TcpStream::connect("127.0.0.1:9000").await {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Unable to connect to price showcase server: {e}");
                return;
            }
        };

        loop {
            if let Some(price) = rx_price.recv().await {
                if let Err(e) = stream.write_all(format!("{}\n", price).as_bytes()).await {
                    eprintln!("Error sending price to server {}: {}", price, e);
                    break;
                }
            } else {
                break;
            }
        }
    };

    tokio::spawn(orderbook_handler_future);

    // tokio::spawn(price_showcase_future);
    //

    tokio::select! {
        _ = client_handler_future => {},
        _ = tokio::signal::ctrl_c() => {
            let total_trades = counter.load(Relaxed);
            let elapsed = start.elapsed().as_secs_f64();
            println!(
                "Total trades: {total_trades}, elapsed: {:.2}s, avg {:.2} trades/sec",
                elapsed,
                total_trades as f64 / elapsed
            );
            println!("\nCtrl-C received. Shutting down server gracefully...");
        }
    }

    Ok(())
}
