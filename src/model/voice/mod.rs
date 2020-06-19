use serenity::voice::AudioReceiver;

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