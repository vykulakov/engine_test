use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Market {
    #[serde(alias = "name")]
    #[serde(alias = "symbol")]
    pub symbol: String,
}

/// Henry: I like this interface. However, some
/// explanation on the purpose of each methods will be great.
#[async_trait]
pub trait Exchange {
    fn get_id(&self) -> String;
    fn is_active(&self) -> bool;
    /// Henry: I would create custom error class for Exchange ie `enum ExchangeError {}`
    /// because it gives us the opportunity to define the different error types
    ///  leading to more ergonomic code.
    /// 
    /// Also, you can wrap these into another Result.
    ///
    /// ```rust
    /// enum ExchangeError {
    ///   HttpError(String),
    ///   ...
    /// }
    /// 
    /// let ExchangeResult<T> = Result<T, ExchangeError>;
    /// 
    /// // Hence
    /// async fn get_markets(&self) -> ExchangeResult<Vec<Market>>;
    /// async fn subscribe_to_books(&self) -> ExchangeResult<()>;
    async fn get_markets(&self) -> Result<Vec<Market>, String>;
    async fn subscribe_to_books(&self) -> Result<(), String>;
}

#[derive(Clone, Debug)]
pub struct OrderbookItem {
    pub id: i128,
    /// Henry: serde::Value is too general as a type here.
    /// Would prefer to have deserialized it to 
    /// f64 or a crate like `rust_decimal` for higher precision
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
        /// Henry: Repeated code. Would extract this into a static function 
        /// fn take_top_n(sequence, &Vec<OrderbookItem>) -> Vec<OrderbookItem>
        /// 
        /// My implementation
        /// 
        /// ```rust
        /// fn take_top_n(sequence: &Vec<OrderbookItem>, n: i64) -> Vec<OrderbookItem> {
        ///   if sequence.len() <= n {
        ///       return sequence.clone();
        ///   }
        ///   return sequence[0..n].to_vec();
        /// }
        /// ```
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
            /// Henry: This is a bug.
            /// Should be `self.bids.get(i)`
            bids.push(self.asks.get(i).unwrap().clone());
        }

        /// Henry: This function is peeking the top of orderbook.
        /// Would prefer to create another struct to represent the top of
        /// orderbook rather than reusing Snapshot struct.
        OrderbookSnapshot {
            asks: asks,
            bids: bids,
            updated_at: self.updated_at.clone(),
            received_at: self.received_at.clone(),
        }
    }
}
