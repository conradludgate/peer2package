// yoke does this
#![allow(clippy::forget_non_drop)]
use std::{net::SocketAddr, ops::Deref, sync::Arc};

use encoding::{read_message_fixed, write_message_fixed};
use quinn::{Endpoint, SendStream};
use quinn_proto::ClientConfig;
use rustls::{Certificate, PrivateKey};
use serde::{Deserialize, Serialize};
use tokio::io::AsyncWriteExt;
use yoke::{Yoke, Yokeable};

use crate::encoding::{read_message, write_message};

pub mod encoding;
pub mod tls;

pub struct Connection {
    inner: quinn::Connection,
}

#[derive(Serialize, Deserialize, Yokeable, Clone, Copy)]
pub enum Requests<'a> {
    #[serde(borrow)]
    FindNode(Id<'a>),
    #[serde(borrow)]
    FindValue(Id<'a>),
    #[serde(borrow)]
    PutValue(Value<'a>),
}

#[derive(Serialize, Deserialize, Yokeable, Clone, Copy)]
pub struct Value<'a> {
    #[serde(borrow)]
    pub id: Id<'a>,
    pub value_len: usize,
}

#[derive(Serialize, Deserialize, Yokeable, Clone, Copy)]
pub enum Responses<'a> {
    #[serde(borrow)]
    Location(Location<'a>),
    #[serde(borrow)]
    Value(Value<'a>),
}

#[derive(Serialize, Deserialize, Yokeable, Clone, Copy)]
pub struct Location<'a> {
    pub address: &'a str,
    #[serde(borrow)]
    pub id: Id<'a>,
}

#[derive(Serialize, Deserialize, Yokeable, Clone, Copy)]
pub struct Id<'a> {
    pub hash_type: &'a str,
    pub hash: &'a [u8],
}


impl Connection {
    pub async fn new(
        socket: SocketAddr,
        hostname: &str,
        private_key: PrivateKey,
        client_certs: impl IntoIterator<Item = Certificate>,
        // ca_certs: impl IntoIterator<Item = Certificate>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let crypto_config = crate::tls::client(private_key, client_certs)?;
        Self::new_with_crypto_config(socket, hostname, crypto_config).await
    }

    pub async fn new_with_crypto_config(
        socket: SocketAddr,
        hostname: &str,
        crypto_config: impl quinn_proto::crypto::ClientConfig + 'static,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let config = ClientConfig::new(Arc::new(crypto_config));

        let mut client = Endpoint::client("0.0.0.0:0".parse()?)?;
        client.set_default_client_config(config);

        let connecting = client.connect(socket, hostname)?;
        let connection = connecting.await?;
        println!("connection established {:?}", connection.rtt());

        Ok(Self { inner: connection })
    }

    // pub async fn request(
    //     &self,
    //     exchange: &str,
    //     routing_key: &str,
    //     message: &[u8],
    // ) -> Result<(), Box<dyn std::error::Error>> {
    //     let (mut send, mut recv) = self.inner.open_bi().await?;

    //     let message = OpenMessage::Publish(Publish {
    //         exchange,
    //         routing_key,
    //         message,
    //     });
    //     write_message(&message, &mut send).await?;

    //     let confirm: PublishConfirm = read_message_fixed(&mut recv, &mut [0; 1]).await?;
    //     match confirm {
    //         PublishConfirm::Ack => Ok(()),
    //     }
    // }

    // pub async fn consume(&self, queue: &str) -> Result<Message, Box<dyn std::error::Error>> {
    //     let (mut send, mut recv) = self.inner.open_bi().await?;
    //     dbg!("consuming");

    //     write_message(&OpenMessage::Consume(Consume { queue }), &mut send).await?;
    //     send.flush().await?;

    //     dbg!("sent");

    //     let payload = read_message::<&[u8]>(&mut recv).await?;

    //     Ok(Message { payload, send })
    // }
}
