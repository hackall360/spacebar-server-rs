use anyhow::{anyhow, Result};
use lapin::{options::{BasicAckOptions, BasicConsumeOptions, BasicPublishOptions, ExchangeDeclareOptions, QueueBindOptions, QueueDeclareOptions}, types::FieldTable, BasicProperties, Channel, Connection, ConnectionProperties, ExchangeKind};
use tokio::sync::{broadcast, OnceCell};
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Event {
    pub event: String,
    pub data: Value,
    pub guild_id: Option<String>,
    pub channel_id: Option<String>,
    pub user_id: Option<String>,
}

static RABBIT_CONN: OnceCell<Connection> = OnceCell::const_new();
static RABBIT_CH: OnceCell<Channel> = OnceCell::const_new();
static LOCAL_TX: OnceCell<broadcast::Sender<Event>> = OnceCell::const_new();

pub async fn init_event() -> Result<()> {
    if RABBIT_CH.get().is_some() || LOCAL_TX.get().is_some() {
        return Ok(());
    }

    let cfg = config::Config::init().await;
    if let Some(host) = &cfg.rabbitmq.host {
        if let Ok(conn) = Connection::connect(host, ConnectionProperties::default()).await {
            let ch = conn.create_channel().await?;
            RABBIT_CONN.set(conn).ok();
            RABBIT_CH.set(ch).ok();
            return Ok(());
        }
    }

    let (tx, _rx) = broadcast::channel(100);
    LOCAL_TX.set(tx).ok();
    Ok(())
}

pub async fn emit_event(event: Event) -> Result<()> {
    let id = event
        .guild_id
        .clone()
        .or(event.channel_id.clone())
        .or(event.user_id.clone())
        .ok_or_else(|| anyhow!("event doesn't contain any id"))?;

    if let Some(ch) = RABBIT_CH.get() {
        ch.exchange_declare(
            &id,
            ExchangeKind::Fanout,
            ExchangeDeclareOptions { durable: false, ..Default::default() },
            FieldTable::default(),
        )
        .await?;
        let payload = serde_json::to_vec(&event.data)?;
        let props = BasicProperties::default().with_type(event.event.clone().into());
        ch.basic_publish(&id, "", BasicPublishOptions::default(), &payload, props)
            .await?
            .await?;
    } else if let Some(tx) = LOCAL_TX.get() {
        let _ = tx.send(event);
    }
    Ok(())
}

pub type Cancel = Box<dyn FnOnce() + Send + Sync>;

pub async fn listen_event<F>(id: &str, callback: F) -> Result<Cancel>
where
    F: Fn(Event) + Send + Sync + 'static,
{
    if let Some(ch) = RABBIT_CH.get() {
        ch.exchange_declare(
            id,
            ExchangeKind::Fanout,
            ExchangeDeclareOptions { durable: false, ..Default::default() },
            FieldTable::default(),
        )
        .await?;
        let queue = ch
            .queue_declare(
                "",
                QueueDeclareOptions { exclusive: true, auto_delete: true, ..Default::default() },
                FieldTable::default(),
            )
            .await?;
        ch.queue_bind(queue.name().as_str(), id, "", QueueBindOptions::default(), FieldTable::default())
            .await?;
        let consumer = ch
            .basic_consume(
                queue.name().as_str(),
                "",
                BasicConsumeOptions::default(),
                FieldTable::default(),
            )
            .await?;
        let cb = Arc::new(callback);
        let id_string = id.to_string();
        let handle = tokio::spawn(async move {
            let mut consumer = consumer;
            while let Some(delivery) = consumer.next().await {
                if let Ok(delivery) = delivery {
                    let data: Value = serde_json::from_slice(&delivery.data).unwrap_or(Value::Null);
                    let event_name = delivery
                        .properties
                        .kind()
                        .as_ref()
                        .map(|s| s.as_str().to_string())
                        .unwrap_or_default();
                    let evt = Event {
                        event: event_name,
                        data,
                        guild_id: Some(id_string.clone()),
                        channel_id: None,
                        user_id: None,
                    };
                    (cb)(evt);
                    let _ = delivery.ack(BasicAckOptions::default()).await;
                }
            }
        });
        let cancel = move || {
            handle.abort();
        };
        Ok(Box::new(cancel))
    } else if let Some(tx) = LOCAL_TX.get() {
        let mut rx = tx.subscribe();
        let cb = Arc::new(callback);
        let id_string = id.to_string();
        let handle = tokio::spawn(async move {
            loop {
                match rx.recv().await {
                    Ok(evt) => {
                        let evt_id = evt
                            .guild_id
                            .as_ref()
                            .or(evt.channel_id.as_ref())
                            .or(evt.user_id.as_ref())
                            .map(|s| s.as_str());
                        if evt_id == Some(id_string.as_str()) {
                            (cb)(evt);
                        }
                    }
                    Err(broadcast::error::RecvError::Closed) => break,
                    Err(broadcast::error::RecvError::Lagged(_)) => continue,
                }
            }
        });
        let cancel = move || {
            handle.abort();
        };
        Ok(Box::new(cancel))
    } else {
        Err(anyhow!("events system not initialized"))
    }
}
