use std::collections::{BTreeMap, VecDeque};
use crate::orders::{LimitOrder, MarketOrder, MarketSide, Orders};

pub struct OrderBook {
    bids: BTreeMap<usize, VecDeque<LimitOrder>>,
    asks: BTreeMap<usize, VecDeque<LimitOrder>>,
}

impl OrderBook {
    pub fn new() -> Self {
        OrderBook {
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
        }
    }

    pub fn handle_order(&mut self, order: Orders) {
        match order {
            Orders::Market(market_order) => {
                self.match_order(market_order);
            },
            Orders::Limit(limit_order) => {
                self.add_order(limit_order);
            }
        }
    }

    fn add_order(&mut self, limit_order: LimitOrder) {
        match limit_order.side() {
            MarketSide::Ask => {
                self.asks
                    .entry(limit_order.price())
                    .or_insert_with(VecDeque::new)
                    .push_back(limit_order);
            },
            MarketSide::Bid => {
                self.bids
                    .entry(limit_order.price())
                    .or_insert_with(VecDeque::new)
                    .push_back(limit_order);
            },
        }
    }

    fn match_order(&mut self, market_order: MarketOrder) {
        todo!()
    }
}
