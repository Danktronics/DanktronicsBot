use std::collections::HashSet;
use std::thread;
use std::sync::mpsc::{channel, Sender, Receiver};
use std::sync::Arc;
use std::time::Duration;
use serenity::client::bridge::voice::ClientVoiceManager;
use serenity::prelude::*;
use anyhow::{Error, bail};
use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use crate::create_tts_source;

enum TTSMessage {
    NewMessage(String),
    EndTTS,
}

pub struct GuildSettings {
    pub id: u64,
    pub volume: Arc<Mutex<u16>>,
    pub tts_channels: HashSet<u64>,
    tts_sender: Option<Mutex<Sender<TTSMessage>>>,
}

impl GuildSettings {
    pub fn new(id: u64) -> GuildSettings {
        GuildSettings {
            id,
            volume: Arc::new(Mutex::new(1)),
            tts_channels: HashSet::new(),
            tts_sender: None
        }
    }

    pub fn initialize_tts(&mut self, voice_manager: Arc<Mutex<ClientVoiceManager>>) {
        if self.tts_sender.is_some() {
            return;
        }

        let (sender, receiver) = channel();
        self.tts_sender = Some(Mutex::new(sender));
        let guild_id = self.id;
        let volume_lock = Arc::clone(&self.volume);

        thread::spawn(move || {
            loop {
                match receiver.recv() {
                    Ok(message) => match message {
                        TTSMessage::NewMessage(tts_message) => {
                            let mut manager = voice_manager.lock();

                            if let Some(handler) = manager.get_mut(guild_id) {
                                let possible_source = create_tts_source(&format!("https://translate.google.com/translate_tts?ie=UTF-8&client=tw-ob&tl=en&q={}", utf8_percent_encode(&tts_message, NON_ALPHANUMERIC)));
                                if possible_source.is_ok() {
                                    let locked_audio = handler.play_returning(Box::new(possible_source.unwrap()));
                                    {
                                        let mut audio = locked_audio.lock();
                                        let volume = volume_lock.lock();
                                        if *volume != 1 {
                                            audio.volume((*volume).into());
                                        }
                                    }

                                    while !locked_audio.lock().finished {
                                        thread::sleep(Duration::from_micros(500000));
                                    }
                                } else {
                                    println!("Error playing TTS: {:?}", possible_source.err());
                                }
                            }
                        },
                        TTSMessage::EndTTS => break
                    },
                    Err(error) => {
                        break;
                    }
                }
            }
        });
    }

    pub fn end_tts(&mut self) {
        if self.tts_sender.is_none() {
            return;
        }

        // self.tts_sender.as_ref().unwrap().lock().send(TTSMessage::EndTTS {});
        self.tts_sender = None;
        self.tts_channels.clear();
    }

    pub fn say_message(&self, message: String) -> Result<(), Error> {
        if self.tts_sender.is_none() {
            bail!("TTS sender is not present");
        }

        Ok(self.tts_sender.as_ref().unwrap().lock().send(TTSMessage::NewMessage(message))?)
    }
}