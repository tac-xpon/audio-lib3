use sdl2::audio::{AudioCallback, AudioSpecDesired, AudioDevice};

pub type SoundData16 = Vec<u16>;
pub const SETUP_U16: i32 = 1 << 15;

pub struct Sound {
    buffer: SoundData16,
    buf_size: usize,
    volume: u16,
    mute: bool,
    current: usize,
    called: usize,
}

pub type SoundDevice = AudioDevice<Sound>;

pub trait Control {
    fn set_mute(&mut self, specifier: bool);
    fn set_volume(&mut self, volume: u16);
    fn set_data(&mut self, offset: usize, sound: &[u16]);
    fn buf_size(&mut self) -> usize;
    fn mute(&mut self) -> bool;
    fn volume(&mut self) -> u16;
    fn current(&mut self) -> usize;
    fn called(&mut self) -> usize;
}

impl Control for SoundDevice {
    fn set_mute(&mut self, specifier: bool) {
        let mut locked = self.lock();
        locked.mute = specifier;
    }

    fn set_volume(&mut self, volume: u16) {
        let mut locked = self.lock();
        locked.volume = volume;
    }

    fn set_data(&mut self, offset: usize, sound: &[u16]) {
        let mut locked = self.lock();
        let mut pos = offset;
        let len = locked.buf_size;
        for a in sound {
            locked.buffer[pos % len] = *a;
            pos += 1;
        }
    }

    fn buf_size(&mut self) -> usize {
        let locked = self.lock();
        locked.buf_size
    }

    fn mute(&mut self) -> bool {
        let locked = self.lock();
        locked.mute
    }

    fn volume(&mut self) -> u16 {
        let locked = self.lock();
        locked.volume
    }

    fn current(&mut self) -> usize {
        let locked = self.lock();
        locked.current
    }

    fn called(&mut self) -> usize {
        let locked = self.lock();
        locked.called
    }
}

impl AudioCallback for Sound {
    type Channel = u16;

    fn callback(&mut self, out: &mut [u16]) {
        for dst in out.iter_mut() {
            let output = if self.mute || self.volume == 0 {
                0
            } else {
            let pos = self.current % self.buf_size;
                let raw_sample = *self.buffer.get(pos).unwrap_or(&(SETUP_U16 as u16));
                let singed_sample = raw_sample as i32 - SETUP_U16;
                let scaled_singed_sample = match self.volume {
                    0 => 0,
                    1 => singed_sample >> 6,
                    2 => singed_sample >> 5,
                    3 => singed_sample >> 4,
                    4 => singed_sample >> 3,
                    5 => singed_sample >> 2,
                    6 => singed_sample >> 1,
                    _ => singed_sample ,
                };
                scaled_singed_sample
            };
            *dst = (output + SETUP_U16) as u16;
            self.current += 1;
        }
        self.called += 1;
    }
}

pub struct AudioContext {
    sdl_context: sdl2::Sdl,
    audio_subsystem: sdl2::AudioSubsystem,
    desired_spec: AudioSpecDesired,
}

impl Default for AudioContext {
    fn default() -> Self {
        Self::new()
    }
}

impl AudioContext {
    pub fn new() -> Self {
        let sdl_context = sdl2::init().unwrap();
        let audio_subsystem = sdl_context.audio().unwrap();
        Self::with_subsystem(audio_subsystem)
    }

    pub fn with_subsystem(audio_subsystem: sdl2::AudioSubsystem) -> Self {
        let sdl_context = audio_subsystem.sdl();
        let desired_spec = AudioSpecDesired {
            freq: None,     // default
            channels: None, // default
            samples: None,  // default
        };
        Self {
            sdl_context,
            audio_subsystem,
            desired_spec,
        }
    }

    pub fn sdl(&self) -> sdl2::Sdl {
        self.sdl_context.clone()
    }

    pub fn freq(&self) -> Option<i32> {
        self.desired_spec.freq
    }

    pub fn set_freq(&mut self, freq: Option<i32>) {
        self.desired_spec.freq = freq;
    }

    pub fn channels(&self) -> Option<u8> {
        self.desired_spec.channels
    }

    pub fn set_channels(&mut self, channels: Option<u8>) {
        self.desired_spec.channels = channels;
    }

    pub fn samples(&self) -> Option<u16> {
        self.desired_spec.samples
    }

    pub fn set_samples(&mut self, samples: Option<u16>) {
        self.desired_spec.samples = samples;
    }

    pub fn open_device(&self, len: usize) -> Result<SoundDevice, String> {
        self.audio_subsystem.open_playback(None, &self.desired_spec, |_spec| {
            Sound {
                buffer: vec![SETUP_U16 as u16; len],
                buf_size: len,
                volume: 0,
                current: 0,
                mute: false,
                called: 0,
            }
        })
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
