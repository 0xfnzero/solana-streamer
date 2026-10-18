//! Native local bridge benchmark; input fixture allocation is outside timed loops.
//! cargo run --release --example hotpath_latency -- examples/hotpath_latency.json
use serde::Deserialize;
use serde_json::json;
use solana_streamer_sdk::streaming::event_parser::Protocol;
use solana_streamer_sdk::{parser_sdk, sdk_bridge};
use std::{hint::black_box, time::Instant};

#[derive(Deserialize)]
#[serde(default)]
struct Config {
    warmup: usize,
    samples: usize,
    batch: usize,
}
impl Default for Config {
    fn default() -> Self {
        Self { warmup: 1000, samples: 1000, batch: 32 }
    }
}
fn main() {
    let cfg: Config = std::env::args()
        .nth(1)
        .map(|p| serde_json::from_str(&std::fs::read_to_string(p).unwrap()).unwrap())
        .unwrap_or_default();
    assert!(
        (1..=100000).contains(&cfg.samples)
            && (1..=10000).contains(&cfg.batch)
            && cfg.warmup <= 1000000
    );
    let protocols = [Protocol::PumpFun];
    let mut timings = vec![];
    for hash in [false, true] {
        let mut trade = parser_sdk::core::events::PumpFunTradeEvent::default();
        trade.metadata.slot = 42;
        trade.metadata.tx_index = 7;
        trade.metadata.recent_blockhash =
            hash.then(|| "11111111111111111111111111111111111111111111".into());
        trade.is_buy = true;
        trade.sol_amount = 10000;
        trade.token_amount = 100000;
        let input = parser_sdk::DexEvent::PumpFunTrade(trade);
        let run = |input| sdk_bridge::adapt_event(input, None, 999, &protocols, None).unwrap();
        let check = run(input.clone());
        assert_eq!(check.metadata().slot, 42);
        assert_eq!(check.metadata().recent_blockhash.is_some(), hash);
        for _ in 0..cfg.warmup {
            black_box(run(input.clone()));
        }
        let mut times = vec![];
        for _ in 0..cfg.samples {
            let inputs: Vec<_> = (0..cfg.batch).map(|_| input.clone()).collect();
            let start = Instant::now();
            for ev in inputs {
                black_box(run(black_box(ev)));
            }
            times.push(start.elapsed().as_nanos() as f64 / cfg.batch as f64);
        }
        times.sort_by(|a, b| a.total_cmp(b));
        let p = |q: f64| times[((times.len() - 1) as f64 * q) as usize];
        timings.push(json!({"case":if hash{"pump_trade_blockhash"}else{"pump_trade_no_blockhash"},"p50_ns":p(0.5),"p95_ns":p(0.95),"p99_ns":p(0.99)}));
    }
    println!("{}",serde_json::to_string_pretty(&json!({"sdk":env!("CARGO_PKG_NAME"),"profile":if cfg!(debug_assertions){"debug"}else{"release"},"warmup":cfg.warmup,"samples":cfg.samples,"batch":cfg.batch,"timings":timings,"rpc_calls":0})).unwrap());
}
