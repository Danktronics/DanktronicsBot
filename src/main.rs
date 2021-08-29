use std::{
    env, sync::Arc, collections::HashMap
};
use serenity::{
    client::{bridge::voice::ClientVoiceManager, Client, Context, EventHandler},
    model::{
        channel::{Message, Reaction, ReactionType}, 
        gateway::Ready,
        voice::VoiceState,
        prelude::*
    },
    prelude::*,
    async_trait
};

use model::{
    voice::{Recorder, EmptyAudioSource},
    guild::DankGuild,
    voice::create_tts_source
};

mod model;
mod helpers;

struct VoiceManager;

impl TypeMapKey for VoiceManager {
    type Value = Arc<Mutex<ClientVoiceManager>>;
}

struct DankGuildMap;

impl TypeMapKey for DankGuildMap {
    type Value = HashMap<u64, DankGuild>;
}

struct MainHandler;

static PREFIX: &str = "d.";

#[async_trait]
impl EventHandler for MainHandler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("Connected as {}#{:04}", ready.user.name, ready.user.discriminator);
        ctx.set_activity(Activity::playing(&format!("the people here | {}help", PREFIX))).await;
    }

    async fn message(&self, ctx: Context, message: Message) {
        if message.is_private() {
            return;
        }

        {
            let data = ctx.data.read().await;
            let guild_settings_map = data.get::<DankGuildMap>().expect("DankGuildMap not stored in client");
            let guild_settings = guild_settings_map.get(&message.guild_id.unwrap().0);
            if guild_settings.is_some() && guild_settings.unwrap().tts_channels.contains(&message.channel_id.0) {
                check!(guild_settings.unwrap().say_message(helpers::clean_message_content(&message, &ctx.cache).await).await);
            }
        }

        if !message.content.starts_with(PREFIX) {
            return;
        }

        let raw_command_message: String = message.content.chars().skip(PREFIX.len()).collect();
        let mut arguments: Vec<&str> = raw_command_message.split(' ').collect();
        let command: &str = arguments[0];
        arguments.remove(0);

        match command {
            "join" => {
                let channel_id = match ctx.cache.guild(&message.guild_id.unwrap()).await.unwrap().voice_states.get(&message.author.id) {
                    Some(voice_state) => voice_state.channel_id.unwrap(),
                    None => {
                        check!(message.channel_id.say(&ctx.http, "You must be in a voice channel").await);
                        return;
                    }
                };

                let channel = channel_id.to_channel_cached(&ctx.cache).await.unwrap().guild().unwrap();
                if !channel.permissions_for_user(&ctx.cache, &ctx.cache.current_user_field(|user| user.id).await).await.unwrap().connect() {
                    check!(message.channel_id.say(&ctx.http, "I do not have permissions to join your channel").await);
                    return;
                }

                let manager_lock = ctx.data.read().await.get::<VoiceManager>().cloned().expect("VoiceManager not stored in client");
                let mut manager = manager_lock.lock().await;
                {
                    let handler = manager.get(message.guild_id.unwrap());
                    if handler.is_some() && handler.unwrap().channel_id.is_some() && handler.unwrap().channel_id.unwrap() == channel_id {
                        check!(message.channel_id.say(&ctx.http, "I am already in this channel!").await);
                        return;
                    }
                }

                if manager.join(message.guild_id.unwrap(), channel_id).is_some() {
                    let mut data = ctx.data.write().await;
                    let guild_settings_map = data.get_mut::<DankGuildMap>().expect("DankGuildMap not stored in client");
                    let guild_settings = guild_settings_map.entry(message.guild_id.unwrap().0).or_insert_with(|| DankGuild::new(message.guild_id.unwrap().into()));

                    guild_settings.initialize_tts(manager_lock.clone());
                    check!(message.channel_id.say(&ctx.http, format!("Successfully joined **{}**", channel_id.name(&ctx.cache).await.unwrap())).await);
                } else {
                    check!(message.channel_id.say(&ctx.http, "Failed to join your voice channel").await);
                }
            },
            "help" => {
                check!(message.channel_id.say(&ctx.http, "You have been helped!").await);
            },
            "record" => {
                let manager_lock = ctx.data.read().await.get::<VoiceManager>().cloned().expect("VoiceManager not stored in client");
                let mut manager = manager_lock.lock().await;
                if let Some(handler) = manager.get_mut(message.guild_id.unwrap()) {
                    handler.play(Box::new(EmptyAudioSource(5 * 1920)));
                    handler.listen(Some(Arc::new(Recorder::new())));
                    message.channel_id.say(&ctx.http, "Recording...");
                } else {
                    message.channel_id.say(&ctx.http, "I must be in a voice channel first");
                }
            },
            "stop" => {
                // let manager_lock = ctx.data.read().get::<VoiceManager>().cloned().expect("VoiceManager not stored in client");
                // let mut manager = manager_lock.lock();
                // if let Some(handler) = manager.get_mut(message.guild_id.unwrap()) {
                //     handler.listen(None);
                //     message.channel_id.say(&ctx.http, "Ended Recording");
                // } else {
                //     message.channel_id.say(&ctx.http, "I must be in a voice channel first");
                // }
            },
            "leave" => {
                let manager_lock = ctx.data.read().await.get::<VoiceManager>().cloned().expect("VoiceManager not stored in client");
                let mut manager = manager_lock.lock().await;
                if manager.get(message.guild_id.unwrap()).is_some() {
                    manager.remove(message.guild_id.unwrap());
                    check!(message.channel_id.say(&ctx.http, "Left").await);
                } else {
                    check!(message.channel_id.say(&ctx.http, "I must be in a voice channel first").await);
                }
            },
            "read" => {
                {
                    let manager_lock = ctx.data.read().await.get::<VoiceManager>().cloned().expect("VoiceManager not stored in client");
                    let manager = manager_lock.lock().await;
                    if manager.get(message.guild_id.unwrap()).is_none() {
                        check!(message.channel_id.say(&ctx.http, "I must be in a voice channel first").await);
                        return;
                    }
                }

                let mut data = ctx.data.write().await;
                let guild_settings_map = data.get_mut::<DankGuildMap>().expect("DankGuildMap not stored in client");
                let guild_settings = guild_settings_map.entry(message.guild_id.unwrap().0).or_insert_with(|| DankGuild::new(message.guild_id.unwrap().into()));

                if guild_settings.tts_channels.contains(&message.channel_id.0) {
                    guild_settings.tts_channels.remove(&message.channel_id.0);
                    check!(message.channel_id.say(&ctx.http, "Removed this channel from TTS").await);
                } else {
                    guild_settings.tts_channels.insert(message.channel_id.0);
                    check!(message.channel_id.say(&ctx.http, "Added this channel to TTS").await);
                }
            },
            "volume" => {
                if arguments.is_empty() {
                    check!(message.channel_id.say(&ctx.http, "You must provide the new volume level").await);
                    return;
                }

                let new_volume = match arguments[0].parse::<u16>() {
                    Ok(vol) => vol,
                    Err(_) => {
                        check!(message.channel_id.say(&ctx.http, "You must provide a valid number").await);
                        return;
                    }
                };

                {
                    let manager_lock = ctx.data.read().await.get::<VoiceManager>().cloned().expect("VoiceManager not stored in client");
                    let manager = manager_lock.lock().await;
                    if manager.get(message.guild_id.unwrap()).is_none() {
                        check!(message.channel_id.say(&ctx.http, "I must be in a voice channel first").await);
                        return;
                    }
                }

                let mut data = ctx.data.write().await;
                let guild_settings_map = data.get_mut::<DankGuildMap>().expect("DankGuildMap not stored in client");
                let guild_settings = guild_settings_map.entry(message.guild_id.unwrap().0).or_insert_with(|| DankGuild::new(message.guild_id.unwrap().into()));

                *guild_settings.volume.lock().await = new_volume;
                check!(message.channel_id.say(&ctx.http, format!("Successfully set TTS volume to {}", new_volume)).await);
            },
            _ => ()
        }
    }

    async fn reaction_add(&self, ctx: Context, reaction: Reaction) {
        if let ReactionType::Unicode(ref unicode) = reaction.emoji {
            if (unicode != "⭐") {
                return;
            }
        } else {
            return;
        }

        if reaction.guild_id.is_none() {
            return;
        }

        let stared_message = match ctx.cache.message(reaction.channel_id, reaction.message_id).await {
            Some(message) => message,
            None => match reaction.message(&ctx.http).await {
                Ok(message) => message,
                Err(_e) => return
            }
        };

        if let Some(channels) = ctx.cache.guild_field(reaction.guild_id.unwrap(), |guild| guild.channels.clone()).await {
            if let Some(channel) = channels.values().find(|&c| c.name == "starboard" || c.name == "cool-messages") {
                let submitter = match reaction.user(&ctx).await {
                    Ok(user) => Some(format!("{} ({})", user.tag(), user.mention())),
                    Err(_) => None
                };

                check!(channel.send_message(ctx, |m| {
                    m.embed(|e| {
                        e.author(|a| {
                            a.name(format!("{}#{:04}", stared_message.author.name, stared_message.author.discriminator));
                            a.icon_url(stared_message.author.face());
                            a
                        });
                        if !stared_message.content.is_empty() {
                            e.description(&stared_message.content);
                        }
                        e.footer(|f| {
                            f.text(stared_message.id.to_string());
                            f
                        });
                        e.timestamp(&stared_message.timestamp);
                        if !stared_message.attachments.is_empty() {
                            e.image(&stared_message.attachments[0].url);
                        }
                        if !stared_message.embeds.is_empty() {
                            if let Some(ref description) = stared_message.embeds[0].description {
                                e.field("Embed", format!("> {}", description), false);
                            }
                        }
                        e.field("Quick Link", format!("[Click Here]({})", format!("https://discord.com/channels/{}/{}/{}", reaction.guild_id.unwrap().0, stared_message.channel_id.0, stared_message.id.0)), true);
                        if let Some(submitter) = submitter {
                            e.field("Submitter", submitter, true);
                        }
                        e.color(16765448);

                        e
                    });

                    m
                }).await);
            }
        }
    }

    async fn voice_state_update(&self, ctx: Context, guild_id: Option<GuildId>, _old_state: Option<VoiceState>, new_state: VoiceState) {
        if guild_id.is_none() || new_state.channel_id.is_some() || new_state.user_id != ctx.cache.current_user_field(|user| user.id).await {
            return;
        }

        let mut data = ctx.data.write().await;
        let guild_settings_map = data.get_mut::<DankGuildMap>().expect("DankGuildMap not stored in client");
        let guild_settings = guild_settings_map.get_mut(&guild_id.unwrap().0);
        if guild_settings.is_some() {
            guild_settings.unwrap().end_tts();
        }
    }
}

#[tokio::main]
async fn main() {
    println!("Starting up...");

    let token: String = env::var("TOKEN").expect("You must provide a token");
    let mut client = Client::builder(&token)
        .event_handler(MainHandler)
        .await
        .expect("Ran into error while initializing client");

    {
        let mut data = client.data.write().await;
        data.insert::<DankGuildMap>(HashMap::default());
        data.insert::<VoiceManager>(Arc::clone(&client.voice_manager));
    }

    client.cache_and_http.cache.set_max_messages(7).await;

    if let Err(error) = client.start().await {
        println!("Ran into a fatal issue: {:?}", error);
    }
}

#[macro_export]
macro_rules! check {
    ($result:expr) => {
        if let Err(error) = $result {
            eprintln!("{:?}", error);
        }
    }
}
