use crate::{StateCommand, StateTx, Token};
use async_trait::async_trait;
use futures::prelude::*;
mod irc {
    pub use irc::client::prelude::*;
    pub use irc::error::*;
}
use std::collections::HashMap;

#[derive(Debug)]
pub enum Commands {
    SendMessage(String),
    Token(Token),
}

pub type Tx = tokio::sync::mpsc::Sender<Commands>;
pub type Rx = tokio::sync::mpsc::Receiver<Commands>;

pub struct Message<'a> {
    pub target: &'a str,
    pub sender: &'a str,
    pub message: &'a str,
}

pub struct Args<'a> {
    pub msg: Message<'a>,
    pub writer: &'a irc::Sender,
    pub rx: &'a Rx,
    pub state_tx: &'a StateTx,
}

#[async_trait]
pub trait Handler: Send + Sync {
    async fn handle(&mut self, args: Args<'_>);
}

#[async_trait]
impl<F> Handler for F
where
    F: Fn(Args<'_>),
    F: Send + Sync,
{
    async fn handle(&mut self, args: Args<'_>) {
        (self)(args)
    }
}

struct QueuePos<'a> {
    index: Option<usize>,
    user_nickname: &'a str,
    group_size: usize,
    wait_per_group: usize, // TODO this should be a chrono::Duration
}

impl std::fmt::Display for QueuePos<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.index {
            Some(index) => {
                let leading_groups = index / self.group_size;
                let wait_time = leading_groups * self.wait_per_group;
                let leading_groups_preamble = match leading_groups {
                    0 => "You're on deck".to_owned(),
                    _ => format!("There are {} groups ahead of you", leading_groups),
                };
                write!(
                    f,
                    "{} is #{}. {}, wait time is approximately {}-{} minutes",
                    self.user_nickname,
                    index + 1,
                    leading_groups_preamble,
                    wait_time,
                    wait_time + 5
                )
            }
            None => {
                write!(f, "{} is not in the queue", self.user_nickname)
            }
        }
    }
}

pub struct Bot {
    channel: String,
    client: irc::Client,
    commands: HashMap<String, Box<dyn Handler>>,
    rx: Rx,
}

impl Bot {
    pub async fn new(user_config: irc::Config, rx: Rx) -> Result<Bot, irc::Error> {
        let channel = user_config.channels.iter().take(1).cloned().collect();
        let client = irc::Client::from_config(user_config).await?;
        client.identify()?;
        Ok(Bot {
            channel,
            client,
            commands: HashMap::new(),
            rx,
        })
    }

    // add this command to the bot
    pub fn with_command(&mut self, name: impl Into<String>, cmd: impl Handler + 'static) {
        self.commands.insert(name.into(), Box::new(cmd));
    }

    // run the bot until its done
    pub async fn run(&mut self, tx: StateTx) {
        tracing::debug!("starting main loop");
        let mut stream = self.client.stream().unwrap(); // TODO error handling
        let sender = self.client.sender();

        loop {
            tokio::select! {
                // Oh wow that Option<Result<_>> nesting is pretty gnarly
                Some(Ok(message)) = stream.next() => {
                    tracing::debug!("{}", message);
                    if let irc::Command::PRIVMSG(ref _target, ref msg) = message.command {
                        // see if its a command and do stuff with it
                        if let Some(cmd) = Self::parse_command(msg) {
                            if let Some(command) = self.commands.get_mut(cmd) {
                                tracing::trace!("dispatching to: {}", cmd.escape_debug());

                                let args = Args {
                                    msg: Message {
                                        target: message.response_target().unwrap(), // TODO error handling
                                        sender: message.source_nickname().unwrap(), // TODO error handling
                                        message: msg,
                                    },
                                    writer: &sender,
                                    rx: &self.rx,
                                    state_tx: &tx,
                                };

                                command.handle(args).await;
                            }
                        }
                    }
                },
                Some(command) = self.rx.recv() => {
                    match command {
                        Commands::SendMessage(message) => {sender.send_privmsg(&self.channel, &message).unwrap();}, // TODO error handling
                        Commands::Token(_) => tracing::debug!("Already handled token"),
                    }
                }
                else => break,
            }
        }

        tracing::trace!("end of main loop");
    }

