// Procedural sound effects — generates simple 8-bit-style WAV audio
// at runtime. No external files needed.
//
// WAV format: PCM 16-bit signed, mono, 22050 Hz.

use std::f32::consts::PI;

/// Sample rate for generated sounds (CD-quality would be 44100, but
/// 22050 is plenty for retro-style effects and saves memory).
const SAMPLE_RATE: u32 = 22050;

// ============================================================================
// Waveform generators
// ============================================================================

/// Fill `buf` with a sine wave of `freq` Hz at `volume` (0..1).
fn sine_wave(buf: &mut [i16], freq: f32, volume: f32) {
    let amp = (volume * 16384.0) as i16; // ~50% of i16 max
    for (i, sample) in buf.iter_mut().enumerate() {
        let t = i as f32 / SAMPLE_RATE as f32;
        *sample = (amp as f32 * (2.0 * PI * freq * t).sin()) as i16;
    }
}

/// Fill `buf` with a square wave of `freq` Hz at `volume` (0..1).
fn square_wave(buf: &mut [i16], freq: f32, volume: f32) {
    let amp = (volume * 12288.0) as i16; // slightly quieter than sine
    let period = SAMPLE_RATE as f32 / freq;
    for (i, sample) in buf.iter_mut().enumerate() {
        let phase = (i as f32 % period) / period;
        *sample = if phase < 0.5 { amp } else { -amp };
    }
}

/// Fill `buf` with a triangle wave of `freq` Hz at `volume` (0..1).
fn triangle_wave(buf: &mut [i16], freq: f32, volume: f32) {
    let amp = (volume * 14336.0) as i16;
    let period = SAMPLE_RATE as f32 / freq;
    for (i, sample) in buf.iter_mut().enumerate() {
        let phase = (i as f32 % period) / period;
        let tri = if phase < 0.5 {
            1.0 - 4.0 * phase
        } else {
            4.0 * phase - 3.0
        };
        *sample = (amp as f32 * tri) as i16;
    }
}

/// Fill `buf` with white noise at `volume` (0..1).
fn noise_wave(buf: &mut [i16], volume: f32) {
    let amp = (volume * 8192.0) as i16;
    // Simple LCG PRNG for noise
    let mut state: u32 = 0xABCD;
    for sample in buf.iter_mut() {
        state = state.wrapping_mul(1103515245).wrapping_add(12345);
        let n = (state >> 16) as i16;
        *sample = (n as f32 / 32768.0 * amp as f32) as i16;
    }
}

/// Fill `buf` with a frequency sweep (sine wave that changes from
/// `freq_start` to `freq_end` over the duration of the buffer).
fn sweep_wave(buf: &mut [i16], freq_start: f32, freq_end: f32, volume: f32) {
    let amp = (volume * 14336.0) as i16;
    let n = buf.len() as f32;
    for (i, sample) in buf.iter_mut().enumerate() {
        let t = i as f32 / SAMPLE_RATE as f32;
        let frac = i as f32 / n;
        let freq = freq_start + (freq_end - freq_start) * frac;
        *sample = (amp as f32 * (2.0 * PI * freq * t).sin()) as i16;
    }
}

/// Apply a simple exponential decay envelope to `buf`.
fn apply_decay(buf: &mut [i16], decay_rate: f32) {
    let n = buf.len();
    for (i, sample) in buf.iter_mut().enumerate() {
        let envelope = (-decay_rate * i as f32 / n as f32).exp();
        *sample = (*sample as f32 * envelope) as i16;
    }
}

/// Apply a quick fade-out over the last `fade_samples` of the buffer.
fn apply_fade_out(buf: &mut [i16], fade_samples: usize) {
    let len = buf.len();
    let start = len.saturating_sub(fade_samples);
    for i in start..len {
        let frac = (len - i) as f32 / fade_samples as f32;
        buf[i] = (buf[i] as f32 * frac) as i16;
    }
}

// ============================================================================
// WAV encoding
// ============================================================================

