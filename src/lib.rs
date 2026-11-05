#[derive(Debug, Clone, PartialEq)]
pub struct AudioSink {
    pub name: String,
    pub volume_percent: u8,
    pub is_muted: bool,
}

#[derive(Debug, Default)]
pub struct SoundIndicatorEngine {
    sinks: Vec<AudioSink>,
    active_sink_idx: usize,
}

impl SoundIndicatorEngine {
    pub fn new() -> Self { Self { sinks: Vec::new(), active_sink_idx: 0 } }
    pub fn add_sink(&mut self, name: &str, vol: u8) {
        self.sinks.push(AudioSink { name: name.into(), volume_percent: vol.min(100), is_muted: false });
    }
    pub fn set_volume(&mut self, vol: u8) {
        if let Some(s) = self.sinks.get_mut(self.active_sink_idx) {
            s.volume_percent = vol.min(100);
        }
    }
    pub fn toggle_mute(&mut self) {
        if let Some(s) = self.sinks.get_mut(self.active_sink_idx) {
            s.is_muted = !s.is_muted;
        }
    }
    pub fn current_sink(&self) -> Option<&AudioSink> {
        self.sinks.get(self.active_sink_idx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_sound_indicator_engine_aaa() {
        // Arrange
        let mut engine = SoundIndicatorEngine::new();
        engine.add_sink("Headphones", 45);
        // Act
        engine.set_volume(80);
        engine.toggle_mute();
        // Assert
        let sink = engine.current_sink().expect("Sink should exist");
        assert_eq!(sink.volume_percent, 80);
        assert!(sink.is_muted);
    }
}
