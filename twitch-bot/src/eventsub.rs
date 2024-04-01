use futures::prelude::*;
use twitch_api::eventsub::channel::ChannelPointsCustomRewardRedemptionAddV1;

pub async fn run(
    channel: &str,
    token: &twitch_oauth2::UserToken,
    sender: async_channel::Sender<twitch_api::eventsub::Event>,
) -> eyre::Result<()> {
    let helix = twitch_api::HelixClient::<reqwest::Client>::new();
    let user = helix
        .get_user_from_login(channel, token)
        .await?
        .expect("No channel {channel:?}");
    let mut connect_url = "wss://eventsub.wss.twitch.tv/ws".to_owned();
    'reconnect: loop {
        let mut eventsub = websocket_lite::ClientBuilder::new(&connect_url)?
            .async_connect()
            .await
            .map_err(|e| eyre::eyre!(e))?;
        while let Some(event) = eventsub.next().await {
            let event = event.map_err(|e| eyre::eyre!(e))?;
            match event.opcode() {
                websocket_lite::Opcode::Text => {
                    let message = event
                        .as_text()
                        .expect("Expected text messages for Text opcodes WTF");
                    match twitch_api::eventsub::Event::parse_websocket(message)? {
                        twitch_api::eventsub::EventsubWebsocketData::Welcome {
                            metadata: _,
                            payload,
                        } => {
                            helix
                                .create_eventsub_subscription(
                                    ChannelPointsCustomRewardRedemptionAddV1::broadcaster_user_id(
                                        user.id.clone(),
                                    ),
                                    twitch_api::eventsub::Transport::websocket(payload.session.id),
                                    token,
                                )
                                .await?;
                            log::info!("Successful eventsub subscription");
                        }
                        twitch_api::eventsub::EventsubWebsocketData::Keepalive {
                            metadata: _,
                            payload: _,
                        } => {
                            // dont care
                        }
                        twitch_api::eventsub::EventsubWebsocketData::Notification {
                            metadata: _,
                            payload,
                        } => {
                            let _ = sender.send(payload).await;
                        }
                        twitch_api::eventsub::EventsubWebsocketData::Revocation {
                            metadata: _,
                            payload: _,
                        } => {
                            panic!("WTF revocation");
                        }
                        twitch_api::eventsub::EventsubWebsocketData::Reconnect {
                            metadata: _,
                            payload,
                        } => {
                            if let Some(url) = payload.session.reconnect_url {
                                connect_url = url.into_owned();
                            }
                            continue 'reconnect;
                        }
                        _ => todo!(),
                    }
                }
                websocket_lite::Opcode::Binary => {
                    unreachable!("twitch supposed to send text messages wtf")
                }
                websocket_lite::Opcode::Close => break,
                websocket_lite::Opcode::Ping => {
                    log::trace!("GOT PING, sending PONG");
                    eventsub
                        .send(websocket_lite::Message::pong(event.into_data()))
                        .await
                        .map_err(|e| eyre::eyre!(e))?;
                    log::trace!("sent PONG POG");
                }
                websocket_lite::Opcode::Pong => unreachable!("ping"),
            }
        }
        panic!("eventsub stopped sending messages wtf");
    }
}
