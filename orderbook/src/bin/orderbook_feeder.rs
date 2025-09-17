use rand::{rng, Rng};
use tokio::{
    io::{self, AsyncWriteExt},
    net::TcpStream,
};

#[tokio::main]
pub async fn main() -> io::Result<()> {
    let mut tasks = Vec::new();
    let mut rng = rng();

    let total_orders = 50;

    for _ in 0..total_orders {
        let side = if rng.random_bool(0.5) { "buy" } else { "sell" };
        let price: usize = rng.random_range(80..=150);
        let qty: usize = rng.random_range(1..=10);

        let cmd = format!("{side} limit {price} {qty}\n");

        let order_task = tokio::spawn(async move {
            if let Ok(mut conn) = TcpStream::connect("127.0.0.1:8080").await {
                if let Err(e) = conn.write_all(cmd.as_bytes()).await {
                    eprintln!("Error sending {cmd}: {e}");
                }
            } else {
                eprintln!("Cannont connect to server for {cmd}");
            }
        });

        tasks.push(order_task);
    }

    for t in tasks {
        let _ = t.await;
    }

    Ok(())
}
