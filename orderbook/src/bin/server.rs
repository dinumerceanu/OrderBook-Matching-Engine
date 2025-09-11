use chrono::Utc;
use orderbook::{client_handler::Client, orders::*};
use tokio::{
    io::{self, AsyncBufReadExt, AsyncWriteExt, BufReader}, net::{TcpListener, TcpStream}, sync::mpsc
};
use std::{net::SocketAddr, time::Duration};

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
            Some(MarketOrder::new(Utc::now(), qty, side.unwrap(), client).into())
        }

        "limit" => {
            let price: usize = parts[2].parse().unwrap();
            let qty: usize = parts[3].parse().unwrap();
            Some(LimitOrder::new(Utc::now(), qty, side.unwrap(), price, client).into())
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
                    println!("Received order: {:?}", order);
                    match order {
                        Orders::Limit(limit_order) => {
                            if let Err(e) = tx_ob.send(Orders::Limit(limit_order)) {
                                eprintln!("Error sending order to OrderBook: {e}");
                            }
                        },
                        Orders::Market(market_order) => {
                            todo!()
                            //match orders
                        },
                    }
                    line.clear();
                },
                Err(e) => {
                    use std::io::ErrorKind;
                    if e.kind() == ErrorKind::ConnectionReset {
                        println!("Client {sockaddr} disconnected (connection reset).");
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
                orderbook.handle_order(order);
                println!("{orderbook}");
            }
        }
    };

    tokio::select! {
        _ = orderbook_handler_future => {},
        _ = client_handler_future => {},
        _ = tokio::signal::ctrl_c() => {
            println!("\nCtrl-C received. Shutting down server gracefully...");
        }
    }

    Ok(())
}
