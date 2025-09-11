use tokio::{
    io::{self, AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader},
    net::TcpStream,
};
use std::io::Write;

pub fn validate_input(input: &str) -> Result<String, String> {
    let parts: Vec<&str> = input.trim().split_whitespace().collect();
    if parts.len() < 3 {
        return Err("Comanda prea scurtă".into());
    }

    let side = match parts[0].to_lowercase().as_str() {
        "buy"  => "buy",
        "sell" => "sell",
        _ => return Err("invalid side: choose 'buy' or 'sell'".into()),
    };

    match parts[1].to_lowercase().as_str() {
        "market" => {
            if parts.len() != 3 {
                return Err("Format market: buy/sell market <qty>".into());
            }
            let qty: usize = parts[2]
                .parse()
                .map_err(|_| "Invalid quantity".to_string())?;
            Ok(format!("{} market {}", side, qty))
        }

        "limit" => {
            if parts.len() != 4 {
                return Err("Format limit: buy/sell limit <price> <qty>".into());
            }
            let price: usize = parts[2]
                .parse()
                .map_err(|_| "Invalid price".to_string())?;
            let qty: usize = parts[3]
                .parse()
                .map_err(|_| "Invalid quantity".to_string())?;
            Ok(format!("{} limit {} {}", side, price, qty))
        }

        _ => Err("Invalid order type: choose 'market' or 'limit'".into()),
    }
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let stream = TcpStream::connect("127.0.0.1:8080").await?;

    println!("Connected to server!");

    let (mut stream_reader, mut stream_writer) = stream.into_split();

    let mut stdin_reader = BufReader::new(io::stdin());
    let mut input_line = String::new();

    let stdin_reader_future = async move {
        loop {
            print!("orderbook> ");
            std::io::stdout().flush().unwrap();
            input_line.clear();

            match stdin_reader.read_line(&mut input_line).await {
                Ok(0) => break,
                Ok(_) => {
                    if let Ok(valid) = validate_input(&input_line) {
                        if let Err(e) = stream_writer.write_all(format!("{}\n", valid).as_bytes()).await {
                            eprintln!("Error writing to server: {e}");
                            break;
                        }
                    } else {
                        eprintln!("Invalid command");
                    }
                }
                Err(e) => {
                    eprintln!("Error reading stdin: {e}");
                    break;
                }
            }
        }
    };

    let server_reader_future = async move {
        loop {
            let mut buf = vec![0; 1024];
            let res = stream_reader.read(&mut buf).await;

            match res {
                Ok(0) => {
                    print!("\x1B[2K\x1B[1G");
                    std::io::stdout().flush().unwrap();
                    println!("Connection terminated by the server");
                    break;
                },
                Ok(n) => {
                    let s = String::from_utf8_lossy(&buf[..n]);
                    let trimmed_s = s.trim();
                    print!("\x1B[2K\x1B[1G");
                    std::io::stdout().flush().unwrap();
                    println!("Received from server: {:?}", trimmed_s);
                    print!("orderbook> ");
                    std::io::stdout().flush().unwrap();
                },
                Err(e) => {
                    eprintln!("Error reading from server: {e}");
                    break;
                }
            } 
        }
    };

    tokio::select! {
        _ = stdin_reader_future => {},
        _ = server_reader_future => {},
        _ = tokio::signal::ctrl_c() => {
            println!("\nCtrl-C received, shutting down client...");
        }
    }

    std::process::exit(0);
}
