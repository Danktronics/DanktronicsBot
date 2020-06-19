use std::collections::HashSet;

pub struct GuildSettings {
    pub volume: u16,
    pub tts_channels: HashSet<u64>
}

impl GuildSettings {
    pub fn new() -> GuildSettings {
        GuildSettings {
            volume: 1,
            tts_channels: HashSet::new()
        }
    }
}