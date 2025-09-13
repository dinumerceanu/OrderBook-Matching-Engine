use core::fmt;
use std::{collections::{BTreeMap, VecDeque}};
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

    fn match_order(&mut self, mut market_order: MarketOrder) {
        match market_order.side() {
            MarketSide::Ask => {
                loop {
                    let available_market_order_size = market_order.size() - market_order.fill_size();

                    if available_market_order_size == 0 {
                        break;
                    }
                    
                    if let Some((price, orders_queue)) = self.get_best_bid() {
                        loop {
                            let available_market_order_size = market_order.size() - market_order.fill_size();
                            if available_market_order_size <= 0 {
                                break;
                            }
                            if let Some(mut limit_order) = orders_queue.pop_front() {
                                let available_limit_order_size = limit_order.size() - limit_order.fill_size();

                                if available_limit_order_size < available_market_order_size {
                                    println!("if1");
                                    let msg_full_fill = format!("Order filled!");
                                    let msg_partial_fill = format!("Order filled [{}/{}] at {}", available_limit_order_size, market_order.size(), limit_order.price());
                                    let limit_client_tx = limit_order.client().tx();
                                    let market_client_tx = market_order.client().tx();
                                    market_order.set_fill_size(market_order.fill_size() + available_limit_order_size);

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
                                } else if available_limit_order_size == available_market_order_size {
                                    println!("if2");
                                    let msg_market_client = format!("Order filled [{}/{}] at {}", available_market_order_size, market_order.size(), limit_order.price());
                                    let limit_client_tx = limit_order.client().tx();
                                    let market_client_tx = market_order.client().tx();
                                    market_order.set_fill_size(market_order.fill_size() + available_market_order_size);
                                    tokio::spawn(async move {
                                        let msg = format!("Order filled [{}/{}]", available_limit_order_size, limit_order.size());
                                        if let Err(e) = limit_client_tx.send(msg).await {
                                            eprintln!("Error writing to channel: {e}");
                                        }
                                    });
                                    tokio::spawn(async move {
                                        if let Err(e) = market_client_tx.send(msg_market_client).await {
                                            eprintln!("Error writing to channel: {e}");
                                        }
                                    });
                                    break;
                                } else {
                                    // limit size > market size
                                    println!("if3");
                                    let msg_full_fill = format!("Order filled [{}/{}] at {}", available_market_order_size, market_order.size(), limit_order.price());
                                    let msg_partial_fill = format!("Order filled [{}/{}]", available_market_order_size, limit_order.size());
                                    let limit_client_tx = limit_order.client().tx();
                                    let market_client_tx = market_order.client().tx();
                                    // UPDATE THE EXISTING LIMIT ORDER
                                    limit_order.set_fill_size(limit_order.fill_size() + available_market_order_size);
                                    market_order.set_fill_size(market_order.fill_size() + available_market_order_size);
                                    orders_queue.push_front(limit_order);
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
                        let unmatched_orders = market_order.size() - market_order.fill_size();
                        let msg = format!("Unfilled [{}/{}]", unmatched_orders, market_order.size());
                        tokio::spawn(async move {
                            if let Err(e) = market_order.client().tx().send(msg).await {
                                eprintln!("Error writing to channel: {e}");
                            }
                        });
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
