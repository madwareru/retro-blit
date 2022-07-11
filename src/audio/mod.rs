use std::collections::VecDeque;
use std::io::{BufReader, Cursor};
use std::sync::Arc;
use rodio::{Source, StreamError, decoder::DecoderError, Sink};
use rodio::dynamic_mixer::{DynamicMixerController, mixer};

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
    _stream: rodio::OutputStream,
    active_sounds: Vec<Option<rodio::Sink>>,
    free_list: VecDeque<usize>,
    global_sink: Sink,
    global_mixer_controller: Arc<DynamicMixerController<f32>>
}
impl SoundDriver {
    pub fn try_create() -> Result<Self, StreamError> {
        let (_stream, handle) = rodio::OutputStream::try_default()?;
        let free_list = VecDeque::new();
        let active_sounds = Vec::new();
        let global_sink = rodio::Sink::try_new(&handle).unwrap();
        let (global_mixer_controller, global_dynamic_mixer) =
            mixer(2, 44100);
        global_sink.append(global_dynamic_mixer);
        Ok(Self {
            _stream,
            active_sounds,
            free_list,
            global_sink,
            global_mixer_controller
        })
    }

    pub fn set_global_volume(&self, volume: f32) {
        self.global_sink.set_volume(volume);
    }

    pub fn play_sound(&mut self, sound: SoundHandle) -> usize
    {
        let (sink, queue_rx) = Sink::new_idle();
        self.global_mixer_controller.add(queue_rx);
        match sound {
            SoundHandle::Memory(memory_sound) => {
                sink.append(memory_sound);
            }
            SoundHandle::File(file_sound) => {
                sink.append(file_sound);
            }
        }
        let id = self.free_list
            .pop_back()
            .unwrap_or(self.active_sounds.len());
        if id < self.active_sounds.len() {
            self.active_sounds[id] = Some(sink);
        } else {
            self.active_sounds.push(Some(sink))
        }
        id
    }

    pub fn playback_in_progress(&self, play_handle: usize) -> bool {
        play_handle < self.active_sounds.len() &&
            self.active_sounds[play_handle].is_some()
    }

    pub fn set_volume(&self, play_handle: usize, volume: f32) {
        if play_handle >= self.active_sounds.len() { return; }
        if let Some(sink) = &self.active_sounds[play_handle] {
            sink.set_volume(volume);
        }
    }

    pub fn pause_playback(&self, play_handle: usize) {
        if play_handle >= self.active_sounds.len() { return; }
        if let Some(sink) = &self.active_sounds[play_handle] {
            sink.pause();
        }
    }

    pub fn continue_playback(&self, play_handle: usize) {
        if play_handle >= self.active_sounds.len() { return; }
        if let Some(sink) = &self.active_sounds[play_handle] {
            sink.play();
        }
    }

    pub fn stop_playback(&mut self, play_handle: usize) {
        if play_handle >= self.active_sounds.len() { return; }
        let stopped = if let Some(sink) = &mut self.active_sounds[play_handle] {
            sink.stop();
            true
        } else {
            false
        };
        if stopped {
            self.free_list.push_back(play_handle);
            self.active_sounds[play_handle] = None;
        }
    }

    pub fn maintain(&mut self) {
        for i in 0..self.active_sounds.len() {
            let should_free = match &(self.active_sounds[i]) {
                Some(sink) if sink.empty() => true,
                _ => false
            };
            if should_free {
                self.active_sounds[i] = None;
                self.free_list.push_back(i);
            }
        }
    }
}