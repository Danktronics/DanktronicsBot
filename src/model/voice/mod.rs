use std::fs::OpenOptions;
use std::io::Write;
use std::io::BufWriter;
use std::fs::File;
use serenity::voice::{AudioReceiver, AudioSource, AudioType};
use byteorder::{WriteBytesExt, LittleEndian};
use hound::{WavWriter, WavSpec, SampleFormat::Int};

pub struct Recorder {
    writer: WavWriter<BufWriter<File>>
}

impl Recorder {
    pub fn new() -> Self {
        Self {
            writer: WavWriter::create("test.wav", hound::WavSpec {
                channels: 2,
                sample_rate: 44100,
                bits_per_sample: 16,
                sample_format: hound::SampleFormat::Int
            }).unwrap()
        }
    }

    /*pub fn finish(&mut self) {
        self.writer.finalize();
    }*/
}

impl AudioReceiver for Recorder {
    fn voice_packet(&mut self, ssrc: u32, sequence: u16, timestamp: u32, stereo: bool, data: &[i16], compressed_size: usize) {
        for frame in data {
            self.writer.write_sample(*frame);
        }
        /*let mut file = OpenOptions::new().append(true).open("./test.mp3").unwrap();
        let mut result: Vec<u8> = Vec::new();
        for &n in data {
            let _ = result.write_i16::<LittleEndian>(n);
        }
        file.write_all(&result);*/
    }

    fn client_disconnect(&mut self, _user_id: u64) {
        self.writer.flush();
    }
}

pub struct EmptyAudioSource(pub usize);

impl AudioSource for EmptyAudioSource {
    fn is_stereo(&mut self) -> bool {
        true
    }

    fn get_type(&self) -> AudioType {
        AudioType::Pcm
    }

    fn read_pcm_frame(&mut self, buffer: &mut [i16]) -> Option<usize> {
        for el in buffer.iter_mut() {
            *el = 0;
        }

        if self.0 != 0 {
            self.0 -= buffer.len().min(self.0);
            Some(buffer.len())
        } else {
            None
        }
    }

    fn read_opus_frame(&mut self) -> Option<Vec<u8>> {
        None
    }

    fn decode_and_add_opus_frame(&mut self, float_buffer: &mut [f32; 1920], volume: f32) -> Option<usize> {
        None
    }
}