use std::fs::OpenOptions;
// use std::io::{Read, Seek, SeekFrom};
use std::io::Write;
use std::io::BufWriter;
use std::fs::File;
use std::sync::Mutex;
use tokio::{
    // io::Read
    sync::mpsc::{unbounded_channel, UnboundedSender}
};
// use serenity::voice::{AudioReceiver, AudioSource, AudioType};
use byteorder::{WriteBytesExt, LittleEndian};
use hound::{WavWriter, WavSpec, SampleFormat::Int};
use std::io::{Read, Seek, ErrorKind, Result, SeekFrom};
use std::process::{Child, Command, Stdio};
use serenity::async_trait;
use std::time::{SystemTime, UNIX_EPOCH};
use songbird::{
    EventHandler as VoiceEventHandler,
    EventContext, input::{Input, Metadata, Reader, Codec, Container}
};
use songbird::input::reader::MediaSource;

// pub struct Recorder {
//     writer_sender: UnboundedSender<Vec<i16>>
// }

// impl Recorder {
//     pub fn new() -> Self {
//         let (sender, mut receiver) = unbounded_channel();

//         tokio::spawn(async move {
//             let mut writer = WavWriter::create(format!("test{}.wav", SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis()), hound::WavSpec {
//                 channels: 2,
//                 sample_rate: 48000,
//                 bits_per_sample: 16,
//                 sample_format: hound::SampleFormat::Int
//             }).unwrap();
            
//             while let Some(voice_data) = receiver.recv().await {
//                 for frame in voice_data {
//                     writer.write_sample(frame); // TODO: Handle Result
//                 }
//             }

//             writer.finalize(); // TODO: Handle Result
//         });

//         Self {
//             writer_sender: sender
//         }
//     }

//     /*pub fn finish(&mut self) {
//         self.writer.finalize();
//     }*/
// }

// #[async_trait]
// impl VoiceEventHandler for Recorder {
//     async fn act(&self, ctx: &EventContext<'_>) {
//         match ctx {
//             EventContext::VoicePacket(data) => {
//                 if let Some(audio) = data.audio {
//                     self.writer_sender.send(audio);
//                 }
//             },
//             _ => ()
//         }
//         // println!("{:?} {:?} {:?} {:?}", ssrc, sequence, stereo, timestamp);
//         // self.writer_sender.send(data.to_vec());
//         /*let mut file = OpenOptions::new().append(true).open("./test.mp3").unwrap();
//         let mut result: Vec<u8> = Vec::new();
//         for &n in data {
//             let _ = result.write_i16::<LittleEndian>(n);
//         }
//         file.write_all(&result);*/
//     }

//     // async fn client_disconnect(&self, _user_id: u64) {
//     //     // self.writer.lock().unwrap().flush();
//     // }
// }

// pub struct EmptyAudioSource(pub usize);

// #[async_trait]
// impl MediaSource for EmptyAudioSource {
//     async fn is_stereo(&mut self) -> bool {
//         true
//     }

//     async fn get_type(&self) -> AudioType {
//         AudioType::Pcm
//     }

//     async fn read(&mut self, buffer: &mut [i16]) -> Option<usize> {
//         for el in buffer.iter_mut() {
//             *el = 0;
//         }

//         if self.0 != 0 {
//             self.0 -= buffer.len().min(self.0);
//             Some(buffer.len())
//         } else {
//             None
//         }
//     }

//     async fn read_opus_frame(&mut self) -> Option<Vec<u8>> {
//         None
//     }

//     async fn decode_and_add_opus_frame(&mut self, float_buffer: &mut [f32; 1920], volume: f32) -> Option<usize> {
//         None
//     }
// }

// Audio Send


pub struct TTSSource {
    child: Child,
}

impl Seek for TTSSource {
    fn seek(&mut self, _:SeekFrom) -> Result<u64> {
        todo!()
    }
}

// #[async_trait]
impl Read for TTSSource {
    fn read(&mut self, buffer: &mut [u8]) -> Result<usize> {
        self.child.stdout.as_mut().unwrap().read(buffer)
        // for (i, val) in buffer.iter_mut().enumerate() {
        //     let mut raw_data = [0];
        //     match self.child.stdout.as_mut().unwrap().read(&mut raw_data) {
        //         Ok(bytes_read) => {
        //             if bytes_read == 0 {
        //                 return Ok(0);
        //             }
                    
        //             *val = raw_data[0];
        //             // *val = i16::from_le_bytes(raw_data);
        //         },
        //         Err(e) => {
        //             if e.kind() == ErrorKind::UnexpectedEof {
        //                 return Ok(i);
        //             }

        //             return Ok(0);
        //         }
        //     }
        // }

        // Ok(buffer.len())
    }
}

impl MediaSource for TTSSource {
    fn is_seekable(&self) -> bool {
        true
    }

    fn byte_len(&self) -> Option<u64> {
        None
    }
    // async fn is_stereo(&mut self) -> bool {
    //     true
    // }

    // async fn get_type(&self) -> AudioType {
    //     AudioType::Pcm
    // }

    

    // async fn read_opus_frame(&mut self) -> std::option::Option<std::vec::Vec<u8>> {
    //     todo!()
    // }

    // async fn decode_and_add_opus_frame(&mut self, _: &mut [f32; 1920], _: f32) -> std::option::Option<usize> {
    //     todo!()
    // }
}

pub fn create_tts_source(url: &str) -> Result<Input> {
    let child = Command::new("ffmpeg")
        .args(&[
            "-i",
            url,
            "-f",
            "s16le",
            "-ac",
            "2",
            "-ar",
            "48000",
            "-acodec",
            "pcm_s16le",
            "-",
        ])
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .stdout(Stdio::piped())
        .spawn()?;
    
    Ok(Input::new(true, Reader::Extension(Box::new(TTSSource { child })), Codec::Pcm, Container::Raw, None))
}

pub fn create_mp3_source(url: &str) -> Result<Input> {
    let child = Command::new("ffmpeg")
        .args(&[
            "-i",
            url,
            "-f",
            "mp3",
            "-ac",
            "2",
            "-ar",
            "48000",
            "-acodec",
            "mp3",
            "-",
        ])
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .stdout(Stdio::piped())
        .spawn()?;
    
    Ok(Input::new(true, Reader::Extension(Box::new(TTSSource { child })), Codec::Pcm, Container::Raw, None))
}
