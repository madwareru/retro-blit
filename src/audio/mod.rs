use std::collections::VecDeque;
use std::io::{BufReader, Cursor};
use rodio::{Source, DeviceSinkError, decoder::DecoderError, Player};
use rodio::mixer::{Mixer, mixer};
use rodio::source::Zero;

type MemoryDecoder = rodio::Decoder<Cursor<&'static[u8]>>;
type FileDecoder = rodio::Decoder<BufReader<std::fs::File>>;

#[derive(Clone)]
pub enum SoundHandle {
    Memory(rodio::source::Buffered<MemoryDecoder>),
    File(rodio::source::Buffered<FileDecoder>)
}
impl SoundHandle {
    pub fn from_file(file: std::fs::File) -> Result<Self, DecoderError> {
        let reader = BufReader::new(file);
        let decoder = rodio::Decoder::new(reader)?;
        Ok(Self::File(decoder.buffered()))
    }
    pub fn from_memory(bytes: &'static [u8]) -> Result<Self, DecoderError> {
        let cursor = Cursor::new(bytes);
        let decoder = rodio::Decoder::new(cursor)?;
        Ok(Self::Memory(decoder.buffered()))
    }
}

pub struct SoundDriver {
    _stream: rodio::MixerDeviceSink,
    active_sounds: Vec<Option<Player>>,
    free_list: VecDeque<usize>,
    global_sink: Player,
    global_mixer_controller: Mixer
}
impl SoundDriver {
    pub fn try_create() -> Result<Self, DeviceSinkError> {
        let mut stream = rodio::DeviceSinkBuilder::from_default_device()?
            .with_buffer_size(rodio::cpal::BufferSize::Fixed(1024))
            .open_sink_or_fallback()?;
        stream.log_on_drop(false);
        let free_list = VecDeque::new();
        let active_sounds = Vec::new();
        let global_sink = Player::connect_new(stream.mixer());
        let (global_mixer_controller, global_dynamic_mixer) =
            mixer(rodio::nz!(2), rodio::nz!(44100));
        global_mixer_controller.add(Zero::new(rodio::nz!(2), rodio::nz!(44100)));
        global_sink.append(global_dynamic_mixer);
        Ok(Self {
            _stream: stream,
            active_sounds,
            free_list,
            global_sink,
            global_mixer_controller
        })
    }

    pub fn set_global_volume(&self, volume: f32) {
        self.global_sink.set_volume(volume * volume);
    }

    pub fn play_sound(&mut self, sound: SoundHandle) -> usize
    {
        let (sink, queue_rx) = Player::new();
        self.global_mixer_controller.add(queue_rx);
        match sound {
            SoundHandle::Memory(memory_sound) => sink.append(memory_sound),
            SoundHandle::File(file_sound) => sink.append(file_sound),
        }

        match self.free_list.pop_back() {
            Some(id) => {
                self.active_sounds[id] = Some(sink);
                id
            },
            _ => {
                let id = self.active_sounds.len();
                self.active_sounds.push(Some(sink));
                id
            }
        }
    }

    pub fn playback_in_progress(&self, play_handle: usize) -> bool {
        self.active_sounds.get(play_handle)
            .map(|it| it.is_some())
            .unwrap_or(false)
    }

    pub fn set_volume(&self, play_handle: usize, volume: f32) {
        let Some(sink_ref) = self.active_sounds.get(play_handle) else { return;} ;
        let Some(sink) = sink_ref else { return; };
        sink.set_volume(volume * volume);
    }

    pub fn pause_playback(&self, play_handle: usize) {
        let Some(sink_ref) = self.active_sounds.get(play_handle) else { return;} ;
        let Some(sink) = sink_ref else { return; };
        sink.pause();
    }

    pub fn continue_playback(&self, play_handle: usize) {
        let Some(sink_ref) = self.active_sounds.get(play_handle) else { return;} ;
        let Some(sink) = sink_ref else { return; };
        sink.play();
    }

    pub fn stop_playback(&mut self, play_handle: usize) {
        let Some(sink_ref) = self.active_sounds.get(play_handle) else { return;} ;
        let Some(sink) = sink_ref else { return; };
        sink.stop();
    }

    pub fn maintain(&mut self) {
        for i in 0..self.active_sounds.len() {
            let Some(sink) = &mut self.active_sounds[i] else { continue; };
            if !sink.empty() { continue; }
            self.active_sounds[i] = None;
            self.free_list.push_back(i);
        }
    }
}