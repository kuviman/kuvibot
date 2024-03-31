use futures::prelude::*;

mod auth;
mod eventsub;
mod secret;

#[derive(serde::Deserialize)]
struct Config {
    bot_account: String,
    channel: String,
}

#[derive(Clone)]
struct Tokens {
    bot: twitch_oauth2::UserToken,
    channel: twitch_oauth2::UserToken,
}

impl Tokens {
    async fn get() -> eyre::Result<Self> {
        let secrets = secret::Secrets::init()?;
        let config: Config = toml::de::from_str(&std::fs::read_to_string("config.toml")?)?;
        let bot = secrets.get_user_token(&config.bot_account).await?;
        let channel = secrets.get_user_token(&config.channel).await?;
        Ok(Self { bot, channel })
    }
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .parse_env("LOG")
        .init();

    let tokens = Tokens::get().await?;

    tokio::spawn(async move {
        eventsub::run("kuviman", &tokens.channel).await?;
        Ok::<_, eyre::Error>(())
    });

    let mut tmi = tmi::Client::connect_with(
        tmi::client::Config::new(tmi::Credentials::new(
            "kuvibot",
            format!("oauth:{}", tokens.bot.access_token.as_str()),
        )),
        tmi::client::DEFAULT_TIMEOUT,
    )
    .await
    .expect("Failed to connect to tmi");

    let channels: Vec<tmi::Channel> = vec![tmi::Channel::parse("#kuviman".to_owned())?];
    tmi.join_all(&channels).await?;

    log::info!("Joined tmi");

    loop {
        let msg = tmi.recv().await?;
        match msg.as_typed()? {
            tmi::Message::Privmsg(msg) => {
                println!("{}: {}", msg.sender().name(), msg.text());
            }
            tmi::Message::Reconnect => {
                tmi.reconnect().await?;
                tmi.join_all(&channels).await?;
            }
            tmi::Message::Ping(ping) => {
                tmi.pong(&ping).await?;
            }
            _ => {}
        };
    }
}
