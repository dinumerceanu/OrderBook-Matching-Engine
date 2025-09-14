use core::fmt;

use chrono::{DateTime, Utc};

use crate::client_handler::Client;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MarketSide {
    Bid,
    Ask,
}

#[derive(Debug)]
pub struct MarketOrder {
    timestamp: DateTime<Utc>,
    size: usize,
    fill_size: usize,
    side: MarketSide,
    client: Client,
}

impl MarketOrder {
    pub fn new(timestamp: DateTime<Utc>, size: usize, fill_size: usize, side: MarketSide, client: Client) -> Self {
        MarketOrder {
            timestamp,
            size,
            fill_size,
            side,
            client,
        }
    }

    pub fn timestamp(&self) -> &DateTime<Utc> {
        &self.timestamp
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn fill_size(&self) -> usize {
        self.fill_size
    }

    pub fn set_fill_size(&mut self, fill_size: usize) {
        self.fill_size = fill_size;
    }

    pub fn side(&self) -> MarketSide {
        self.side
    }

    pub fn client(&self) -> &Client {
        &self.client
    }
}

#[derive(Debug)]
pub struct LimitOrder {
    timestamp: DateTime<Utc>,
    size: usize,
    fill_size: usize,
    side: MarketSide,
    price: usize,
    client: Client,
}

impl LimitOrder {
    pub fn new(timestamp: DateTime<Utc>, size: usize, fill_size: usize, side: MarketSide, price: usize, client: Client) -> Self {
        LimitOrder {
            timestamp,
            size,
            fill_size,
            side,
            price,
            client,
        }
    }

    pub fn timestamp(&self) -> &DateTime<Utc> {
        &self.timestamp
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn fill_size(&self) -> usize {
        self.fill_size
    }

    pub fn set_fill_size(&mut self, fill_size: usize) {
        self.fill_size = fill_size;
    }

    pub fn side(&self) -> MarketSide {
        self.side
    }

    pub fn price(&self) -> usize {
        self.price
    }

    pub fn client(&self) -> &Client {
        &self.client
    }
}

impl fmt::Display for LimitOrder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "LimitOrder: size: {}, fill_size: {}", self.size, self.fill_size)
    }
}

#[derive(Debug)]
pub enum Orders {
    Market(MarketOrder),
    Limit(LimitOrder),
}

impl From<MarketOrder> for Orders {
    fn from(order: MarketOrder) -> Self {
        Orders::Market(order)
    }
}

impl From<LimitOrder> for Orders {
    fn from(order: LimitOrder) -> Self {
        Orders::Limit(order)
    }
}
