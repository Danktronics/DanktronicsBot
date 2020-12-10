use std::{
    env, sync::Arc, collections::HashMap, thread,
};
use serenity::{
    client::{bridge::voice::ClientVoiceManager, Client, Context, EventHandler},
    model::{channel::Message, gateway::Ready, id::GuildId, voice::VoiceState},
    prelude::*,
    voice,
    async_trait
};

use model::{
    settings::GuildSettings,
    voice::{Recorder, create_tts_source}
};

mod model;

struct VoiceManager;

impl TypeMapKey for VoiceManager {
    type Value = Arc<Mutex<ClientVoiceManager>>;
}

struct GuildSettingsMap;

impl TypeMapKey for GuildSettingsMap {
    type Value = HashMap<u64, GuildSettings>;
}

struct MainHandler;

static PREFIX: &str = "d!";

#[async_trait]
impl EventHandler for MainHandler {
    async fn ready(&self, _: Context, ready: Ready) {
        println!("Connected as {}#{}", ready.user.name, ready.user.discriminator);
    }

    async fn message(&self, ctx: Context, message: Message) {
        {
            let data = ctx.data.read().await;
            let guild_settings_map = data.get::<GuildSettingsMap>().expect("GuildSettingsMap not stored in client");
            let guild_settings = guild_settings_map.get(&message.guild_id.unwrap().0);
            if guild_settings.is_some() && guild_settings.unwrap().tts_channels.contains(&message.channel_id.0) {
                guild_settings.unwrap().say_message(message.content_safe(&ctx.cache).await).await;
            }
        }

        if message.content.to_lowercase() == "cough" {
            let role_result = ctx.cache.guild(&message.guild_id.unwrap()).await.unwrap().member(&ctx, message.author.id).await.unwrap().add_role(&ctx.http, 687873868106432661).await;
            if role_result.is_ok() {
                message.channel_id.say(&ctx.http, "This is the CDC. You are being quarantined as you are suspected to have a deadly virus pandemic. UwU").await;
            } else {
                message.channel_id.say(&ctx.http, "This is the CDC. You are on watch for coughing. We were unable to quarantine you").await;
            }
            return;
        }

        if !message.content.starts_with(PREFIX) {
            return;
        }

        let raw_command_message: String = message.content.chars().skip(PREFIX.len()).collect();
        let mut arguments: Vec<&str> = raw_command_message.split(" ").collect();
        let command: &str = arguments[0];
        arguments.remove(0);

        match command {
            "join" => {
                let channel_id = match ctx.cache.guild(&message.guild_id.unwrap()).await.unwrap().voice_states.get(&message.author.id) {
                    Some(voice_state) => voice_state.channel_id.unwrap(),
                    None => {
                        message.channel_id.say(&ctx.http, "You must be in a voice channel").await;
                        return;
                    }
                };

                if !channel_id.to_channel_cached(&ctx.cache).await.unwrap().guild().unwrap().permissions_for_user(&ctx.cache, &ctx.cache.current_user_field(|user| user.id).await).await.unwrap().connect() {
                    message.channel_id.say(&ctx.http, "I do not have permissions to join your channel").await;
                    return;
                }

                let manager_lock = ctx.data.read().await.get::<VoiceManager>().cloned().expect("VoiceManager not stored in client");
                let mut manager = manager_lock.lock().await;
                {
                    let handler = manager.get(message.guild_id.unwrap());
                    if handler.is_some() && handler.unwrap().channel_id.is_some() && handler.unwrap().channel_id.unwrap() == channel_id {
                        message.channel_id.say(&ctx.http, "I am already in this channel!").await;
                        return;
                    }
                }

                if manager.join(message.guild_id.unwrap(), channel_id).is_some() {
                    let mut data = ctx.data.write().await;
                    let guild_settings_map = data.get_mut::<GuildSettingsMap>().expect("GuildSettingsMap not stored in client");
                    if !guild_settings_map.contains_key(&message.guild_id.unwrap().0) {
                        guild_settings_map.insert(message.guild_id.unwrap().0, GuildSettings::new(message.guild_id.unwrap().into()));
                    }

                    let guild_settings = guild_settings_map.get_mut(&message.guild_id.unwrap().0).unwrap();
                    guild_settings.initialize_tts(manager_lock.clone());
                    message.channel_id.say(&ctx.http, format!("Successfully joined **{}**", channel_id.name(&ctx.cache).await.unwrap())).await;
                } else {
                    message.channel_id.say(&ctx.http, "Failed to join your voice channel").await;
                }
            },
            "help" => {
                message.channel_id.say(&ctx.http, "You have been helped!").await;
            },
            "record" => {
                // let manager_lock = ctx.data.read().get::<VoiceManager>().cloned().expect("VoiceManager not stored in client");
                // let mut manager = manager_lock.lock();
                // if let Some(handler) = manager.get_mut(message.guild_id.unwrap()) {
                //     handler.listen(Some(Box::new(Recorder::new())));
                //     message.channel_id.say(&ctx.http, "Recording...");
                // } else {
                //     message.channel_id.say(&ctx.http, "I must be in a voice channel first");
                // }
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
                    message.channel_id.say(&ctx.http, "Left").await;
                } else {
                    message.channel_id.say(&ctx.http, "I must be in a voice channel first").await;
                }
            },
            "read" => {
                let manager_lock = ctx.data.read().await.get::<VoiceManager>().cloned().expect("VoiceManager not stored in client");
                let mut manager = manager_lock.lock().await;
                if manager.get(message.guild_id.unwrap()).is_none() {
                    message.channel_id.say(&ctx.http, "I must be in a voice channel first").await;
                    return;
                }

                let mut data = ctx.data.write().await;
                let guild_settings_map = data.get_mut::<GuildSettingsMap>().expect("GuildSettingsMap not stored in client");
                if !guild_settings_map.contains_key(&message.guild_id.unwrap().0) {
                    guild_settings_map.insert(message.guild_id.unwrap().0, GuildSettings::new(message.guild_id.unwrap().into()));
                }

                let guild_settings = guild_settings_map.get_mut(&message.guild_id.unwrap().0).unwrap();
                if guild_settings.tts_channels.contains(&message.channel_id.0) {
                    guild_settings.tts_channels.remove(&message.channel_id.0);
                    message.channel_id.say(&ctx.http, "Removed this channel from TTS").await;
                } else {
                    guild_settings.tts_channels.insert(message.channel_id.0);
                    message.channel_id.say(&ctx.http, "Added this channel to TTS").await;
                }
            },
            "volume" => {
                if arguments.len() == 0 {
                    message.channel_id.say(&ctx.http, "You must provide the new volume level").await;
                    return;
                }

                let new_volume = match arguments[0].parse::<u16>() {
                    Ok(vol) => vol,
                    Err(error) => {
                        message.channel_id.say(&ctx.http, "You must provide a valid number").await;
                        return;
                    }
                };

                let manager_lock = ctx.data.read().await.get::<VoiceManager>().cloned().expect("VoiceManager not stored in client");
                let mut manager = manager_lock.lock().await;
                if manager.get(message.guild_id.unwrap()).is_none() {
                    message.channel_id.say(&ctx.http, "I must be in a voice channel first").await;
                    return;
                }

                let mut data = ctx.data.write().await;
                let guild_settings_map = data.get_mut::<GuildSettingsMap>().expect("GuildSettingsMap not stored in client");
                if !guild_settings_map.contains_key(&message.guild_id.unwrap().0) {
                    guild_settings_map.insert(message.guild_id.unwrap().0, GuildSettings::new(message.guild_id.unwrap().into()));
                }

                let guild_settings = guild_settings_map.get_mut(&message.guild_id.unwrap().0).unwrap();
                *guild_settings.volume.lock().await = new_volume;
                message.channel_id.say(&ctx.http, format!("Successfully set TTS volume to {}", new_volume)).await;
            },
            _ => ()
        }
    }

    async fn voice_state_update(&self, ctx: Context, guild_id: Option<GuildId>, old_state: Option<VoiceState>, new_state: VoiceState) {
        if guild_id.is_none() || new_state.channel_id.is_some() || new_state.user_id != ctx.cache.current_user_field(|user| user.id).await {
            return;
        }

        let mut data = ctx.data.write().await;
        let guild_settings_map = data.get_mut::<GuildSettingsMap>().expect("GuildSettingsMap not stored in client");
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
        data.insert::<GuildSettingsMap>(HashMap::default());
        data.insert::<VoiceManager>(Arc::clone(&client.voice_manager));
    }

    if let Err(error) = client.start().await {
        println!("Ran into a fatal issue: {:?}", error);
    }
}
