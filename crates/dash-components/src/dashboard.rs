//! Main dashboard layout component

use dash_charts::{CandlestickChart, DepthChart};
use dash_state::{use_app_state, Panel};
use leptos::prelude::*;

use crate::{OrderBook, TickerBar, TradeHistory};

#[component]
pub fn Dashboard() -> impl IntoView {
    let state = use_app_state();

    view! {
        <div class="dashboard">
            <header class="dash-header">
                <TickerBar
                    market=state.market.clone()
                    connection=Signal::from(state.connection)
                />
            </header>

            <main class="dash-main">
                <aside class="dash-sidebar left">
                    <div class="panel">
                        <div class="panel-header">
                            <span class="panel-title">"Order Book"</span>
                        </div>
                        <div class="panel-content">
                            <OrderBook market=state.market.clone() />
                        </div>
                    </div>
                </aside>

                <section class="dash-center">
                    <div class="panel chart-container">
                        <div class="panel-header">
                            <span class="panel-title">"Chart"</span>
                        </div>
                        <div class="panel-content">
                            <CandlestickChart candles=state.market.candles.into() />
                        </div>
                    </div>

                    <div class="panel depth-container">
                        <div class="panel-header">
                            <span class="panel-title">"Market Depth"</span>
                        </div>
                        <div class="panel-content">
                            <DepthChart depth=state.market.depth.into() />
                        </div>
                    </div>
                </section>

                <aside class="dash-sidebar right">
                    <div class="panel">
                        <div class="panel-header">
                            <span class="panel-title">"Recent Trades"</span>
                        </div>
                        <div class="panel-content">
                            <TradeHistory market=state.market.clone() />
                        </div>
                    </div>
                </aside>
            </main>

            <footer class="dash-footer">
                <StatusBar />
            </footer>
        </div>
    }
}

#[component]
fn StatusBar() -> impl IntoView {
    let state = use_app_state();
    let connection = state.connection;
    let error = state.error;

    view! {
        <div class="status-bar">
            <div class="sb-connection">
                <span class="sb-label">"Status:"</span>
                <span class=move || format!("sb-value {}", connection.get().css_class())>
                    {move || connection.get().label()}
                </span>
            </div>

            {move || {
                error.get().map(|e| {
                    view! {
                        <div class="sb-error">
                            <span class="error-icon">"⚠"</span>
                            <span class="error-msg">{e}</span>
                        </div>
                    }
                })
            }}

            <div class="sb-version">
                <span>"v0.1.0"</span>
            </div>
        </div>
    }
}