    pub fn parse_command(input: &str) -> Option<&str> {
        if !input.starts_with('!') {
            return None;
        }
        input.splitn(2, ' ').next()
    }
}

// GAT support can't come soon enough

struct Peek;

#[async_trait]
impl Handler for Peek {
    async fn handle(&mut self, args: Args<'_>) {
        let (state_tx, state_rx) = tokio::sync::oneshot::channel();
        args.state_tx
            .send(StateCommand::PeekQueue {
                count: 4,
                tx: state_tx,
            })
            .await
            .unwrap(); // TODO error handling

        let first_n: Vec<String> = state_rx
            .await
            .unwrap() // TODO error handling
            .into_iter()
            .map(|u| u.nickname)
            .collect();

        if !first_n.is_empty() {
            args.writer
                .send_privmsg(args.msg.target, &first_n.join(", "))
                .unwrap(); // TODO error handling
        } else {
            args.writer
                .send_privmsg(args.msg.target, "The queue is empty")
                .unwrap(); // TODO error handling
        }
    }
}

struct Join;

#[async_trait]
impl Handler for Join {
    async fn handle(&mut self, args: Args<'_>) {
        let (resp_tx, resp_rx) = tokio::sync::oneshot::channel();
        args.state_tx
            .send(StateCommand::GetQueueStatus(resp_tx))
            .await
            .unwrap(); // TODO error handling
        let status = resp_rx.await.unwrap(); // TODO error handling

        if !status {
            return;
        }

        let (resp_tx, resp_rx) = tokio::sync::oneshot::channel();
        args.state_tx
            .send(StateCommand::AddUser {
                user: args.msg.sender.to_string(),
                tx: resp_tx,
            })
            .await
            .unwrap(); // TODO error handling
        let index = resp_rx.await.unwrap(); // TODO error handling

        let queue_pos = QueuePos {
            index: Some(index),
            user_nickname: args.msg.sender,
            group_size: 4,
            wait_per_group: 5,
        };

        args.writer
            .send_privmsg(&args.msg.target, &format!("{}", queue_pos))
            .unwrap(); // TODO error handling
    }
}

struct Place;

#[async_trait]
impl Handler for Place {
    async fn handle(&mut self, args: Args<'_>) {
        let (resp_tx, resp_rx) = tokio::sync::oneshot::channel();
        args.state_tx
            .send(StateCommand::FindUser {
                name: args.msg.sender.to_string(),
                tx: resp_tx,
            })
            .await
            .unwrap(); // TODO error handling
        let index = resp_rx.await.unwrap(); // TODO error handling

        let queue_pos = QueuePos {
            index,
            user_nickname: args.msg.sender,
            group_size: 4,
            wait_per_group: 5,
        };
        args.writer
            .send_privmsg(args.msg.target, &format!("{}", queue_pos))
            .unwrap(); // TODO error handling
    }
}

struct Leave;
#[async_trait]
impl Handler for Leave {
    async fn handle(&mut self, args: Args<'_>) {
        let (resp_tx, resp_rx) = tokio::sync::oneshot::channel();
        args.state_tx
            .send(StateCommand::RemoveUser {
                user: args.msg.sender.to_string(),
                tx: resp_tx,
            })
            .await
            .unwrap(); // TODO error handling
        let user = resp_rx.await.unwrap(); // TODO error handling
        if user.is_some() {
            args.writer
                .send_privmsg(
                    args.msg.target,
                    &format!("{} has been removed from the queue.", args.msg.sender),
                )
                .unwrap(); // TODO error handling
        }
    }
}

pub fn build_bot(bot: &mut Bot) {
    bot.with_command("!join", Join {});
    bot.with_command("!next", Peek {});
    bot.with_command("!place", Place {});
    bot.with_command("!leave", Leave {});
}
