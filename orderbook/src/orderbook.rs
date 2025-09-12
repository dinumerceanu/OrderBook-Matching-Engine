use core::fmt;
use std::{collections::{BTreeMap, VecDeque}};
use crate::orders::{LimitOrder, MarketOrder, MarketSide, Orders};
use crate::client_handler::Client;

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

    fn get_best_bid(&mut self) -> Option<(usize, &mut VecDeque<LimitOrder>)> {
        if let Some((&price, _)) = self.bids.last_key_value() {
            if let Some(queue) = self.bids.get_mut(&price) {
                Some((price, queue))
            } else {
                None
            }
        } else {
            None
        }
    }

    fn match_order(&mut self, market_order: MarketOrder) {
        match market_order.side() {
            MarketSide::Ask => {
                let mut market_order_size = market_order.size();
                while market_order_size > 0 {
                    if let Some((price, orders_queue)) = self.get_best_bid() {
                        loop {
                            if let Some(order) = orders_queue.pop_front() {
                                let limit_order_size = order.size();

                                if limit_order_size < market_order_size {
                                    let msg_full_fill = format!("Order filled!");
                                    let msg_partial_fill = format!("Partial fill [{}/{}]", limit_order_size, market_order_size);
                                    let limit_client_tx = order.client().tx();
                                    let market_client_tx = market_order.client().tx();
                                    market_order_size -= limit_order_size;
                                    tokio::spawn(async move {
                                        if let Err(e) = limit_client_tx.send(msg_full_fill).await {
                                            eprintln!("Error writing to channel: {e}");
                                        }
                                    });
                                    tokio::spawn(async move {
                                        if let Err(e) = market_client_tx.send(msg_partial_fill).await {
                                            eprintln!("Error writing to channel: {e}");
                                        }
                                    });
                                } else if limit_order_size == market_order_size {
                                    market_order_size = 0;
                                    let msg_full_fill = format!("Order filled!");
                                    let msg_full_fill_clone = msg_full_fill.clone();
                                    let limit_client_tx = order.client().tx();
                                    let market_client_tx = market_order.client().tx();
                                    tokio::spawn(async move {
                                        if let Err(e) = limit_client_tx.send(msg_full_fill).await {
                                            eprintln!("Error writing to channel: {e}");
                                        }
                                    });
                                    tokio::spawn(async move {
                                        if let Err(e) = market_client_tx.send(msg_full_fill_clone).await {
                                            eprintln!("Error writing to channel: {e}");
                                        }
                                    });
                                    break;
                                } else {
                                    // limit size > market size
                                    let msg_full_fill = format!("Order filled!");
                                    let msg_partial_fill = format!("Partial fill [{}/{}]", market_order_size, limit_order_size);
                                    let limit_client_tx = order.client().tx();
                                    let market_client_tx = market_order.client().tx();
                                    // UPDATE THE EXISTING LIMIT ORDER
                                    let old_ts = *order.timestamp();
                                    let old_side = *order.side();
                                    let client = Client::new(order.client().tx(), order.client().sockaddr());
                                    let new_limit_order = LimitOrder::new(old_ts, limit_order_size - market_order_size, old_side, order.price(), client);
                                    orders_queue.push_front(new_limit_order);
                                    market_order_size = 0;
                                    tokio::spawn(async move {
                                        if let Err(e) = limit_client_tx.send(msg_partial_fill).await {
                                            eprintln!("Error writing to channel: {e}");
                                        }
                                    });
                                    tokio::spawn(async move {
                                        if let Err(e) = market_client_tx.send(msg_full_fill).await {
                                            eprintln!("Error writing to channel: {e}");
                                        }
                                    });
                                    break;
                                }
                            } else {
                                println!("No bids at {price}");
                                self.bids.remove_entry(&price);
                                break;
                            }
                        }
                    } else {
                        println!("There are no bids!");
                        break;
                    }
                }
            },
            MarketSide::Bid => {
                // caut best ask
            },
        }
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
