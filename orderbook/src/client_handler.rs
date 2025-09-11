use tokio::sync::mpsc;
use std::net::SocketAddr;

#[derive(Debug, Clone)]
pub struct Client {
    tx: mpsc::Sender<String>,
    sockaddr: SocketAddr,
}

impl Client {
    pub fn new(tx: mpsc::Sender<String>, sockaddr: SocketAddr) -> Self {
        Client {
            tx,
            sockaddr,
        }
    }

    pub fn tx(&self) -> mpsc::Sender<String> {
        self.tx.clone()
    }

    pub fn sockaddr(&self) -> SocketAddr {
        self.sockaddr
    }
}
