use std::str::FromStr;
use lazy_static::lazy_static;
use regex::Regex;
use serenity::model::id::ChannelId;
use serenity::model::channel::{Message, Channel};
use serenity::cache::Cache;
use serenity::prelude::*;

lazy_static! {
    static ref URL_REGEX: Regex = Regex::new(r"(https?://)?(www\.)?([-a-zA-Z0-9@:%._\+~#=]{1,256}\.[a-zA-Z0-9()]{1,6})\b[-a-zA-Z0-9()@:%_\+.~#?&//=]*").unwrap();
    static ref EMOTE_REGEX: Regex = Regex::new(r"<a?(:\w+:)[0-9]+>").unwrap();
    static ref CHANNEL_REGEX: Regex = Regex::new(r"<#([0-9]+)>").unwrap();
}

pub async fn clean_message_content(message: &Message, cache: &Cache) -> String {
    let guild = message.guild(&cache).unwrap();
    let mut temp_msg = EMOTE_REGEX.replace_all(&URL_REGEX.replace_all(&message.content, "${3} link"), "${1} emote").into_owned();

    for user in &message.mentions {
        let mut user_mention = user.mention().to_string();
        if !temp_msg.contains(&user_mention) {
            user_mention.insert(2, '!');
        }

        if guild.members.contains_key(&user.id) && guild.members.get(&user.id).unwrap().nick.is_some() {
            temp_msg = temp_msg.replace(&user_mention, &format!("@{}", guild.members.get(&user.id).unwrap().nick.as_ref().unwrap()));
        } else {
            temp_msg = temp_msg.replace(&user_mention, &format!("@{}", user.name));
        }
    }

    for role_id in &message.mention_roles {
        let role_mention = role_id.mention().to_string();

        if let Some(role) = role_id.to_role_cached(&cache) {
            temp_msg = temp_msg.replace(&role_mention, &format!("@{}", role.name));
        } else {
            temp_msg = temp_msg.replace(&role_mention, "@deleted-role");
        }
    }

    let mut clean_content = temp_msg.clone();
    for channel in CHANNEL_REGEX.captures_iter(&temp_msg) {
        if guild.channels.contains_key(&ChannelId::from_str(&channel[1]).unwrap()) {
            if let Channel::Guild(guild_area) = guild.channels.get(&ChannelId::from_str(&channel[1]).unwrap()).unwrap() {
                clean_content = clean_content.replace(&channel[0], &format!("{} channel", &guild_area.name));
            }
        } else {
            clean_content = clean_content.replace(&channel[0], "deleted channel");
        }
    }

    for channel in &message.mention_channels {
        clean_content = clean_content.replace(&format!("<#{}>", channel.id), &channel.name);
    }

    clean_content = clean_content.replace("owo", "ohwo");
    clean_content = clean_content.replace("uwu", "oowoo");

    clean_content
}
