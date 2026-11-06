//! SigmaOS-indicator-sound — panel sound indicator engine
//! (Linux Mint `indicator-sound` parity).
//!
//! Manages output sinks (speakers/headphones/HDMI), per-sink volume &
//! mute, scroll-step adjustment, sink switching, and a simple peak
//! meter model fed by the audio subsystem.

#[derive(Debug, Clone, PartialEq)]
pub struct AudioSink {
    pub name: String,
    pub volume_percent: u8,
    pub is_muted: bool,
    /// Sink kind for icon selection.
    pub kind: SinkKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SinkKind {
    Speakers,
    Headphones,
    Hdmi,
    Usb,
    Bluetooth,
}

impl SinkKind {
    pub fn icon_hint(self) -> &'static str {
        match self {
            Self::Speakers => "audio-speakers",
            Self::Headphones => "audio-headphones",
            Self::Hdmi => "audio-hdmi",
            Self::Usb => "audio-usb",
            Self::Bluetooth => "audio-bluetooth",
        }
    }
}

#[derive(Debug, Default)]
pub struct SoundIndicatorEngine {
    sinks: Vec<AudioSink>,
    active_sink_idx: usize,
    /// Peak level 0-100 from the mixer, decays over time.
    peak: u8,
}

/// Scroll wheel volume step used by real indicators.
pub const SCROLL_STEP: u8 = 5;

impl SoundIndicatorEngine {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_sink(&mut self, name: &str, vol: u8) {
        self.add_sink_typed(name, vol, SinkKind::Speakers);
    }

    pub fn add_sink_typed(&mut self, name: &str, vol: u8, kind: SinkKind) {
        self.sinks.push(AudioSink {
            name: name.into(),
            volume_percent: vol.min(100),
            is_muted: false,
            kind,
        });
    }

    pub fn set_volume(&mut self, vol: u8) {
        if let Some(s) = self.sinks.get_mut(self.active_sink_idx) {
            s.volume_percent = vol.min(100);
            // unmute on explicit volume change (standard indicator behaviour)
            s.is_muted = false;
        }
    }

    /// Scroll up/down: +step / -step clamped to 0..=100.
    pub fn scroll_volume(&mut self, up: bool) {
        if let Some(s) = self.sinks.get_mut(self.active_sink_idx) {
            let v = if up {
                s.volume_percent.saturating_add(SCROLL_STEP)
            } else {
                s.volume_percent.saturating_sub(SCROLL_STEP)
            };
            s.volume_percent = v.min(100);
            s.is_muted = false;
        }
    }

    pub fn toggle_mute(&mut self) {
        if let Some(s) = self.sinks.get_mut(self.active_sink_idx) {
            s.is_muted = !s.is_muted;
        }
    }

    /// Mute/unmute all sinks at once.
    pub fn set_all_muted(&mut self, muted: bool) {
        for s in &mut self.sinks {
            s.is_muted = muted;
        }
    }

    /// Cycle to next output sink; returns new index.
    pub fn switch_sink(&mut self) -> Option<usize> {
        if self.sinks.len() < 2 {
            return None;
        }
        self.active_sink_idx = (self.active_sink_idx + 1) % self.sinks.len();
        Some(self.active_sink_idx)
    }

    pub fn active_sink_index(&self) -> usize {
        self.active_sink_idx
    }

    pub fn sink_count(&self) -> usize {
        self.sinks.len()
    }

    pub fn sink_names(&self) -> Vec<&str> {
        self.sinks.iter().map(|s| s.name.as_str()).collect()
    }

    pub fn current_sink(&self) -> Option<&AudioSink> {
        self.sinks.get(self.active_sink_idx)
    }

    /// Feed a peak sample 0-100; engine keeps the max and decays it.
    pub fn feed_peak(&mut self, level: u8) {
        let level = level.min(100);
        if level > self.peak {
            self.peak = level;
        }
    }

    /// Decay peak by `amount`; call periodically for meter animation.
    pub fn decay_peak(&mut self, amount: u8) {
        self.peak = self.peak.saturating_sub(amount);
    }

    pub fn peak(&self) -> u8 {
        self.peak
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sound_indicator_engine_aaa() {
        let mut engine = SoundIndicatorEngine::new();
        engine.add_sink("Headphones", 45);
        engine.set_volume(80);
        engine.toggle_mute();
        let sink = engine.current_sink().expect("Sink should exist");
        assert_eq!(sink.volume_percent, 80);
        assert!(sink.is_muted);
    }

    #[test]
    fn scroll_steps_and_clamps() {
        let mut e = SoundIndicatorEngine::new();
        e.add_sink("Speakers", 98);
        e.scroll_volume(true);
        assert_eq!(e.current_sink().unwrap().volume_percent, 100);
        e.scroll_volume(false);
        e.scroll_volume(false);
        e.scroll_volume(false);
        e.scroll_volume(false);
        e.scroll_volume(false);
        assert_eq!(e.current_sink().unwrap().volume_percent, 75);
    }

    #[test]
    fn volume_change_unmutes() {
        let mut e = SoundIndicatorEngine::new();
        e.add_sink("HDMI", 50);
        e.toggle_mute();
        assert!(e.current_sink().unwrap().is_muted);
        e.scroll_volume(true);
        assert!(!e.current_sink().unwrap().is_muted);
    }

    #[test]
    fn sink_switching_and_peak() {
        let mut e = SoundIndicatorEngine::new();
        e.add_sink_typed("Speakers", 50, SinkKind::Speakers);
        e.add_sink_typed("Headphones", 70, SinkKind::Headphones);
        assert_eq!(e.switch_sink(), Some(1));
        assert_eq!(e.current_sink().unwrap().name, "Headphones");
        assert_eq!(e.switch_sink(), Some(0));
        e.feed_peak(90);
        e.feed_peak(40);
        assert_eq!(e.peak(), 90);
        e.decay_peak(50);
        assert_eq!(e.peak(), 40);
    }
}