/// Encode raw i16 mono samples into a complete WAV file (Vec<u8>).
fn encode_wav(samples: &[i16]) -> Vec<u8> {
    let data_size = (samples.len() * 2) as u32; // 2 bytes per i16
    let file_size = 44 + data_size;

    let mut wav = Vec::with_capacity(file_size as usize);

    // RIFF header
    wav.extend_from_slice(b"RIFF");
    wav.extend_from_slice(&file_size.to_le_bytes());
    wav.extend_from_slice(b"WAVE");

    // fmt chunk
    wav.extend_from_slice(b"fmt ");
    wav.extend_from_slice(&16u32.to_le_bytes()); // chunk size
    wav.extend_from_slice(&1u16.to_le_bytes());  // PCM format
    wav.extend_from_slice(&1u16.to_le_bytes());  // 1 channel (mono)
    wav.extend_from_slice(&SAMPLE_RATE.to_le_bytes());
    let byte_rate = SAMPLE_RATE * 2; // sample_rate * channels * bytes_per_sample
    wav.extend_from_slice(&byte_rate.to_le_bytes());
    wav.extend_from_slice(&2u16.to_le_bytes());  // block align
    wav.extend_from_slice(&16u16.to_le_bytes()); // bits per sample

    // data chunk
    wav.extend_from_slice(b"data");
    wav.extend_from_slice(&data_size.to_le_bytes());
    for sample in samples {
        wav.extend_from_slice(&sample.to_le_bytes());
    }

    wav
}

// ============================================================================
// Sound effect presets
// ============================================================================

/// Duration helper: number of samples for `secs` seconds.
fn samples_for(secs: f32) -> usize {
    (secs * SAMPLE_RATE as f32) as usize
}

/// Generate a jump sound — quick ascending sweep with a pop.
pub fn generate_jump() -> Vec<u8> {
    let n = samples_for(0.12);
    let mut buf = vec![0i16; n];
    sweep_wave(&mut buf, 250.0, 700.0, 0.7);
    apply_decay(&mut buf, 3.0);
    encode_wav(&buf)
}

/// Generate a coin collect sound — two-note "bling".
pub fn generate_coin() -> Vec<u8> {
    let n = samples_for(0.15);
    let mut buf = vec![0i16; n];
    let half = n / 2;
    square_wave(&mut buf[..half], 988.0, 0.6);
    square_wave(&mut buf[half..], 1319.0, 0.6);
    apply_decay(&mut buf, 4.0);
    encode_wav(&buf)
}

/// Generate a power-up collect sound — ascending arpeggio.
pub fn generate_powerup() -> Vec<u8> {
    let n = samples_for(0.35);
    let mut buf = vec![0i16; n];
    let notes = [523.0, 659.0, 784.0, 1047.0, 1319.0];
    let step = n / notes.len();
    for (i, &freq) in notes.iter().enumerate() {
        let start = i * step;
        let end = ((i + 1) * step).min(n);
        square_wave(&mut buf[start..end], freq, 0.6);
    }
    apply_decay(&mut buf, 2.0);
    encode_wav(&buf)
}

/// Generate an enemy stomp sound — low thud.
pub fn generate_stomp() -> Vec<u8> {
    let n = samples_for(0.10);
    let mut buf = vec![0i16; n];
    noise_wave(&mut buf, 0.5);
    triangle_wave(&mut buf, 80.0, 0.3);  // mix in low rumble
    apply_decay(&mut buf, 8.0);
    encode_wav(&buf)
}

/// Generate a damage sound — descending sweep.
pub fn generate_damage() -> Vec<u8> {
    let n = samples_for(0.20);
    let mut buf = vec![0i16; n];
    sweep_wave(&mut buf, 500.0, 150.0, 0.6);
    apply_decay(&mut buf, 3.0);
    encode_wav(&buf)
}

/// Generate a block bump sound — quick impact.
pub fn generate_bump() -> Vec<u8> {
    let n = samples_for(0.06);
    let mut buf = vec![0i16; n];
    triangle_wave(&mut buf, 200.0, 0.7);
    apply_decay(&mut buf, 12.0);
    encode_wav(&buf)
}

/// Generate a fireball shoot sound — short noise burst.
pub fn generate_fireball() -> Vec<u8> {
    let n = samples_for(0.12);
    let mut buf = vec![0i16; n];
    noise_wave(&mut buf, 0.4);
    apply_decay(&mut buf, 6.0);
    encode_wav(&buf)
}

/// Generate a 1-UP sound — rapid ascending notes.
pub fn generate_oneup() -> Vec<u8> {
    let n = samples_for(0.40);
    let mut buf = vec![0i16; n];
    let notes = [523.0, 659.0, 784.0, 1047.0, 1319.0, 1568.0];
    let step = n / notes.len();
    for (i, &freq) in notes.iter().enumerate() {
        let start = i * step;
        let end = ((i + 1) * step).min(n);
        square_wave(&mut buf[start..end], freq, 0.55);
    }
    apply_decay(&mut buf, 1.5);
    encode_wav(&buf)
}

