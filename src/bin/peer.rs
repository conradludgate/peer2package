use std::{
    collections::HashMap, fs::File, io::BufReader, net::SocketAddr, path::PathBuf, pin::pin,
    sync::Arc,
};

use clap::Parser;
use peer2package::{
    encoding::{read_message, read_message_fixed, write_message, write_message_fixed},
    Requests,
};
use quinn::{Connecting, Endpoint, RecvStream, SendStream, ServerConfig};
use tokio::sync::Notify;

#[derive(clap::Parser)]
struct Args {
    #[arg(long, short = 'c')]
    cert_path: PathBuf,
    #[arg(long, short = 'k')]
    key_path: PathBuf,
    #[arg(long, short = 'a')]
    addr: SocketAddr,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let mut keys =
        rustls_pemfile::pkcs8_private_keys(&mut BufReader::new(File::open(args.key_path)?))?;
    let key = rustls::PrivateKey(keys.remove(0));
    let certs = rustls_pemfile::certs(&mut BufReader::new(File::open(args.cert_path)?))?;

    let crypto_config = peer2package::tls::server(
        key,
        certs.into_iter().map(rustls::Certificate),
        // ca_certs.into_iter().map(rustls::Certificate),
    )?;

    let config = ServerConfig::with_crypto(Arc::new(crypto_config));

    let server = Endpoint::server(config, args.addr)?;

    let state = Arc::new(SharedState {});
    while let Some(connecting) = server.accept().await {
        tokio::spawn(handle_connection(state.clone(), connecting));
    }

    Ok(())
}

struct SharedState {}

async fn handle_connection(state: Arc<SharedState>, connecting: Connecting) {
    match handle_connection_inner(state, connecting).await {
        Ok(()) => {}
        Err(e) => {
            eprintln!("error handling connection {e:?}");
        }
    }
}

async fn handle_connection_inner(
    state: Arc<SharedState>,
    connecting: Connecting,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", connecting.remote_address());

    let connection = connecting.await?;
    println!("connection established {:?}", connection.rtt());

    loop {
        match connection.accept_bi().await {
            Ok((send, recv)) => tokio::spawn(handle_stream(
                state.clone(),
                connection.stable_id(),
                send,
                recv,
            )),
            Err(e) => {
                return Err(e.into());
            }
        };

        println!("connection continue {:?}", connection.rtt());
    }
}

async fn handle_stream(
    state: Arc<SharedState>,
    conn_id: usize,
    send: SendStream,
    recv: RecvStream,
) {
    match handle_stream_inner(state, conn_id, send, recv).await {
        Ok(()) => {}
        Err(e) => {
            eprintln!("error handling stream {e:?}");
        }
    }
}

async fn handle_stream_inner(
    state: Arc<SharedState>,
    conn_id: usize,
    send: SendStream,
    mut recv: RecvStream,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("new stream {}", send.id());

    let message = read_message::<Requests>(&mut recv).await?;

    // match message.get() {
    //     Requests::Publish(publish) => handle_stream_publish(state, send, publish).await?,
    //     Requests::Consume(consume) => {
    //         handle_stream_consume(state, conn_id, send, recv, consume).await?
    //     }
    // }

    Ok(())
}

// async fn handle_stream_publish(
//     state: Arc<SharedState>,
//     mut send: SendStream,
//     publish: &Requests<'_>,
// ) -> Result<(), Box<dyn std::error::Error>> {
//     let Publish {
//         exchange,
//         routing_key,
//         message,
//     } = *publish;
//     let message_str = String::from_utf8_lossy(message);

//     println!("{exchange} {routing_key} {message_str}");

//     let message_id = uuid::Uuid::new_v4();
//     state.messages.insert(message_id.as_bytes(), message)?;

//     let exchange = state.exchanges.get(exchange).ok_or("unknown exchange")?;
//     let queues = exchange.routes.get(routing_key).ok_or("unknown route")?;
//     for queue in queues {
//         let mut key = Vec::new();
//         key.extend_from_slice(queue.as_bytes());
//         key.extend_from_slice(&[0]);
//         key.extend_from_slice(message_id.as_bytes());

//         dbg!("inserting queue");
//         dbg!(state.available.insert(key, &(0_u64).to_le_bytes()))?;
//     }

//     dbg!("flushing");
//     state.messages.flush_async().await?;
//     state.available.flush_async().await?;

//     for queue in queues {
//         let queue = state.queues.get(queue).unwrap();
//         queue.notify.notify_one();
//     }

//     for entry in state.available.iter() {
//         let (key, _) = entry?;
//         println!("{}", String::from_utf8_lossy(&key));
//     }

//     write_message_fixed(&PublishConfirm::Ack, &mut send, &mut [0; 1]).await?;
//     send.finish().await?;

//     Ok(())
// }

// async fn handle_stream_consume(
//     state: Arc<SharedState>,
//     conn_id: usize,
//     mut send: SendStream,
//     mut recv: RecvStream,
//     consume: &Consume<'_>,
// ) -> Result<(), Box<dyn std::error::Error>> {
//     let queue_name = consume.queue;
//     dbg!(&queue_name);

//     let mut key_prefix = Vec::new();
//     key_prefix.extend_from_slice(queue_name.as_bytes());
//     key_prefix.extend_from_slice(&[0]);

//     // use sled::transaction::Transactional;
//     // let res = (&state.published, &state.acquired).transaction(|(message, acquired)| {
//     //     if let Some(entry) = state.published.range((&*key)..).next() {
//     //         let (key2, value) = entry?;
//     //         if let Some(id) = key2.strip_prefix(&*key) {
//     //             let message_id = uuid::Uuid::from_bytes(id.try_into().unwrap());
//     //             dbg!(message_id);

