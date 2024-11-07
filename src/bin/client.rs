use bytes::Bytes;
use mini_redis::client;
use tokio::sync::{mpsc, oneshot};

#[derive(Debug)]
enum Command {
    Get {
        key: String,
        resp: oneshot::Sender<Option<Bytes>>,
    },
    Set {
        key: String,
        val: Bytes,
        resp: oneshot::Sender<()>,
    },
}

#[tokio::main]
async fn main() {
    let (tx, mut rx) = mpsc::channel::<Command>(32); // Establish a connection to the server

    let manager = tokio::spawn(async move {
        let mut client = client::connect("127.0.0.1:6379").await.unwrap();
        // loop {
        //     let received = rx.recv().await.unwrap();

        //     match received {
        //         Command::Get { key } => {
        //             let res = client.get(&key).await.unwrap();
        //             println!("CMD GET : Got value from the server: res: {:?}", res);
        //         }
        //         Command::Set { key, val } => {
        //             let res = client.set(&key, val).await.unwrap();
        //             println!("CMD SET: Got value from the server: res: {:?}", res);
        //         }
        //         cmd => panic!("unimplemented"),
        //     }
        // }
        while let Some(cmd) = rx.recv().await {
            use Command::*;

            match cmd {
                Get { key, resp } => {
                    let res = client.get(&key).await.unwrap();
                    let _ = resp.send(res);
                }
                Set { key, val, resp } => {
                    let res = client.set(&key, val).await.unwrap();
                    let _ = resp.send(res);
                }
            }
        }
    });

    // Spawn two tasks, one gets a key, the other sets a key
    let tx1 = tx.clone();
    let t1 = tokio::spawn(async move {
        let (resp_tx, resp_rx) = oneshot::channel();
        let cmd = Command::Get {
            key: "foo".to_string(),
            resp: resp_tx,
        };
        tx1.send(cmd).await.unwrap();
        let res = resp_rx.await;
        println!("GOT = {:?}", res);
    });
    let tx2 = tx.clone();
    let t2 = tokio::spawn(async move {
        let (resp_tx, resp_rx) = oneshot::channel();
        let cmd = Command::Set {
            key: "foo".to_string(),
            val: "bar".into(),
            resp: resp_tx,
        };
        tx2.send(cmd).await.unwrap();
        let res = resp_rx.await;
        println!("GOT = {:?}", res);
    });
    t1.await.unwrap();
    t2.await.unwrap();
    manager.await.unwrap();
    // t1.await.unwrap();
    // t2.await.unwrap();
}
