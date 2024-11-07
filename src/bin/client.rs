use bytes::Bytes;
use mini_redis::client;
use tokio::sync::mpsc;

#[derive(Debug)]
enum Command {
    Get { key: String },
    Set { key: String, val: Bytes },
}

#[tokio::main]
async fn main() {
    let (tx, mut rx) = mpsc::channel::<Command>(32); // Establish a connection to the server
    let mut client = client::connect("127.0.0.1:6379").await.unwrap();

    let tclient = tokio::spawn(async move {
        loop {
            let received = rx.recv().await.unwrap();

            match received {
                Command::Get { key } => {
                    let res = client.get(&key).await.unwrap();
                    println!("CMD GET : Got value from the server: res: {:?}", res);
                }
                Command::Set { key, val } => {
                    let res = client.set(&key, val).await.unwrap();
                    println!("CMD SET: Got value from the server: res: {:?}", res);
                }
                cmd => panic!("unimplemented"),
            }
        }
    });

    // Spawn two tasks, one gets a key, the other sets a key
    let tx1 = tx.clone();
    let t1 = tokio::spawn(async move {
        tx1.send(Command::Get { key: "foo".into() }).await.unwrap();
    });
    let tx2 = tx.clone();
    let t2 = tokio::spawn(async move {
        tx2.send(Command::Set {
            key: "foo".into(),
            val: "bar".into(),
        })
        .await
        .unwrap();
    });
    tclient.await.unwrap();
    t1.await.unwrap();
    t2.await.unwrap();
    // t1.await.unwrap();
    // t2.await.unwrap();
}
