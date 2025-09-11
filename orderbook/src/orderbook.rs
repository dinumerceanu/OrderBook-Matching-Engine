use core::fmt;
use std::collections::{BTreeMap, VecDeque};
use crate::orders::{LimitOrder, MarketOrder, MarketSide, Orders};

#[derive(Debug)]
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

impl fmt::Display for OrderBook {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "OrderBook:")?;
        writeln!(f, "Bids:")?;
        for (price, orders) in &self.bids {
            write!(f, "  {} -> ", price)?;
            for o in orders {
                write!(f, "{} ", o)?;
            }
            writeln!(f)?;
        }
        writeln!(f, "Asks:")?;
        for (price, orders) in &self.asks {
            write!(f, "  {} -> ", price)?;
            for o in orders {
                write!(f, "{} ", o)?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}
