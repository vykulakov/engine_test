use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use tokio::{task, time};
use tokio::signal;

use exchanges::Exchange;

use crate::exchanges::Market;
use crate::ftx::{build, Ftx};

mod exchanges;
mod ftx;

#[tokio::main]
async fn main() {
    println!("start!");
    let exchanges = init_exchanges();

    /// Henry: This works as you have multiple readers and writers accessin this state across various tasks.
    /// I would prefer not to communcate the state and changes via channel instead.
    /// 
    /// Use oneshot channel with response for reads in init_state_handler
    /// All writes will need to send `markets_state` mpsc channel.
    /// Can discuss further.
    let markets_state: Arc<std::sync::RwLock<HashMap<String, Vec<Market>>>> = Arc::new(RwLock::new(HashMap::new()));

    init_state_handler(markets_state.clone()).await;

    for ex in exchanges {
        let id = ex.get_id();

        match ex.subscribe_to_books().await {
            Ok(_) => {}
            Err(e) => {
                println!("{}: cannot subscribe to books: {}", id, e)
            }
        };

        let markets_state = Arc::clone(&markets_state);

        task::spawn(async move {
            println!("{}: starting", id.clone());

            let mut interval = time::interval(time::Duration::from_secs(90));
            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        let res = ex.get_markets();
                        match res.await {
                            Ok(markets) => {
                                let mut state = markets_state.write().expect("Cannot lock markets");
                                state.insert(id.clone(), markets);
                            }
                            Err(e) => {
                                println!("{}: markets handler failed: {}", id, e);
                                break;
                            }
                        };
                    }
                }
            }

            println!("{}: finished", id.clone());
        });
    }

    println!("Wait termination signal");
    match signal::ctrl_c().await {
        Ok(_) => {
            println!("Got termination signal");
        }
        Err(e) => {
            println!("Cannot wait termination signal: {}", e);
        }
    }
}

// Inits exchanges
fn init_exchanges() -> Vec<Box<dyn Exchange + Send>> {
    println!("exchanges!");

    let mut exchanges = Vec::new();

    /// Henry: Use enum
    // Available exchanges
    // All the exchanges should have if in the loop below!
    let list = ["ftx", "ftx-test"];

    // Iterate over list
    for e in list {
        if e == "ftx" {
            exchanges.push(Box::new(ftx::build(String::from(e))) as Box<dyn Exchange + Send>);
        }
        if e == "ftx-test" {
            exchanges.push(Box::new(ftx::build(String::from(e))) as Box<dyn Exchange + Send>);
        }
    }

    exchanges
}

// Inits the handler to process exchange states
async fn init_state_handler(markets_state: Arc<RwLock<HashMap<String, Vec<Market>>>>) {
    println!("state handlers!");

    task::spawn(async move {
        println!("starting");

        let mut interval = time::interval(time::Duration::from_secs(15));
        loop {
            tokio::select! {
                _ = interval.tick() => {
                    let state = markets_state.read().expect("Cannot lock markets");
                    println!("    markets:");
                    for (k, v) in state.clone().into_iter() {
                        println!("        {}: {:?}", k, v);
                    }
                }
            }
        }
    });
}