/// Generate a death sound — slow descending melody.
pub fn generate_death() -> Vec<u8> {
    let n = samples_for(0.70);
    let mut buf = vec![0i16; n];
    let notes = [440.0, 370.0, 294.0, 220.0];
    let step = n / notes.len();
    for (i, &freq) in notes.iter().enumerate() {
        let start = i * step;
        let end = ((i + 1) * step).min(n);
        square_wave(&mut buf[start..end], freq, 0.6);
    }
    apply_decay(&mut buf, 1.8);
    encode_wav(&buf)
}

/// Generate a game over sound — sad descending notes.
pub fn generate_gameover() -> Vec<u8> {
    let n = samples_for(1.0);
    let mut buf = vec![0i16; n];
    let notes = [523.0, 392.0, 330.0, 262.0];
    let step = n / notes.len();
    for (i, &freq) in notes.iter().enumerate() {
        let start = i * step;
        let end = ((i + 1) * step).min(n);
        square_wave(&mut buf[start..end], freq, 0.5);
    }
    apply_decay(&mut buf, 1.5);
    apply_fade_out(&mut buf, samples_for(0.3));
    encode_wav(&buf)
}

/// Generate a level-complete flagpole sound — victory jingle.
pub fn generate_victory() -> Vec<u8> {
    let n = samples_for(1.2);
    let mut buf = vec![0i16; n];
    let notes = [
        (523.0, 0.10), (659.0, 0.10), (784.0, 0.10), (1047.0, 0.15),
        (784.0, 0.10), (1047.0, 0.25),
        (1319.0, 0.15), (1568.0, 0.25),
    ];
    let mut pos = 0;
    for &(freq, dur) in &notes {
        let end = (pos + samples_for(dur)).min(n);
        square_wave(&mut buf[pos..end], freq, 0.55);
        pos = end;
    }
    apply_decay(&mut buf, 1.2);
    encode_wav(&buf)
}

// ============================================================================
// SoundManager — loads and plays procedural sound effects.
// ============================================================================

use quad_snd::{AudioContext, PlaySoundParams, Sound};

/// Holds all game sound effects, loaded from procedural WAV data.
pub struct SoundManager {
    ctx: AudioContext,
    jump: Sound,
    coin: Sound,
    powerup: Sound,
    stomp: Sound,
    damage: Sound,
    bump: Sound,
    fireball: Sound,
    oneup: Sound,
    death: Sound,
    gameover: Sound,
    victory: Sound,
}

impl SoundManager {
    /// Create and load all sound effects. Call once during startup (requires
    /// an active audio backend — only call when screen_w > 0).
    pub fn new() -> Self {
        let ctx = AudioContext::new();

        let jump = Sound::load(&ctx, &generate_jump());
        let coin = Sound::load(&ctx, &generate_coin());
        let powerup = Sound::load(&ctx, &generate_powerup());
        let stomp = Sound::load(&ctx, &generate_stomp());
        let damage = Sound::load(&ctx, &generate_damage());
        let bump = Sound::load(&ctx, &generate_bump());
        let fireball = Sound::load(&ctx, &generate_fireball());
        let oneup = Sound::load(&ctx, &generate_oneup());
        let death = Sound::load(&ctx, &generate_death());
        let gameover = Sound::load(&ctx, &generate_gameover());
        let victory = Sound::load(&ctx, &generate_victory());

        Self { ctx, jump, coin, powerup, stomp, damage, bump, fireball, oneup, death, gameover, victory }
    }

    // Helper: play a sound once with default volume.
    fn play(&self, sound: &Sound) {
        sound.play(&self.ctx, PlaySoundParams { looped: false, volume: 0.6 });
    }

    pub fn play_jump(&mut self)       { self.play(&self.jump); }
    pub fn play_coin(&mut self)       { self.play(&self.coin); }
    pub fn play_powerup(&mut self)    { self.play(&self.powerup); }
    pub fn play_stomp(&mut self)      { self.play(&self.stomp); }
    pub fn play_damage(&mut self)     { self.play(&self.damage); }
    pub fn play_bump(&mut self)       { self.play(&self.bump); }
    pub fn play_fireball(&mut self)   { self.play(&self.fireball); }
    pub fn play_oneup(&mut self)      { self.play(&self.oneup); }
    pub fn play_death(&mut self)      { self.play(&self.death); }
    pub fn play_gameover(&mut self)   { self.play(&self.gameover); }
    pub fn play_victory(&mut self)    { self.play(&self.victory); }
}
