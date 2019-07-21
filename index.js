const Eris = require("eris");
const fs = require("fs");
const lamejs = require("lamejs");
const streamBuffers = require("stream-buffers");
const settings = require("./settings.json");

const client = new Eris(settings.token);
const EmbedBuilder = require("./structures/EmbedBuilder");

const danktronics = "293935518801199106";
const starboard = "502655582247845898";
const starEmoji = "⭐";
const prefix = "d.";

let messageRate = new Map();

let voiceEncoder = new lamejs.Mp3Encoder(1, 44100, 96);
let recordingData = new Eris.Collection();

let playDict = [];

function linkMessage(message) {
    return `https://discordapp.com/channels/${message.channel.guild.id}/${message.channel.id}/${message.id}`
}

function getMe(guild) {
    return guild.members.get(client.user.id);
}

function record(voiceConnection) {
    voiceConnection.play(new streamBuffers.ReadableStreamBuffer({frequency: 10, chunkSize: 2048}));
    let voiceDataStream = voiceConnection.receive("pcm");
    recordingData.set(voiceConnection.channelID, {channelID: voiceConnection.channelID, dataStream: voiceDataStream, data: []});
    voiceDataStream.on("data", (data, userID, timestamp, sequence) => {
        recordingData.get(voiceConnection.channelID).data.push(data);
    });
    voiceDataStream.on("error", console.error);
}

function tts(message) {
    if (getMe(message.channel.guild).voiceState == null) {
        playDict.splice(playDict.indexOf(message.channel));
        return;
    }
    voiceConnection = client.voiceConnections.get(message.channel.guild.id);
    voiceConnection.playStream("https://translate.google.com/translate_tts?ie=UTF-8&client=tw-ob&tl=en&q="+encodeURIComponent(message.content));
}

function saveRecording(voiceReceiver) {
    return new Promise((resolve, reject) => {
        const Lame = require("node-lame").Lame;
 
        const encoder = new Lame({
            "output": `./${Date.now()}.mp3`,
            "bitrate": 96
        }).setBuffer(Buffer.concat(voiceReceiver.data));
        
        encoder.encode()
        .then(resolve)
        .catch(reason => {
            console.error(reason);
            reject(reason);
        });
        /*let polishedData = [];
        let mp3Data = voiceEncoder.encodeBuffer(Buffer.concat(voiceReceiver.data));
    
        /*polishedData.push(mp3Data);
        mp3Data = voiceEncoder.flush();
        polishedData.push(mp3Data);
    
        fs.writeFile("./test.mp3", mp3Data, error => {
            if (error) reject();
            resolve();
        });*/
    });
}

client.on("ready", () => {
    console.log("Ready. I guess...");
    client.editStatus({name: "idk", type: 3});
});

// COMMANDS
client.on("messageCreate", message => {
    //messageRate.set(message.channel.id, messageRate.get(message.channel.id) != null ? messageRate.get(message.channel.id) + 1 : 1);

    if (!message.content.startsWith(prefix)) return;
    if (message.channel in playDict) tts(message);
    
    let strippedMessage = message.content.slice(prefix.length);
    let args = strippedMessage.split(" ");
    let cmd = args[0];

    if (cmd === "join") {
        if (message.member.voiceState == null) return message.channel.createMessage("You are not in a voice channel.");
        let voiceChannel = message.channel.guild.channels.get(message.member.voiceState.channelID);
        voiceChannel.join()
        .then(() => message.channel.createMessage(`Successfully joined **${voiceChannel.name}**`))
        .catch(() => message.channel.createMessage("Failed to join voice channel"));
    }
    if (cmd === "record") {
        let voiceState = getMe(message.channel.guild).voiceState;
        if (voiceState == null) return message.channel.createMessage("I am not in a voice channel.");
        record(client.voiceConnections.get(message.channel.guild.id));
        message.channel.createMessage("Recording...");
    }
    if (cmd === "stop") {
        let voiceState = getMe(message.channel.guild).voiceState;
        if (voiceState == null) return message.channel.createMessage("I am not in a voice channel.");
        saveRecording(recordingData.find(d => client.channelGuildMap[d.channelID] === message.channel.guild.id))
        .then(() => message.channel.createMessage("Ended recording"))
        .catch(() => message.channel.createMessage("Error occurred"));
    }
    if (cmd === "read") {
        let voiceState = getMe(message.channel.guild).voiceState;
        if (voiceState == null) return message.channel.createMessage("I am not in a voice channel.");
        playDict.push(channel);
        message.channel.createMessage("Reading from this channel.");
    }
    if (cmd === "rate") {
        message.channel.createMessage("**sithsiri#3253** has sent the most messages on the server. Last check resulted in 39,997 messages.");
    }
});

// STARBOARD

/*client.on("channelPinUpdate", async (channel, timestamp, oldTimestamp) => {
    console.log(timestamp + " | " + oldTimestamp)
    if (oldTimestamp == null || channel.guild.id !== danktronics || channel.id === starboard) return;
    
    let pins = await channel.getPins();
    if (pins == null || pins.length === 0) return;
    let latestMessage = pins[0];

    let embed = new EmbedBuilder()
    .setAuthor(`${latestMessage.author.username}#${latestMessage.author.discriminator}`, null, latestMessage.author.avatarURL)
    .setDescription(latestMessage.content.length > 0 ? latestMessage.content : "Unknown")
    .addField("Quick Link", `[Click Here](${linkMessage(latestMessage)})`)
    .setFooter(latestMessage.id)
    .setTimestamp(new Date());

    if (latestMessage.attachments.length > 0) embed.setImage(latestMessage.attachments[0].url);

    client.guilds.get(danktronics).channels.get(starboard).createMessage({content: `<#${channel.id}>`, embed: embed.render()});
});*/

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

    guild.channels.get(starboard).createMessage({content: `<#${channel.id}>`, embed: embed.render()});
});

client.on("error", console.error);
process.on("unhandledRejection", console.error);

client.connect();
