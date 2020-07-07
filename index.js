const Eris = require("eris");
const fs = require("fs");
const lamejs = require("lamejs");
const settings = require("./settings.json");

const client = new Eris(settings.token);
const EmbedBuilder = require("./structures/EmbedBuilder");
const DankGuild = require("./structures/DankGuild");

const prefix = "d.";
const starEmoji = "⭐";
const starboardChannelName = "starboard";

let dankGuilds = new Eris.Collection();

function linkMessage(message) {
    return `https://discordapp.com/channels/${message.channel.guild.id}/${message.channel.id}/${message.id}`;
}

function getMe(guild) {
    return guild.members.get(client.user.id);
}

client.on("ready", () => {
    console.log("Ready. I guess...");
    client.editStatus({name: "the people here", type: 3});
});

// COMMANDS
client.on("messageCreate", message => {
    let dankGuild = dankGuilds.get(message.channel.guild.id);
    if (dankGuild == null) {
        dankGuild = new DankGuild(message.channel.guild.id, client);
        dankGuilds.set(message.channel.guild.id, dankGuild);
    }

    if (dankGuild.ttsChannels.includes(message.channel.id)) {
        if (getMe(message.channel.guild).voiceState == null) return;
    
        dankGuild.ttsQueue.enqueue(message.cleanContent);
    }

    if (!message.content.startsWith(prefix)) return;
    
    let strippedMessage = message.content.slice(prefix.length);
    let args = strippedMessage.split(" ");
    let cmd = args[0];

    if (cmd === "join") {
        if (message.member.voiceState == null) return message.channel.createMessage("You are not in a voice channel.");
        let voiceChannel = message.channel.guild.channels.get(message.member.voiceState.channelID);
        voiceChannel.join()
        .then(voiceConnection => {
            dankGuild.setupVoiceConnection(voiceConnection);
            message.channel.createMessage(`Successfully joined **${voiceChannel.name}**`)
        })
        .catch(() => message.channel.createMessage("Failed to join voice channel"));
    }
    if (cmd === "help") {
        message.channel.createMessage("You have been helped!");
    }
    if (cmd === "record") {
        let voiceState = getMe(message.channel.guild).voiceState;
        if (voiceState == null) return message.channel.createMessage("I am not in a voice channel.");

        dankGuild.record();
        message.channel.createMessage("Recording...");
    }
    if (cmd === "stop") {
        let voiceState = getMe(message.channel.guild).voiceState;
        if (voiceState == null) return message.channel.createMessage("I am not in a voice channel.");

        dankGuild.saveRecording()
        .then(() => message.channel.createMessage("Ended recording"))
        .catch(() => message.channel.createMessage("Error occurred"));
    }
    if (cmd === "read") {
        let voiceState = getMe(message.channel.guild).voiceState;
        if (voiceState == null) return message.channel.createMessage("I am not in a voice channel.");
        if (dankGuild.ttsChannels.includes(message.channel.id)) {
            dankGuild.ttsChannels.splice(dankGuild.ttsChannels.indexOf(message.channel.id));
            message.channel.createMessage("This channel is no longer being read.");
            return;
        }

        dankGuild.ttsChannels.push(message.channel.id);
        message.channel.createMessage("Reading from this channel.");
    }
    if (cmd === "rate") {
        message.channel.createMessage("**sithsiri#3253** has sent the most messages on the server. Last check resulted in 100,618 messages.");
    }
    if (cmd === "ttsvolume") {
        let newVolume = parseInt(args[1]);
        if (isNaN(newVolume) && newVolume >= 1 && newVolume <= 9) return message.channel.createMessage("Please provide a valid number");

        dankGuild.ttsVolume = newVolume;
        message.channel.createMessage("Successfully set tts volume to " + newVolume);
    }
});

/*client.on("voiceChannelLeave", (member, oldChannel) => {
    if (member.id === client.user.id && oldChannel != null) {
        let dankGuild = dankGuilds.get(member.guild.id);
        if (dankGuild == null) return;
        dankGuild.resetVoice();
    }
});*/

// STARBOARD

client.on("messageReactionAdd", async (message, emoji, userID) => {
    if (emoji.name !== starEmoji) return;
    let guild = client.guilds.get(client.channelGuildMap[message.channel.id]);
    let channel = guild.channels.get(message.channel.id);
    
    let latestMessage = await channel.getMessage(message.id);
    if (latestMessage.reactions.size > 1) return;

    let embed = new EmbedBuilder()
    .setAuthor(`${latestMessage.author.username}#${latestMessage.author.discriminator}`, null, latestMessage.author.avatarURL)
    .setDescription(latestMessage.content.length > 0 ? latestMessage.content : "Unknown")
    .addField("Quick Link", `[Click Here](${linkMessage(latestMessage)})`)
    .setFooter(latestMessage.id)
    .setTimestamp(new Date());

    if (latestMessage.attachments.length > 0) embed.setImage(latestMessage.attachments[0].url);
    if (latestMessage.embeds.length > 0) {
        if (latestMessage.embeds[0].description != null) embed.setDescription(`> ${latestMessage.embeds[0].description}`);
    }

    let starboardChannel = guild.channels.find(channel => channel.name === starboardChannelName || channel.name === "cool-messages");
    if (starboardChannel == null) {
        message.channel.createMessage("This server does not have a starboard channel");
        return;
    }

    guild.channels.get(starboardChannel.id).createMessage({content: channel.mention, embed: embed.render()});
});

client.on("error", console.error);
process.on("unhandledRejection", console.error);

client.connect();
