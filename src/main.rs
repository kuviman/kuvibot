use std::collections::BTreeMap;
use std::path::PathBuf;

use chrono::Datelike;
use eyre::OptionExt;
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
    pub pushups: i64,
}

#[derive(serde::Deserialize)]
pub struct FffConfig {
    pub ask: Option<String>,
}

#[derive(serde::Deserialize)]
pub struct Config {
    pub bot_account: String,
    pub channel: String,
    pub pushup_rewards: Vec<PushupRewardConfig>,
    pub text_commands: Vec<TextCommand>,
    pub fff: FffConfig,
}

#[derive(Default, serde::Serialize, serde::Deserialize)]
struct Save {
    pushups: BTreeMap<String, BTreeMap<chrono::NaiveDate, i64>>,
    remembers: Vec<String>,
    #[serde(default)]
    holdon: usize,
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

#[derive(clap::Parser)]
struct CliArgs {
    config: Option<PathBuf>,
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .parse_env("LOG")
        .init();

    let cli_args: CliArgs = clap::Parser::parse();

    let mut kast = kast::Kast::new().unwrap();
    let kast_bot = kast
        .eval_file(std::path::Path::new(option_env!("BOT_KS").unwrap_or("src-ks")).join("main.ks"))
        .unwrap();

    let config: Config = {
        let config = cli_args
            .config
            .as_deref()
            .or(option_env!("CONFIG").map(std::path::Path::new))
            .ok_or_eyre("Specify config")?;
        toml::de::from_str(&std::fs::read_to_string(config)?)?
    };
    let tokens = get_tokens(&config).await?;
    let mut ttv = TwitchApi::connect(&config.channel, &tokens).await?;
    ttv.say("Hello, im a bot").await;

    let mut save = Save::load()?;

    let mut default_pushups = None;
    let mut last_holdon = None;

