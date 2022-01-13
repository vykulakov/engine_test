use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Market {
    #[serde(alias = "name")]
    #[serde(alias = "symbol")]
    pub symbol: String,
}

#[async_trait]
pub trait Exchange {
    fn get_id(&self) -> String;
    fn is_active(&self) -> bool;
    async fn get_markets(&self) -> Result<Vec<Market>, String>;
    async fn subscribe_to_books(&self) -> Result<(), String>;
}

#[derive(Clone, Debug)]
pub struct OrderbookItem {
    pub id: i128,
    pub size: Value,
    pub price: Value,
}

#[derive(Debug)]
pub struct OrderbookSnapshot {
    pub asks: Vec<OrderbookItem>,
    pub bids: Vec<OrderbookItem>,
    pub updated_at: u128,
    pub received_at: u128,
}

impl OrderbookSnapshot {
    // Will be used later - DO NOT REMOVE THIS!!!
    pub fn head(&self) -> OrderbookSnapshot {
        let mut asks: Vec<OrderbookItem> = Vec::new();
        let mut i = 0;
        let mut j = 0;
        while i < 10 && i < self.asks.len() {
            i = i + 1;
            asks.push(self.asks.get(i).unwrap().clone());
        }

        let mut bids: Vec<OrderbookItem> = Vec::new();
        while j < 10 && j < self.bids.len() {
            j = j + 1;
            bids.push(self.asks.get(i).unwrap().clone());
        }

        OrderbookSnapshot {
            asks: asks,
            bids: bids,
            updated_at: self.updated_at.clone(),
            received_at: self.received_at.clone(),
        }
    }
}
