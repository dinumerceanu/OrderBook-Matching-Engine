# Orderbook Matching Engine in Rust

This is an educational project written in **Rust**, implementing a simple **orderbook** and **matching engine** similar to what is used in exchanges. The goal of the project is both to understand how an orderbook works (handling *limit* and *market* orders) and to deepen knowledge of Rust and its async ecosystem (`tokio`).

---

## 📌 Main Features
- Support for **Limit** and **Market** orders.
- **Order matching engine**: matches buy (*Bid*) and sell (*Ask*) orders based on price and quantity.
- Handles **multiple concurrent clients** over **TCP**.
- Real-time **notifications** for clients when orders are **filled**, **partially filled**, or **unfilled**.
- Live export of **traded prices** through a dedicated channel.
- **CLI Client** with input validation and instant feedback.

---

## 📂 Project Structure

```text
.
├── Cargo.toml
├── src
│   ├── lib.rs             # Exposes project modules
│   ├── orders.rs          # Order structures (LimitOrder, MarketOrder, etc.)
│   ├── orderbook.rs       # Orderbook implementation and matching logic
│   ├── client_handler.rs  # Handles client connections and communication channels
│   └── bin
│       ├── server.rs          # TCP server receiving orders and interacting with the orderbook
│       ├── client.rs          # Interactive CLI client to send commands and read responses
│       ├── test.rs            # Load-testing client spawner for benchmarking
│       └── orderbook_feeder.rs# Feeder for seeding the orderbook with random orders
```

---

## 🚀 Running the Project

### 1. Start the orderbook server
```bash
cargo run --bin server
```

The server will listen on **127.0.0.1:8080** and accept client connections.

### 2. Run the interactive client
```bash
cargo run --bin client
```

You can send commands like:
- **Market orders:**
  ```
  buy market 50
  sell market 30
  ```
- **Limit orders:**
  ```
  buy limit 120 10
  sell limit 90 5
  ```

### 3. Seed the orderbook with random orders (optional)
```bash
cargo run --bin orderbook_feeder
```

### 4. Run load testing with multiple clients
```bash
cargo run --bin test
```

This will launch 100 concurrent clients sending random orders to the server.

---

## 📊 Performance (benchmarks)

Performance measurements:
- With **50 existing orders** and **100 clients** connected → ~**100 TPS** (transactions per second).
- With **200 existing orders** and **750 clients** connected → ~**500 TPS**.

> Note: Performance can be improved by optimizing the internal data structures, reducing bottlenecks in `tokio::mpsc`, and possibly using more efficient matching algorithms.

---

## 🔧 Technologies Used
- **Rust** (safe, performant, systems-level)
- **Tokio** – async runtime for networking and concurrency
- **Chrono** – timestamps for orders
- **BTreeMap & VecDeque** – storage and fast access to orders
- **Rand** – generating random orders for testing

---

## 📖 What I Learned
- How a **real-world orderbook** works and the principles of a **matching engine**.
- Handling **concurrent clients** using channels and async in Rust.
- Differences between **market** and **limit** orders and their market impact.
- Potential optimizations for scalability (higher TPS).
- Organizing a modular Rust project with **multiple binaries** and a **shared crate library**.

---

## 🛠 Future Improvements
- Persistent storage of orders and trades.
- Support for **stop-loss** and **iceberg orders**.
- WebSocket API instead of raw TCP.
- Real-time web UI for orderbook visualization.
- More detailed benchmarking and profiling for optimization.

---

## 📜 License
This project is for **educational purposes only** and is **not intended for production trading systems**.

---

## 👤 Author
Developed to learn **Rust** and gain a deeper understanding of how a **matching engine** works.
