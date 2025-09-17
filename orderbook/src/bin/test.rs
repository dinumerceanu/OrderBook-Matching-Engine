use rand::Rng;
use tokio::{
    io::{self, AsyncWriteExt},
    net::TcpStream,
    time::{sleep, Duration},
};

async fn client_loop(id: usize) {
    loop {
        let (cmd, delay) = {
            let mut rng = rand::rng();

            let side = if rng.random_bool(0.5) { "buy" } else { "sell" };
            let order_type = if rng.random_bool(0.6) { "limit" } else { "market" };

            let command = if order_type == "limit" {
                let price: usize = rng.random_range(70..=160);
                let qty: usize = rng.random_range(1..=16);
                format!("{side} limit {price} {qty}\n")
            } else {
                let qty: usize = rng.random_range(1..=100);
                format!("{side} market {qty}\n")
            };

            let delay_ms = rng.random_range(100..1500);
            (command, delay_ms)
        };

        if let Ok(mut conn) = TcpStream::connect("127.0.0.1:8080").await {
            if let Err(e) = conn.write_all(cmd.as_bytes()).await {
                eprintln!("[client {id}] Eroare trimitere `{cmd}`: {e}");
            }
        } else {
            eprintln!("[client {id}] Nu pot conecta");
        }

        sleep(Duration::from_millis(delay)).await;
    }
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let num_clients = 100;
    let mut handles = Vec::new();

    for i in 0..num_clients {
        handles.push(tokio::spawn(client_loop(i)));
    }

    for h in handles {
        let _ = h.await;
    }

    Ok(())
}
