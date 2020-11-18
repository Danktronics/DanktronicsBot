use std::io::{Read, ErrorKind, Result};
use std::process::{Child, Command, Stdio};
use serenity::voice::{AudioReceiver, AudioSource, AudioType};

pub struct Recorder;

impl Recorder {
    pub fn new() -> Self {
        Self { }
    }
}

impl AudioReceiver for Recorder {
    fn voice_packet(&mut self, ssrc: u32, sequence: u16, timestamp: u32, stereo: bool, data: &[i16], compressed_size: usize) {
        println!("Received a voice packet");
    }
}

// Audio Send

pub struct TTSSource {
    child: Child,
}

impl AudioSource for TTSSource {
    fn is_stereo(&mut self) -> bool {
        true
    }

    fn get_type(&self) -> AudioType {
        AudioType::Pcm
    }

    fn read_pcm_frame(&mut self, buffer: &mut [i16]) -> Option<usize> {
        for (i, val) in buffer.iter_mut().enumerate() {
            let mut raw_data = [0, 0];
            match self.child.stdout.as_mut().unwrap().read(&mut raw_data) {
                Ok(bytes_read) => {
                    if bytes_read == 0 {
                        return None;
                    }

                    *val = i16::from_le_bytes(raw_data);
                },
                Err(e) => {
                    if e.kind() == ErrorKind::UnexpectedEof {
                        return Some(i);
                    }

                    return None;
                }
            }
        }

        Some(buffer.len())
    }

    fn read_opus_frame(&mut self) -> std::option::Option<std::vec::Vec<u8>> {
        todo!()
    }

    fn decode_and_add_opus_frame(&mut self, _: &mut [f32; 1920], _: f32) -> std::option::Option<usize> {
        todo!()
    }
}

pub fn create_tts_source(url: &str) -> Result<TTSSource> {
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
    
    Ok(TTSSource {
        child
    })
}
