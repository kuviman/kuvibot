use futures::FutureExt;

use crate::eventsub;

#[derive(Clone)]
pub struct Tokens {
    pub bot: twitch_oauth2::UserToken,
    pub channel: twitch_oauth2::UserToken,
}

pub struct TwitchApi {
    event_receiver: async_channel::Receiver<Event>,
    say_sender: async_channel::Sender<String>,
}

impl TwitchApi {
    pub async fn connect(channel: &str, tokens: &Tokens) -> eyre::Result<Self> {
        let (event_sender, event_receiver) = async_channel::unbounded();
        let (eventsub_sender, eventsub_receiver) = async_channel::bounded(10);
        tokio::spawn({
            let sender = event_sender.clone();
            async move {
                while let Ok(event) = eventsub_receiver.recv().await {
                    let _ = sender.send(Event::EventSub(event)).await;
                }
            }
        });
        let _eventsub = tokio::spawn({
            let channel = channel.to_owned();
            let token = tokens.channel.clone();
            async move {
                eventsub::run(&channel, &token, eventsub_sender).await?;
                Ok::<_, eyre::Error>(())
            }
        });
        let (say_sender, say_receiver) = async_channel::bounded::<String>(1);
        let _tmi = tokio::spawn({
            let channel = channel.to_owned();
            let token = tokens.bot.clone();
            async move {
                let mut tmi = tmi::Client::connect_with(
                    tmi::client::Config::new(tmi::Credentials::new(
                        token.login,
                        format!("oauth:{}", token.access_token.as_str()),
                    )),
                    tmi::client::DEFAULT_TIMEOUT,
                )
                .await
                .expect("Failed to connect to tmi");

                let channel = tmi::Channel::parse(format!("#{channel}"))?;
                tmi.join_all([&channel]).await?;

                log::info!("Joined tmi");

                loop {
                    futures::select! {
                        msg = tmi.recv().fuse() => {
                            let msg = msg?;
                            match msg.as_typed()? {
                                tmi::Message::Reconnect => {
                                    tmi.reconnect().await?;
                                    tmi.join_all([&channel]).await?;
                                }
                                tmi::Message::Ping(ping) => {
                                    tmi.pong(&ping).await?;
                                }
                                _ => {}
                            };
                            let _ = event_sender.send(Event::Tmi(msg)).await;
                        }
                        say = say_receiver.recv().fuse() => {
                            if let Ok(say) = say {
                                tmi.privmsg(&channel, &say).send().await?;
                            }
                        }
                    }
                }

                #[allow(unreachable_code)]
                Ok::<_, eyre::Error>(())
            }
        });
        Ok(Self {
            event_receiver,
            say_sender,
        })
    }

    pub async fn recv(&mut self) -> Event {
        match self.event_receiver.recv().await {
            Ok(event) => event,
            Err(e) => {
                panic!("{e}");
            }
        }
    }

    pub async fn say(&mut self, text: impl AsRef<str>) {
        let _ = self.say_sender.send(text.as_ref().to_owned()).await;
    }
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug)]
pub enum Event {
    EventSub(twitch_api::eventsub::Event),
    Tmi(tmi::IrcMessage),
}
