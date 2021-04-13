use chrono::prelude::*;
use irc::client::prelude::*;
use serde::{Deserialize, Serialize, Serializer};
use std::collections::{HashMap, VecDeque};
use tokio::sync::oneshot;
use uuid::Uuid;
use warp::Filter;

pub type StateTx = tokio::sync::mpsc::Sender<StateCommand>;
pub type StateRx<T> = tokio::sync::oneshot::Receiver<T>;

pub mod chatbot;
pub mod database;
pub mod server;

lazy_static::lazy_static! {
    static ref CLIENT_ID: String = {
        match std::env::var("CLIENT_ID") {
            Ok(var) => var,
            Err(_) => {
                println!("CLIENT_ID not definied in environment variables.");
                std::process::exit(1)
            }
        }
    };
    static ref CLIENT_SECRET: String = {
        match std::env::var("CLIENT_SECRET") {
            Ok(var) => var,
            Err(_) => {
                println!("CLIENT_SECRET not definied in environment variables.");
                std::process::exit(1)
            }
        }
    };
}

pub fn with_tx<T>(
    tx: tokio::sync::mpsc::Sender<T>,
) -> impl Filter<Extract = (tokio::sync::mpsc::Sender<T>,), Error = std::convert::Infallible> + Clone
where
    T: Send + Sync,
{
    warp::any().map(move || tx.clone())
}

#[derive(Debug)]
pub enum StateCommand {
    AddUser {
        user: String,
        tx: oneshot::Sender<usize>,
    },
    GetQueue(oneshot::Sender<serde_json::Value>),
    GetQueueStatus(oneshot::Sender<bool>),
    FindUser {
        name: String,
        tx: oneshot::Sender<Option<usize>>,
    },
    PeekQueue {
        count: u16,
        tx: oneshot::Sender<Vec<UserEntry>>,
    },
    PopQueue {
        count: u16,
        tx: oneshot::Sender<Option<Vec<UserEntry>>>,
    },
    RemoveUser {
        user: String,
        tx: oneshot::Sender<Option<()>>,
    },
    ToggleQueue(oneshot::Sender<bool>),
}

fn serialize_datetime<S>(date_time: &DateTime<Local>, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    const TIME_FMT: &str = "%H:%M:%S";
    s.serialize_str(&date_time.format(TIME_FMT).to_string())
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct UserEntry {
    pub nickname: String,
    #[serde(serialize_with = "serialize_datetime")]
    pub time_joined: DateTime<Local>,
    pub id: Uuid,
}

#[derive(Default, Deserialize, Serialize)]
pub struct Queue {
    pub queue: VecDeque<UserEntry>,
    pub is_open: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Token {
    pub access_token: String,
    scope: String,
    token_type: String,
}

pub fn pop(num: u16, user_queue: &mut VecDeque<UserEntry>) -> Option<Vec<UserEntry>> {
    let mut popped_entries = Vec::new();
    for _ in 0..num {
        if let Some(x) = user_queue.pop_front() {
            popped_entries.push(x);
        }
    }
    if popped_entries.is_empty() {
        None
    } else {
        Some(popped_entries)
    }
}
pub fn get_user_config(token: &str) -> Config {
    let mut settings = config::Config::default();
    settings
        .merge(config::File::with_name("Settings").required(false))
        .unwrap() // TODO error handling
        .merge(config::Environment::with_prefix("TWITCH"))
        .unwrap(); // TODO error handling
    let config = settings.try_into::<HashMap<String, String>>().unwrap(); // TODO error handling

    let name = match config.get("name") {
        Some(n) => n,
        None => {
            panic!("Expected `name` in Settings.toml or TWITCH_NAME environment variable");
        }
    };
    let channel = match config.get("channel") {
        Some(n) => {
            if n.starts_with('#') {
                n.clone()
            } else {
                format!("#{}", n)
            }
        }
        None => {
            panic!("Expected `channel` in Settings.toml or TWITCH_CHANNEL environment variable");
        }
    };

    Config {
        nickname: Some(name.clone()),
        password: Some(token.to_owned()),
        server: Some("irc.chat.twitch.tv".to_owned()),
        port: Some(6697),
        channels: vec![channel],
        ..Default::default()
    }
}

pub fn find(nickname: &str, user_queue: &VecDeque<UserEntry>) -> Option<usize> {
    user_queue
        .iter()
        .position(|entry: &UserEntry| entry.nickname == nickname)
}

pub fn remove(name: &str, user_queue: &mut VecDeque<UserEntry>) -> Option<()> {
    match find(name, user_queue) {
        Some(index) => user_queue.remove(index).map(|_| ()),
        None => None,
    }
}
