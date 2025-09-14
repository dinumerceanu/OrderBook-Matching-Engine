use core::fmt;
use std::{collections::{BTreeMap, VecDeque}};
use tokio::sync::mpsc;

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

    pub fn handle_order(&mut self, order: Orders, tx_price: mpsc::UnboundedSender<usize>) {
        match order {
            Orders::Market(market_order) => {
                self.match_order(market_order, tx_price);
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

    fn notify(tx: mpsc::Sender<String>, msg: String) {
        tokio::spawn(async move {
            if let Err(e) = tx.send(msg).await {
                eprintln!("Error writing to channel: {e}");
            }
        });
    }

    fn send_price(tx: mpsc::UnboundedSender<usize>, price: usize) {
        tokio::spawn(async move {
            if let Err(e) = tx.send(price) {
                eprintln!("Error writing price on channel: {e}");
            }
        });
    }

    fn get_best_bid(&mut self) -> Option<usize> {
        if let Some((&price, _)) = self.bids.last_key_value() {
            Some(price)
        } else {
            None
        }
    }

    fn get_best_ask(&mut self) -> Option<usize> {
        if let Some((&price, _)) = self.asks.first_key_value() {
            Some(price)
        } else {
            None
        }
    }

    fn match_order(&mut self, mut market_order: MarketOrder, tx_price: mpsc::UnboundedSender<usize>) {
        loop {
            let available_market_order_size = market_order.size() - market_order.fill_size();

            if available_market_order_size == 0 {
                break;
            }

            let best_price = if market_order.side() == MarketSide::Ask {
                self.get_best_bid()
            } else {
                self.get_best_ask()
            };

            if let Some(price) = best_price {
                let orders_queue = if market_order.side() == MarketSide::Ask {
                    self.bids.get_mut(&price).unwrap()
                } else {
                    self.asks.get_mut(&price).unwrap()
                };
                loop {
                    let available_market_order_size = market_order.size() - market_order.fill_size();
                    if available_market_order_size <= 0 {
                        break;
                    }
                    if let Some(mut limit_order) = orders_queue.pop_front() {
                        let available_limit_order_size = limit_order.size() - limit_order.fill_size();

                        if available_limit_order_size < available_market_order_size {
                            let msg_full_fill = format!("Order filled [{}/{}]", available_limit_order_size, limit_order.size());
                            let msg_partial_fill = format!("Order filled [{}/{}] at {}", available_limit_order_size, market_order.size(), limit_order.price());
                            
                            let limit_client_tx = limit_order.client().tx();
                            let market_client_tx = market_order.client().tx();

                            market_order.set_fill_size(market_order.fill_size() + available_limit_order_size);

                            Self::notify(limit_client_tx, msg_full_fill);
                            Self::notify(market_client_tx, msg_partial_fill);
                            Self::send_price(tx_price.clone(), limit_order.price());

                        } else if available_limit_order_size == available_market_order_size {
                            let msg_market_client = format!("Order filled [{}/{}] at {}", available_market_order_size, market_order.size(), limit_order.price());
                            let msg_limit_client = format!("Order filled [{}/{}]", available_limit_order_size, limit_order.size());
                            let limit_client_tx = limit_order.client().tx();
                            let market_client_tx = market_order.client().tx();
                            let price = limit_order.price();

                            market_order.set_fill_size(market_order.fill_size() + available_market_order_size);
                            
                            Self::notify(limit_client_tx, msg_limit_client);
                            Self::notify(market_client_tx, msg_market_client);
                            Self::send_price(tx_price.clone(), price);

                            break;
                        } else {
                            let msg_full_fill = format!("Order filled [{}/{}] at {}", available_market_order_size, market_order.size(), limit_order.price());
                            let msg_partial_fill = format!("Order filled [{}/{}]", available_market_order_size, limit_order.size());

                            let limit_client_tx = limit_order.client().tx();
                            let market_client_tx = market_order.client().tx();

                            let price= limit_order.price();

                            limit_order.set_fill_size(limit_order.fill_size() + available_market_order_size);
                            market_order.set_fill_size(market_order.fill_size() + available_market_order_size);

                            orders_queue.push_front(limit_order);

                            Self::notify(limit_client_tx, msg_partial_fill);
                            Self::notify(market_client_tx, msg_full_fill);
                            Self::send_price(tx_price.clone(), price);

                            break;
                        }
                    } else {
                        if market_order.side() == MarketSide::Ask {
                            println!("No bids at {price}");
                        } else {
                            println!("No asks at {price}");
                        }
                        break;
                    }
                }
                let empty = orders_queue.is_empty();
                if empty {
                    match market_order.side() {
                        MarketSide::Ask => {
                            self.bids.remove(&price);
                        },
                        MarketSide::Bid => {
                            self.asks.remove(&price);
                        }
                    }
                }
            } else {
                if market_order.side() == MarketSide::Ask {
                    println!("There are no bids!");
                } else {
                    println!("There are no asks!"); 
                }
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
