use std::{
    env, sync::Arc, collections::HashMap
};
use serenity::{
    client::{bridge::voice::ClientVoiceManager, Client, Context, EventHandler},
    model::{channel::Message, gateway::Ready},
    prelude::*,
    voice::AudioReceiver
};

use model::{
    settings::GuildSettings
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
        // TODO: Queue TTS message

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
        let arguments: Vec<&str> = raw_command_message.split(" ").collect();
        let command: &str = arguments[0];

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

            },
            "stop" => {

            },
            "read" => {

            },
            "ttsvolume" => {

            },
            _ => ()
        }
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
