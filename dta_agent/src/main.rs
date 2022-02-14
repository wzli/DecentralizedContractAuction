use sp_keyring::AccountKeyring;
use std::fmt;
use subxt::{ClientBuilder, DefaultConfig, DefaultExtra, PairSigner};

#[subxt::subxt(runtime_metadata_path = "metadata.scale")]
pub mod runtime {}

impl fmt::Display for runtime::runtime_types::sp_runtime::DispatchError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let d = self.details().unwrap();
        write!(f, "DispatchError {} {} {}", d.pallet, d.error, d.docs)
    }
}

#[async_std::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    //let signer = PairSigner::new(AccountKeyring::Alice.pair());

    let signer = PairSigner::<DefaultConfig, DefaultExtra<DefaultConfig>, _>::new(
        AccountKeyring::Alice.pair(),
    );

    let bidder = PairSigner::<DefaultConfig, DefaultExtra<DefaultConfig>, _>::new(
        AccountKeyring::Bob.pair(),
    );

    let api = ClientBuilder::new()
        .set_url("ws://localhost:9944")
        .build()
        .await?
        .to_runtime_api::<runtime::RuntimeApi<DefaultConfig, DefaultExtra<DefaultConfig>>>();

    let blocknum = api.storage().system().number(None).await?;
    println!("blocknum {blocknum}");

    let tx_result = api
        .tx()
        .task_auction()
        .create(
            AccountKeyring::Alice.into(),
            10000,
            5000,
            blocknum + 1000,
            vec![0xFF; 8],
        )
        .sign_and_submit_then_watch(&signer)
        .await?
        .wait_for_finalized_success()
        .await;

    let events = tx_result.map_err(|e| match e {
        subxt::Error::Runtime(re) => panic!("{}", re.inner()),
        e => e,
    })?;

    for e in events.as_slice() {
        println!("{e:?}");
    }

    let created_event = events
        .find_first_event::<runtime::task_auction::events::Created>()?
        .unwrap();
    println!("{created_event:#?}");

    let (owner, nonce) = created_event.auction_key.clone();
    let auction = api
        .storage()
        .task_auction()
        .auctions(owner, nonce, None)
        .await?;

    println!("{auction:#?}");

    /*
    match tx_result {
        Ok(events) => {
            let auction = events
                .find_first_event::<runtime::task_auction::events::Created>()?
                .unwrap();
            println!("{auction:#?}");
            let new_auction = api
                .storage()
                .task_auction()
                .auctions(auction.auction_id.clone(), None)
                .await?;
            println!("{new_auction:#?}");
            let account = api
                .storage()
                .system()
                .account(auction.auction_id.clone(), None)
                .await?;
            println!("{account:#?}");
            for i in 1..5 {
                let ev = api
                    .tx()
                    .task_auction()
                    .bid(auction.auction_id.clone(), auction.bounty.clone() - i)
                    .sign_and_submit_then_watch(&bidder)
                    .await?
                    .wait_for_finalized_success()
                    .await?;
                let e = ev
                    .find_first_event::<runtime::task_auction::events::Bid>()?
                    .unwrap();
                println!("{e:#?}");
            }
            let bids = api
                .storage()
                .task_auction()
                .bids(auction.auction_id.clone(), None)
                .await?;
            println!("{bids:?}");
        }
        Err(subxt::Error::Runtime(e)) => {
            println!("{}", e.inner());
        }
        Err(e) => Err(e)?,
    }
    */

    let mut auctions = api.storage().task_auction().auctions_iter(None).await?;
    while let Some(auc) = auctions.next().await? {
        println!("{auc:?}");
    }

    Ok(())
}
