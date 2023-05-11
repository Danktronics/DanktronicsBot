use std::collections::HashSet;
use tokio::sync::mpsc::{unbounded_channel, UnboundedSender};
use tokio::sync::oneshot::{channel, Sender};
use songbird::{Call, Event, TrackEvent, tracks::PlayMode, EventHandler, EventContext}; use std::pin::Pin; use futures::{task::Poll, task::Context as FuturesContext, task::Waker};
use futures::Future;
use std::sync::Arc;
use std::time::Duration;
// use serenity::client::bridge::voice::ClientVoiceManager;
use serenity::{prelude::*, async_trait};
use anyhow::{Error, bail};
use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use crate::create_tts_source;

#[derive(Debug)]
enum TTSMessage {
    NewMessage(String)
}

pub struct DankGuild {
    pub id: u64,
    pub volume: Arc<Mutex<u16>>,
    pub tts_channels: HashSet<u64>,
    tts_sender: Option<Mutex<UnboundedSender<TTSMessage>>>,
}


pub struct Handler {
    sender: Mutex<Option<Sender<()>>>
    // pub ready: Mutex<bool>,
    // waker: Mutex<Option<Waker>>
}
#[async_trait]
impl EventHandler for Handler {
    async fn act(&self, context: &EventContext<'_>) -> Option<Event> {
        if let EventContext::Track(track_array) = context {
            if track_array[0].0.playing == PlayMode::End {
                let send_end = self.sender.lock().await.take();
                if send_end.is_some() {
                    send_end.unwrap().send(());
                }
                // let mut ready = self.ready.lock().await;
                // *ready = true;
                // let waker = self.waker.lock().await;
                // if waker.is_some() {
                //     waker.as_ref().unwrap().clone().wake();
                // }
            }
        }

        None
    }
}
// impl Future for Handler {
//     type Output = ();
//     fn poll(self: Pin<&mut Self>, cx: &mut FuturesContext<'_>)->Poll<Self::Output> {
//         if *self.ready.blocking_lock() {
//             return Poll::Ready(());
//         }

//         let mut waker = self.waker.blocking_lock();
//         *waker = Some(cx.waker().clone());
//         Poll::Pending
//     }
// }

impl DankGuild {
    pub fn new(id: u64) -> DankGuild {
        DankGuild {
            id,
            volume: Arc::new(Mutex::new(1)),
            tts_channels: HashSet::new(),
            tts_sender: None
        }
    }

    pub fn initialize_tts(&mut self, voice_manager: Arc<Mutex<Call>>) {
        if self.tts_sender.is_some() {
            return;
        }

        let (sender, mut receiver) = unbounded_channel();
        self.tts_sender = Some(Mutex::new(sender));
        let guild_id = self.id;
        let volume_lock = Arc::clone(&self.volume);

        tokio::spawn(async move {
            while let Some(message) = receiver.recv().await {
                match message {
                    TTSMessage::NewMessage(tts_message) => {
                        let locked_audio;
                        
                        {
                            let mut manager = voice_manager.lock().await;
                            let possible_source = create_tts_source(&format!("https://translate.google.com/translate_tts?ie=UTF-8&client=tw-ob&tl=en&q={}", utf8_percent_encode(&tts_message, NON_ALPHANUMERIC)));
                                if let Ok(source) = possible_source {
                                    locked_audio = manager.play_source(source);
                                } else {
                                    println!("Error playing TTS: {:?}", possible_source.err());
                                    continue;
                                }
                        }

                        {
                            let volume = volume_lock.lock().await;
                            if *volume != 1 {
                                locked_audio.set_volume((*volume).into());
                            }
                        }
                        let (sender, receiver) = channel();
                        let handler = Handler {sender: Mutex::new(Some(sender))};
                        // let handler = Arc::new(Handler {ready:Mutex::new(false), waker:Mutex::new(None)});
                        locked_audio.add_event(Event::Track(TrackEvent::End), handler);
                        receiver.await;
                        
                    }
                }
            }
        });
    }

    pub fn end_tts(&mut self) {
        if self.tts_sender.is_none() {
            return;
        }

        self.tts_sender = None;
        self.tts_channels.clear();
    }

    pub async fn say_message(&self, message: String) -> Result<(), Error> {
        if self.tts_sender.is_none() {
            bail!("TTS sender is not present");
        }

        Ok(self.tts_sender.as_ref().unwrap().lock().await.send(TTSMessage::NewMessage(message))?)
    }
}