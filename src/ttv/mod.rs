pub mod auth;
mod eventsub;

#[derive(Clone)]
pub struct Tokens {
    pub bot: twitch_oauth2::UserToken,
    pub channel: twitch_oauth2::UserToken,
}

pub struct TwitchApi {
    receiver: async_channel::Receiver<Event>,
}

impl TwitchApi {
    pub async fn connect(channel: &str, tokens: &Tokens) -> eyre::Result<Self> {
        let (sender, receiver) = async_channel::unbounded();
        let (eventsub_sender, eventsub_receiver) = async_channel::bounded(10);
        tokio::spawn({
            let sender = sender.clone();
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

                let channels: Vec<tmi::Channel> = vec![tmi::Channel::parse(format!("#{channel}"))?];
                tmi.join_all(&channels).await?;

                log::info!("Joined tmi");

                loop {
                    let msg = tmi.recv().await?;
                    match msg.as_typed()? {
                        tmi::Message::Reconnect => {
                            tmi.reconnect().await?;
                            tmi.join_all(&channels).await?;
                        }
                        tmi::Message::Ping(ping) => {
                            tmi.pong(&ping).await?;
                        }
                        _ => {}
                    };
                    let _ = sender.send(Event::Tmi(msg)).await;
                }

                #[allow(unreachable_code)]
                Ok::<_, eyre::Error>(())
            }
        });
        Ok(Self { receiver })
    }

    pub async fn recv(&mut self) -> Event {
        match self.receiver.recv().await {
            Ok(event) => event,
            Err(e) => {
                panic!("{e}");
            }
        }
    }
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug)]
pub enum Event {
    EventSub(twitch_api::eventsub::Event),
    Tmi(tmi::IrcMessage),
}