    loop {
        let event = ttv.recv().await;
        match event {
            Event::EventSub(
                twitch_api::eventsub::Event::ChannelPointsCustomRewardRedemptionAddV1(redemption),
            ) => {
                match redemption.message {
                    twitch_api::eventsub::Message::Notification(data) => {
                        if let Some(pushups) = config
                            .pushup_rewards
                            .iter()
                            .find(|reward| reward.title == data.reward.title)
                        {
                            default_pushups = Some(pushups.pushups);
                            ttv.say(format!(
                                "type !done in chat if you did {} pushups",
                                pushups.pushups,
                            ))
                            .await;
                        }
                    }
                    _ => todo!(),
                };
            }
            Event::Tmi(msg) => {
                if let tmi::Message::Privmsg(pmsg) = msg.as_typed()? {
                    let sender = &*pmsg.sender().name();
                    let reply_to_msg_id = pmsg.id();
                    let msg = pmsg.text().trim();

                    let run_kast_bot = async || -> eyre::Result<Option<String>> {
                        let reply = kast
                            .call(
                                kast_bot.clone(),
                                kast::ValueShape::String(msg.to_owned()).into(),
                            )
                            .await?;
                        let reply = reply.into_inferred()?.into_variant()?;
                        Ok(if reply.name == "Some" {
                            let reply = reply
                                .value
                                .ok_or(eyre::eyre!("kast return :Some with no value???"))?;
                            let reply = reply.into_value()?.into_inferred()?.into_string()?;
                            Some(reply)
                        } else {
                            None
                        })
                    };

                    match run_kast_bot().await {
                        Ok(reply) => {
                            if let Some(reply_text) = reply {
                                ttv.reply(reply_text, reply_to_msg_id).await;
                            }
                        }
                        Err(e) => log::error!("{e:?}"),
                    }

                    if msg.contains("69") {
                        ttv.reply("nice", reply_to_msg_id).await;
                    }
                    if let Some(cmd) = msg.split_whitespace().next() {
                        let text = msg.strip_prefix(cmd).unwrap().trim();
                        match cmd {
                            "!holdon" => {
                                let time = chrono::Local::now();
                                // Timeout to filter spam
                                if last_holdon.is_none_or(|last| {
                                    time.signed_duration_since(last).num_seconds() > 10
                                }) {
                                    save.holdon += 1;
                                    ttv.reply(
                                        format!("The hold has been on {} times", save.holdon),
                                        reply_to_msg_id,
                                    )
                                    .await;
                                    last_holdon = Some(time);
                                }
                                save.save()?;
                            }
                            "!done" => {
                                let amount = if text.trim().is_empty() {
                                    match default_pushups {
                                        Some(amount) => amount,
                                        None => {
                                            ttv.reply("How many?", reply_to_msg_id).await;
                                            continue;
                                        }
                                    }
                                } else {
                                    match text.parse() {
                                        Ok(number) => number,
                                        Err(_) => {
                                            ttv.reply("wut", reply_to_msg_id).await;
                                            continue;
                                        }
                                    }
                                };
                                let today = save
                                    .pushups
                                    .entry(pmsg.sender().name().into_owned())
                                    .or_default()
                                    .entry(today())
                                    .or_default();
                                *today += amount;
                                ttv.reply(
                                    format!("good job!, you did {} pushups today", *today),
                                    reply_to_msg_id,
                                )
                                .await;
                                save.save()?;
                            }
                            "!pushboard" => {
                                let mut all: Vec<(&str, i64)> = save
                                    .pushups
                                    .iter()
                                    .map(|(who, log)| (who.as_str(), log.values().copied().sum()))
                                    .collect();
                                all.sort_by_key(|(_name, pushups)| -pushups);
                                let mut top = String::new();
                                for (index, (who, pushups)) in all.into_iter().take(5).enumerate() {
                                    if index != 0 {
                                        top += ", ";
                                    }
                                    let rank = index + 1;
                                    top += &format!("{rank}. {who} - {pushups}");
                                }
                                ttv.say(format!("Pushups leaderboard: {top}")).await;
                            }
                            "!pushups" => {
                                let person = if text.is_empty() { sender } else { text };
                                let pushups = save.pushups.get(person);
                                let today = pushups
                                    .and_then(|pushups| pushups.get(&today()).copied())
                                    .unwrap_or_default();
                                let total = pushups
                                    .map(|pushups| pushups.values().copied().sum::<i64>())
                                    .unwrap_or_default();
                                if total == 0 {
                                    ttv.say(format!("{person} was never seen doing pushups :O"))
                                        .await;
                                } else {
                                    ttv.say(format!(
                                        "{person}'s pushups today: {today}, total: {total}"
                                    ))
                                    .await;
                                }
                            }
                            "!remember" => {
                                save.remembers.push(text.to_owned());
                                save.save()?;
                                ttv.say("Memory must grow").await;
                            }
                            "!remind" => match save.remembers.choose(&mut rand::rng()) {
                                Some(thing) => {
                                    ttv.say(format!("Remember: {thing}")).await;
                                }
                                None => {
                                    ttv.say("Memory is empty D:").await;
                                }
                            },
                            "!fff" => {
                                if let Some(name) = &config.fff.ask {
                                    ttv.say(format!("Next FFF is in <ask {name}>")).await;
                                } else {
                                    let mut fff = chrono::Utc::now()
                                        .with_timezone(&chrono_tz::Europe::Prague)
                                        .with_time(
                                            chrono::NaiveTime::from_hms_opt(12, 0, 0).unwrap(),
                                        )
                                        .unwrap();
                                    while fff.weekday() != chrono::Weekday::Fri {
                                        fff += chrono::TimeDelta::days(1);
                                    }
                                    let fff = chrono_humanize::HumanTime::from(fff).to_text_en(
                                        chrono_humanize::Accuracy::Precise,
                                        chrono_humanize::Tense::Future,
                                    );
                                    ttv.say(format!("Next FFF is {fff}")).await;
                                }
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
