const streamBuffers = require("stream-buffers");
const Lame = require("node-lame").Lame;

const Queue = require("./Queue");

class DankGuild {
    constructor(id, client) {
        this.id = id;
        this.client = client;
        this.recording = false;
        this.rawRecordingData = [];
        this.ttsChannels = [];
        this.ttsQueue = new Queue(this.playMessageTTS.bind(this));
        this.ttsVolume = 1;
    }

    getVoiceConnection() {
        return this.client.voiceConnections.find(voiceConnection => this.client.channelGuildMap[voiceConnection.channelID] === this.id);
    }

    playMessageTTS(content) {
        return new Promise((resolve, reject) => {
            let voiceConnection = this.getVoiceConnection();
            if (voiceConnection == null) {
                resolve();
                return;
            }

            if (voiceConnection.volume !== this.ttsVolume) voiceConnection.setVolume(this.ttsVolume);
            voiceConnection.play("https://translate.google.com/translate_tts?ie=UTF-8&client=tw-ob&tl=en&q=" + encodeURIComponent(content), {inlineVolume: true});
            let streamEndHandler = () => {
                voiceConnection.removeListener("end", streamEndHandler);
                resolve();
            };
            voiceConnection.on("end", streamEndHandler);
        });
    }

    record() {
        let voiceConnection = this.getVoiceConnection();
        if (voiceConnection == null) return;
        
        voiceConnection.play(new streamBuffers.ReadableStreamBuffer({frequency: 10, chunkSize: 2048}));
        this.voiceDataStream = voiceConnection.receive("pcm");
        recording = true;
        this._voiceDataStreamListener = this.voiceDataStream.on("data", (data, userID, timestamp, sequence) => {
            this.rawRecordingData.push(data);
        });
        this._voiceDataStreamErrorListener = this.voiceDataStream.on("error", error => {
            console.error(error);
            this.saveRecording();
        });
    }

    saveRecording() {
        recording = false;
        this.voiceDataStream.removeListener("data", this._voiceDataStreamListener);
        this.voiceDataStream.removeListener("error", this._voiceDataStreamErrorListener);

        return new Promise((resolve, reject) => {
            if (this.rawRecordingData.length === 0) resolve();
            
            const encoder = new Lame({
                "output": `./${Date.now()}.mp3`,
                "bitrate": 64
            }).setBuffer(Buffer.concat(this.rawRecordingData));
            
            encoder.encode()
            .then(resolve)
            .catch(reason => {
                console.error(reason);
                reject(reason);
            });
            /*
            let voiceEncoder = new lamejs.Mp3Encoder(1, 44100, 96);
            let polishedData = [];
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

    async resetVoice() {
        if (recording) await this.saveRecording().catch(error => console.error(error));
        this.rawRecordingData = [];
        this.ttsQueue.clear();
        this.ttsVolume = 1;
    }
}

module.exports = DankGuild;