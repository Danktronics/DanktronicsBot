use std::{
    env, sync::Arc, collections::HashMap
};
use serenity::{
    client::{bridge::voice::ClientVoiceManager, Client, Context, EventHandler},
    model::{
        channel::{Message, Reaction, ReactionType}, 
        gateway::Ready
    },
    prelude::*,
    voice
};
use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};

use model::{
    settings::GuildSettings,
    voice::{Recorder, EmptyAudioSource}
};

mod model;
mod helpers;

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

impl EventHandler for MainHandler {
    fn ready(&self, _: Context, ready: Ready) {
        println!("Connected as {}#{}", ready.user.name, ready.user.discriminator);
    }

    fn message(&self, ctx: Context, message: Message) {
        {
            let data = ctx.data.read();
            let guild_settings_map = data.get::<GuildSettingsMap>().expect("GuildSettingsMap not stored in client");
            let guild_settings = guild_settings_map.get(&message.guild_id.unwrap().0);
            if guild_settings.is_some() && guild_settings.unwrap().tts_channels.contains(&message.channel_id.0) {
                let manager_lock = ctx.data.read().get::<VoiceManager>().cloned().expect("VoiceManager not stored in client");
                let mut manager = manager_lock.lock();
                if let Some(handler) = manager.get_mut(message.guild_id.unwrap()) {
                    let possible_source = voice::ytdl(&format!("https://translate.google.com/translate_tts?ie=UTF-8&client=tw-ob&tl=en&q={}", utf8_percent_encode(message.content_safe(&ctx.cache).as_str(), NON_ALPHANUMERIC)));
                    if possible_source.is_ok() {
                        let locked_audio = handler.play_returning(possible_source.unwrap());
                        let mut audio = locked_audio.lock();
                        audio.volume(guild_settings.unwrap().volume.into());
                    } else {
                        println!("Error playing TTS: {:?}", possible_source.err());
                    }
                }
            }
        }

        if message.content.to_lowercase() == "cough" {
            let role_result = ctx.cache.read().guilds.get(&message.guild_id.unwrap()).unwrap().read().member(&ctx, message.author.id).unwrap().add_role(&ctx.http, 687873868106432661);
            if role_result.is_ok() {
                message.channel_id.say(&ctx.http, "This is the CDC. You are being quarantined as you are suspected to have a deadly virus pandemic. UwU");
            } else {
                message.channel_id.say(&ctx.http, "This is the CDC. You are on watch for coughing. We were unable to quarantine you");
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
                let channel_id = match ctx.cache.read().guilds.get(&message.guild_id.unwrap()).unwrap().read().voice_states.get(&message.author.id) {
                    Some(voice_state) => voice_state.channel_id.unwrap(),
                    None => {
                        message.channel_id.say(&ctx.http, "You must be in a voice channel");
                        return;
                    }
                };

                if !channel_id.to_channel_cached(&ctx.cache).unwrap().guild().unwrap().read().permissions_for_user(&ctx.cache, &ctx.cache.read().user.id).unwrap().connect() {
                    message.channel_id.say(&ctx.http, "I do not have permissions to join your channel");
                    return;
                }

                let manager_lock = ctx.data.read().get::<VoiceManager>().cloned().expect("VoiceManager not stored in client");
                let mut manager = manager_lock.lock();
                if manager.join(message.guild_id.unwrap(), channel_id).is_some() {
                    message.channel_id.say(&ctx.http, format!("Successfully joined **{}**", channel_id.name(&ctx.cache).unwrap()));
                } else {
                    message.channel_id.say(&ctx.http, "Failed to join your voice channel");
                }
            },
            "help" => {
                message.channel_id.say(&ctx.http, "You have been helped!");
            },
            "record" => {
                let manager_lock = ctx.data.read().get::<VoiceManager>().cloned().expect("VoiceManager not stored in client");
                let mut manager = manager_lock.lock();
                if let Some(handler) = manager.get_mut(message.guild_id.unwrap()) {
                    handler.play(Box::new(EmptyAudioSource(5 * 1920)));
                    handler.listen(Some(Box::new(Recorder::new())));
                    message.channel_id.say(&ctx.http, "Recording...");
                } else {
                    message.channel_id.say(&ctx.http, "I must be in a voice channel first");
                }
            },
            "stop" => {
                let manager_lock = ctx.data.read().get::<VoiceManager>().cloned().expect("VoiceManager not stored in client");
                let mut manager = manager_lock.lock();
                if let Some(handler) = manager.get_mut(message.guild_id.unwrap()) {
                    handler.listen(None);
                    message.channel_id.say(&ctx.http, "Ended Recording");
                } else {
                    message.channel_id.say(&ctx.http, "I must be in a voice channel first");
                }
            },
            "leave" => {
                let manager_lock = ctx.data.read().get::<VoiceManager>().cloned().expect("VoiceManager not stored in client");
                let mut manager = manager_lock.lock();
                if manager.get(message.guild_id.unwrap()).is_some() {
                    manager.remove(message.guild_id.unwrap());
                    message.channel_id.say(&ctx.http, "Left");
                } else {
                    message.channel_id.say(&ctx.http, "I must be in a voice channel first");
                }
            },
            "read" => {
                let manager_lock = ctx.data.read().get::<VoiceManager>().cloned().expect("VoiceManager not stored in client");
                let mut manager = manager_lock.lock();
                if manager.get(message.guild_id.unwrap()).is_none() {
                    message.channel_id.say(&ctx.http, "I must be in a voice channel first");
                    return;
                }

                let mut data = ctx.data.write();
                let guild_settings_map = data.get_mut::<GuildSettingsMap>().expect("GuildSettingsMap not stored in client");
                if !guild_settings_map.contains_key(&message.guild_id.unwrap().0) {
                    guild_settings_map.insert(message.guild_id.unwrap().0, GuildSettings::new());
                }

                let guild_settings = guild_settings_map.get_mut(&message.guild_id.unwrap().0).unwrap();
                if guild_settings.tts_channels.contains(&message.channel_id.0) {
                    guild_settings.tts_channels.remove(&message.channel_id.0);
                    message.channel_id.say(&ctx.http, "Removed this channel from TTS");
                } else {
                    guild_settings.tts_channels.insert(message.channel_id.0);
                    message.channel_id.say(&ctx.http, "Added this channel to TTS");
                }
            },
            "ttsvolume" => {
                if arguments.len() == 0 {
                    message.channel_id.say(&ctx.http, "You must provide the new volume level");
                    return;
                }

                let new_volume = match arguments[0].parse::<u16>() {
                    Ok(vol) => vol,
                    Err(error) => {
                        message.channel_id.say(&ctx.http, "You must provide a valid number");
                        return;
                    }
                };

                let manager_lock = ctx.data.read().get::<VoiceManager>().cloned().expect("VoiceManager not stored in client");
                let mut manager = manager_lock.lock();
                if manager.get(message.guild_id.unwrap()).is_none() {
                    message.channel_id.say(&ctx.http, "I must be in a voice channel first");
                    return;
                }

                let mut data = ctx.data.write();
                let guild_settings_map = data.get_mut::<GuildSettingsMap>().expect("GuildSettingsMap not stored in client");
                if !guild_settings_map.contains_key(&message.guild_id.unwrap().0) {
                    guild_settings_map.insert(message.guild_id.unwrap().0, GuildSettings::new());
                }

                let guild_settings = guild_settings_map.get_mut(&message.guild_id.unwrap().0).unwrap();
                guild_settings.volume = new_volume;
                message.channel_id.say(&ctx.http, format!("Successfully set TTS volume to {}", new_volume));
            },
            _ => ()
        }
    }

    fn reaction_add(&self, ctx: Context, reaction: Reaction) {
        /*match reaction.emoji {
            ReactionType::Custom{..} => return,
            ReactionType::Unicode(unicode) => {
                if unicode == "⭐" {
                   let possible_message = reaction.message(&ctx.http);
                }
            },
            _ => return
        }*/
    }
}

fn main() {
    println!("Starting up...");

    let token: String = env::var("TOKEN").expect("You must provide a token");
    let mut client = Client::new(&token, MainHandler).expect("Ran into error while initializing client");

    {
        let mut data = client.data.write();
        data.insert::<GuildSettingsMap>(HashMap::default());
        data.insert::<VoiceManager>(Arc::clone(&client.voice_manager));
    }

    if let Err(error) = client.start() {
        println!("Ran into a fatal issue: {:?}", error);
    }
}
