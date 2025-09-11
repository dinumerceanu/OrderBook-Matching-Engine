use tokio::net::TcpStream;
use std::net::SocketAddr;

pub struct Client {
    stream: TcpStream,
    sockaddr: SocketAddr
}

impl Client {
    pub fn new(stream: TcpStream, sockaddr: SocketAddr) -> Self {
        Client {
            stream,
            sockaddr,
        }
    }
}
