use std::cmp::Ordering;
use std::fmt;
use std::sync::{Arc, RwLock};
use std::time::{Duration, SystemTime};

use async_trait::async_trait;
use futures_util::{SinkExt, StreamExt};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tokio::{task, time};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use url::Url;

use crate::exchanges::{Exchange, Market, OrderbookItem, OrderbookSnapshot};

const WS_URL: &str = "wss://ftx.com/ws/";
const BASE_URL: &str = "https://ftx.com";
const MARKETS_PATH: &str = "/api/markets";

#[derive(Debug)]
pub struct Ftx {
    id: String,
    client: Client,
    active: bool,
    snapshot: Arc<RwLock<OrderbookSnapshot>>,
}

pub fn build(id: String) -> Ftx {
    Ftx {
        id,
        client: reqwest::Client::new(),
        active: true,
        snapshot: Arc::new(RwLock::new(OrderbookSnapshot {
            asks: vec![],
            bids: vec![],
            updated_at: 0,
            received_at: 0,
        })),
    }
}

#[async_trait]
impl Exchange for Ftx {
    fn get_id(&self) -> String {
        self.id.clone()
    }

    fn is_active(&self) -> bool {
        self.active
    }

    async fn get_markets(&self) -> Result<Vec<Market>, String> {
        let url = format!("{}{}", BASE_URL, MARKETS_PATH);

        let res = self.client.get(url).send().await.expect("Fail to make markets request");
        let resp = res.json::<MarketsResponse>().await.expect("Fail to parse body of markets request");

        return if resp.success {
            Ok(resp.result.unwrap())
        } else {
            Err(format!("Bad markets response: {}", resp.error.unwrap()))
        };
    }

    async fn subscribe_to_books(&self) -> Result<(), String> {
        let id = self.get_id();

        let url = Url::parse(WS_URL).unwrap();
        let (stream, _) = connect_async(url).await.expect("Cannot connect to FTX WS");

        let (mut write, mut read) = stream.split();

        let req = WsRequest::subscribe("orderbook", "BTC/USD").json();
        let msg = Message::Text(req);

        write.send(msg).await.expect("Cannot write subscription request");
        let msg = read.next().await.unwrap().expect("Cannot read subscription response");
        if msg.is_text() {
            let msg = msg.to_text().unwrap();
            let msg: WsResponse<WsEmptyResponseData> = serde_json::from_str(msg).unwrap();
            if msg.op == "subscribed" {
                println!("{}: subscribed: {} / {}", &id, msg.channel.unwrap(), msg.market.unwrap());
            } else {
                panic!("{}: bad message op: {:?}", &id, msg);
            }
        } else {
            panic!("{}: bad message type: {:?}", id.clone(), msg)
        }

        task::spawn(async move {
            println!("{}: starting orderbook loop", &id);

            let req = WsRequest::ping().json();
            let png = Message::Text(req);

            let mut ping_interval = time::interval(Duration::from_secs(15));

            let mut cnt = 0;
            let mut upd = SystemTime::now();
            loop {
                tokio::select! {
                        m = read.next() => {
                            upd = SystemTime::now();
                            let msg = m.unwrap().unwrap();
                            cnt += 1;
                        }
                        _ = ping_interval.tick() => {
                            println!("send ping, count = {}, last update = {:?}", cnt, upd);
                            write.send(png.clone()).await.expect("Cannot send ping message");
                        }
                    }
            }

            println!("{}: finished orderbook loop", id.clone());
        });

        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct WsRequest {
    op: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    market: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    channel: Option<&'static str>,
}

impl WsRequest {
    fn json(&self) -> String {
        serde_json::to_string(self).unwrap()
    }

    fn ping() -> WsRequest {
        WsRequest {
            op: "ping",
            market: None,
            channel: None,
        }
    }

    fn subscribe(channel: &'static str, market: &'static str) -> WsRequest {
        WsRequest {
            op: "subscribe",
            market: Some(market),
            channel: Some(channel),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct WsResponse<D: WsResponseData> {
    #[serde(alias = "type")]
    op: String,
    msg: Option<String>,
    code: Option<String>,
    data: Option<D>,
    market: Option<String>,
    channel: Option<String>,
}

trait WsResponseData {}

#[derive(Debug, Serialize, Deserialize)]
struct WsEmptyResponseData {}

impl WsResponseData for WsEmptyResponseData {}

#[derive(Debug, Serialize, Deserialize)]
struct MarketsResponse {
    success: bool,
    error: Option<String>,
    result: Option<Vec<Market>>,
}