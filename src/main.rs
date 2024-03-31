mod secret;
mod ttv;

#[derive(serde::Deserialize)]
pub struct Config {
    pub bot_account: String,
    pub channel: String,
}

async fn get_tokens(config: &Config) -> eyre::Result<ttv::Tokens> {
    let secrets = secret::Secrets::init()?;
    let bot = secrets.get_user_token(&config.bot_account).await?;
    let channel = secrets.get_user_token(&config.channel).await?;
    Ok(ttv::Tokens { bot, channel })
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .parse_env("LOG")
        .init();

    let config: Config = toml::de::from_str(&std::fs::read_to_string("config.toml")?)?;
    let tokens = get_tokens(&config).await?;
    let mut ttv = ttv::TwitchApi::connect(&config.channel, &tokens).await?;
    ttv.say("Hello, im a bot").await;
    loop {
        let event = ttv.recv().await;
        match event {
            ttv::Event::EventSub(
                twitch_api::eventsub::Event::ChannelPointsCustomRewardRedemptionAddV1(redemption),
            ) => {
                log::info!("{redemption:#?}");
            }
            ttv::Event::Tmi(msg) => {
                if let tmi::Message::Privmsg(msg) = msg.as_typed()? {
                    log::info!("{msg:#?}");
                }
            }
            _ => {}
        };
    }
}
