# Cryptocurrency HFTx Exchange Dash (Rust)
High-Frequency Trading Exchange Cryptocurrency Tx Service w/ Dash using Rust, Rust Tokio Asyc, Leptos WASM and Canvas.


## Project Structure

```shell
crytptocurrency-htfx-exchange-dash-rs/
├── Cargo.toml                          # Workspace manifest
├── README.md                           # Docs + Firecracker migration path
│
├── crates/
│   ├── dash-core/                      # Domain types (Trade, OrderBook, Candle, Ticker)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── trade.rs
│   │       ├── order.rs
│   │       ├── candle.rs
│   │       └── ticker.rs
│   │
│   ├── dash-state/                     # Leptos signals & reactive state
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       └── market.rs
│   │
│   ├── dash-charts/                    # D3-style SVG charts
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── candlestick.rs
│   │       ├── depth.rs
│   │       ├── sparkline.rs
│   │       └── chartkit.rs
│   │
│   ├── dash-websocket/                 # WebSocket client
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       └── client.rs
│   │
│   ├── dash-components/                
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── order.rs            
│   │       ├── trade_history.rs        
│   │       ├── ticker_bar.rs           
│   │       └── dashboard.rs            
│   │
│   └── dash-app/                       
│       ├── Cargo.toml
│       ├── Trunk.toml
│       ├── index.html
│       └── src/
│           └── main.rs
│
├── server/
│   └── dash-server/                    # Axum WebSocket server
│       ├── Cargo.toml
│       └── src/
│           ├── main.rs
│           ├── ws.rs
│           └── mock.rs
│
├── deploy/
│   ├── firecracker/                    
│   │   ├── vm-config.json
│   │   └── setup.sh
│   └── docker/                         
│       ├── Dockerfile.frontend
│       ├── Dockerfile.server-end
│       └── docker-compose.yml
│
└── static/
    └── css/
        └── theme.css                   
```
