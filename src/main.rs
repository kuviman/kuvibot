use twitch_bot::service;
use twitch_bot::service::{Event, TwitchApi};

mod secret;

#[derive(serde::Deserialize)]
pub struct TextCommand {
    pub alias: String,
    pub text: String,
}

#[derive(serde::Deserialize)]
pub struct Config {
    pub bot_account: String,
    pub channel: String,
    pub pushup_reward: String,
    pub text_commands: Vec<TextCommand>,
}

#[derive(Default, serde::Serialize, serde::Deserialize)]
struct Save {
    pushups: u64,
}

impl Save {
    fn load() -> eyre::Result<Self> {
        match std::fs::File::open("save.json") {
            Ok(file) => {
                let reader = std::io::BufReader::new(file);
                Ok(serde_json::from_reader(reader)?)
            }
            Err(e) => {
                if e.kind() == std::io::ErrorKind::NotFound {
                    Ok(Self::default())
                } else {
                    eyre::bail!(e);
                }
            }
        }
    }
    fn save(&self) -> eyre::Result<()> {
        serde_json::to_writer_pretty(
            std::io::BufWriter::new(std::fs::File::create("save.json")?),
            self,
        )?;
        Ok(())
    }
}

async fn get_tokens(config: &Config) -> eyre::Result<service::Tokens> {
    let secrets = secret::Secrets::init()?;
    let bot = secrets.get_user_token(&config.bot_account).await?;
    let channel = secrets.get_user_token(&config.channel).await?;
    Ok(service::Tokens { bot, channel })
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .parse_env("LOG")
        .init();

    let config: Config = toml::de::from_str(&std::fs::read_to_string("kuvibot.toml")?)?;
    let tokens = get_tokens(&config).await?;
    let mut ttv = TwitchApi::connect(&config.channel, &tokens).await?;
    ttv.say("Hello, im a bot").await;

    let mut save = Save::load()?;

    loop {
        let event = ttv.recv().await;
        match event {
            Event::EventSub(
                twitch_api::eventsub::Event::ChannelPointsCustomRewardRedemptionAddV1(redemption),
            ) => {
                match redemption.message {
                    twitch_api::eventsub::Message::Notification(data) => {
                        if data.reward.title == config.pushup_reward {
                            save.pushups += 10;
                            save.save()?;
                            ttv.say(format!("Total pushups today: {}", save.pushups))
                                .await;
                        }
                    }
                    _ => todo!(),
                };
            }
            Event::Tmi(msg) => {
                if let tmi::Message::Privmsg(msg) = msg.as_typed()? {
                    if let Some(cmd) = msg.text().split_whitespace().next() {
                        match cmd {
                            "!pushups" => {
                                ttv.say(format!("Total pushups today: {}", save.pushups))
                                    .await;
                            }
                            _ => {
                                if let Some(cmd) = config
                                    .text_commands
                                    .iter()
                                    .find(|command| command.alias == cmd)
                                {
                                    ttv.say(&cmd.text).await;
                                }
                            }
                        }
                    }
                }
            }
            _ => {}
        };
    }
}
