extern crate tokio;
use tokio::{io::{self, AsyncWriteExt}, net::TcpStream};

#[tokio::main]
async fn main() -> io::Result<()> {
    let mut c1 = TcpStream::connect("127.0.0.1:8080").await?;
    let mut c2 = TcpStream::connect("127.0.0.1:8080").await?;

    let send1 = async {
        c1.write_all(b"sell market 8\n").await?;
        Ok::<_, std::io::Error>(())
    };
    let send2 = async {
        c2.write_all(b"sell market 6\n").await?;
        Ok::<_, std::io::Error>(())
    };

    tokio::try_join!(send1, send2)?;
    Ok(())
}
