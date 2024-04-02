use std::collections::BTreeMap;

use rand::prelude::*;
use twitch_bot::service;
use twitch_bot::service::{Event, TwitchApi};

mod secret;

fn today() -> chrono::NaiveDate {
    chrono::Local::now().date_naive()
}

#[derive(serde::Deserialize)]
pub struct TextCommand {
    pub alias: String,
    pub text: String,
}

#[derive(serde::Deserialize)]
pub struct PushupRewardConfig {
    pub title: String,
    pub pushups: u64,
}

#[derive(serde::Deserialize)]
pub struct Config {
    pub bot_account: String,
    pub channel: String,
    pub pushup_reward: PushupRewardConfig,
    pub text_commands: Vec<TextCommand>,
}

#[derive(Default, serde::Serialize, serde::Deserialize)]
struct Save {
    pushups: BTreeMap<chrono::NaiveDate, u64>,
    remembers: Vec<String>,
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
                        if data.reward.title == config.pushup_reward.title {
                            let today = save.pushups.entry(today()).or_default();
                            *today += config.pushup_reward.pushups;
                            ttv.say(format!("pushups += {}", config.pushup_reward.pushups))
                                .await;
                            save.save()?;
                        }
                    }
                    _ => todo!(),
                };
            }
            Event::Tmi(msg) => {
                if let tmi::Message::Privmsg(msg) = msg.as_typed()? {
                    let msg = msg.text().trim();
                    if let Some(cmd) = msg.split_whitespace().next() {
                        let text = msg.strip_prefix(cmd).unwrap().trim();
                        match cmd {
                            "!pushups" => {
                                let today = save.pushups.get(&today()).copied().unwrap_or_default();
                                let total = save.pushups.values().copied().sum::<u64>();
                                ttv.say(format!(
                                    "Total pushups today: {today}, Total recorded pushups: {total}"
                                ))
                                .await;
                            }
                            "!remember" => {
                                save.remembers.push(text.to_owned());
                                save.save()?;
                                ttv.say("Memory must grow").await;
                            }
                            "!remind" => match save.remembers.choose(&mut thread_rng()) {
                                Some(thing) => {
                                    ttv.say(format!("Remember: {thing}")).await;
                                }
                                None => {
                                    ttv.say("Memory is empty D:").await;
                                }
                            },
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
