use chrono::{DateTime, Utc};

pub enum MarketSide {
    Bid,
    Ask,
}

pub struct MarketOrder {
    client_id: usize,
    timestamp: DateTime<Utc>,
    size: usize,
    side: MarketSide,
}

impl MarketOrder {
    pub fn new(client_id: usize, timestamp: DateTime<Utc>, size: usize, side: MarketSide) -> Self {
        MarketOrder {
            client_id,
            timestamp,
            size,
            side,
        }
    }
}

pub struct LimitOrder {
    client_id: usize,
    timestamp: DateTime<Utc>,
    size: usize,
    side: MarketSide,
    price: usize,
}

impl LimitOrder {
    pub fn new(client_id: usize, timestamp: DateTime<Utc>, size: usize, side: MarketSide, price: usize) -> Self {
        LimitOrder {
            client_id,
            timestamp,
            size,
            side,
            price,
        }
    }
}

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
