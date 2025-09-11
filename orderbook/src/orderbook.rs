use std::collections::{BTreeMap, VecDeque};
use crate::orders::{Orders};
use crate::client::Client;

pub struct OrderBook {
    Bids: BTreeMap<usize, VecDeque<Client>>,
    Asks: BTreeMap<usize, VecDeque<Client>>,
}

impl OrderBook {
    pub fn new() -> Self {
        OrderBook {
            Bids: BTreeMap::new(),
            Asks: BTreeMap::new(),
        }
    }

    pub fn handle_order(&mut self, order: Orders) {
        match order {
            Orders::Market(market_order) => {
                todo!()
            },
            Orders::Limit(limit_order) => {
                todo!()
            }
        }
    }
}