//     //             let message = message.remove(&key2)?.unwrap();
//     //             acquired.insert(&key2, message)?;
//     //             Ok((message_id, value))
//     //         } else {
//     //             Err(ConflictableTransactionError::Abort(()))
//     //         }
//     //     } else {
//     //         Err(ConflictableTransactionError::Abort(()))
//     //     }
//     // });

//     // match res {
//     //     Ok((message_id, value)) => {
//     //         println!("{message_id} {}", String::from_utf8_lossy(&value));
//     //     }
//     //     Err(TransactionError::Abort(())) => return Err("abort".into()),
//     //     Err(TransactionError::Storage(e)) => return Err(e.into()),
//     // }

//     let queue = state.queues.get(queue_name).unwrap();
//     let (key, payload) = loop {
//         let mut notified = pin!(queue.notify.notified());
//         notified.as_mut().enable();

//         dbg!("checking for messages");

//         let message = if let Some(entry) = state.available.scan_prefix(&*key_prefix).next() {
//             let (key, _) = entry?;
//             let message_id = key.strip_prefix(&*key_prefix).unwrap().try_into().unwrap();
//             let message_id = uuid::Uuid::from_bytes(message_id);
//             dbg!(message_id);

//             let mut conn_message_id = Vec::new();
//             conn_message_id.extend_from_slice(&u64::to_ne_bytes(conn_id as u64));
//             conn_message_id.extend_from_slice(&key);

//             // move message from published to acquired
//             let res = (&state.available, &state.acquired, &state.connections).transaction(
//                 |(published, acquired, connections): &(
//                     TransactionalTree,
//                     TransactionalTree,
//                     TransactionalTree,
//                 )| {
//                     dbg!("swap");
//                     // remove from the available queue
//                     let val = published
//                         .remove(&key)?
//                         .ok_or(ConflictableTransactionError::Abort(()))?;
//                     // add to the acquired queue
//                     acquired.insert(&key, val)?;
//                     // add to the acquired list of this connection
//                     connections.insert(&*conn_message_id, &[])?;
//                     Ok(())
//                 },
//             );
//             match res {
//                 Ok(()) => state
//                     .messages
//                     .get(message_id.as_bytes())?
//                     .map(|payload| (key, payload)),
//                 Err(TransactionError::Abort(())) => continue,
//                 Err(TransactionError::Storage(e)) => return Err(e.into()),
//             }
//         } else {
//             dbg!("none with prefix :(");
//             None
//         };

//         // for entry in state.published.iter() {
//         //     let (key, _) = entry?;
//         //     println!("entry {}", String::from_utf8_lossy(&key));
//         // }
//         // for entry in state.acquired.iter() {
//         //     let (key, _) = entry?;
//         //     println!("acquired {}", String::from_utf8_lossy(&key));
//         // }

//         // let mut conn_state = conn_state.messages.lock().await;
//         // let mut message_lock = queue.messages.lock().await;

//         // let message = message_lock.iter_mut().find(|m| !m.inflight);

//         match message {
//             Some(message) => {
//                 // conn_state.insert((queue_name.to_owned(), message.id));
//                 // dbg!(&conn_state);
//                 // message.inflight = true;

//                 break message;
//             }
//             None => {
//                 dbg!("waiting");
//                 // drop(message_lock);
//                 notified.await
//             }
//         };
//     };

//     write_message(&*payload, &mut send).await?;
//     send.finish().await?;

//     let confirm: MessageAck = read_message_fixed(&mut recv, &mut [0; 1]).await?;

//     // let mut conn_state = conn_state.messages.lock().await;
//     // let mut messages = queue.messages.lock().await;
//     // let (idx, message) = messages
//     //     .iter_mut()
//     //     .enumerate()
//     //     .find(|m| m.1.id == message_id)
//     //     .ok_or("invalid message")?;

//     // match dbg!(confirm) {
//     //     MessageAck::Ack => {
//     //         messages.remove(idx);
//     //     }
//     //     MessageAck::Nack => message.inflight = false,
//     //     // DLQ
//     //     MessageAck::Reject => {
//     //         messages.remove(idx);
//     //     }
//     // }

//     let mut conn_message_id = Vec::new();
//     conn_message_id.extend_from_slice(&usize::to_ne_bytes(conn_id));
//     conn_message_id.extend_from_slice(&key);

//     // move message from acquired to available
//     let res = (&state.available, &state.acquired, &state.connections).transaction(
//         |(published, acquired, connections): &(
//             TransactionalTree,
//             TransactionalTree,
//             TransactionalTree,
//         )| {
//             dbg!("release");
//             connections.remove(&key)?.unwrap();
//             acquired.remove(&key)?.unwrap();
//             match confirm {
//                 MessageAck::Ack => {}
//                 MessageAck::Nack => {
//                     published.insert(&key, &[])?;
//                 }
//                 // TODO: DLQ
//                 MessageAck::Reject => {}
//             }
//             Ok(())
//         },
//     );
//     match res {
//         Ok(()) => {}
//         Err(TransactionError::Abort(())) => return Err("aborted".into()),
//         Err(TransactionError::Storage(e)) => return Err(e.into()),
//     }

//     // conn_state.remove(&(queue_name.to_owned(), message_id));
//     // dbg!(&conn_state);

//     Ok(())
// }
