use std;

pub trait Track {
  // Can't use Iterator: annoying errors.
  fn next(&mut self) -> Option<f32>;
  fn is_done(&self) -> bool;
}

#[allow(unused)]
pub struct OneShotTrack {
  data: Vec<f32>,
  idx: usize,
}

impl OneShotTrack {
  #[allow(unused)]
  pub fn new(data: Vec<f32>) -> Self {
    OneShotTrack {
      data: data,
      idx: 0,
    }
  }
}

impl Track for OneShotTrack {
  fn next(&mut self) -> Option<f32> {
    if self.is_done() {
      None
    } else {
      let r = self.data[self.idx];
      self.idx = self.idx + 1;
      Some(r)
    }
  }

  fn is_done(&self) -> bool {
    self.idx >= self.data.len()
  }
}

pub struct LoopTrack {
  data: Vec<f32>,
  idx: usize,
}

impl LoopTrack {
  pub fn new(data: Vec<f32>, start: usize) -> Self {
    assert!(!data.is_empty());
    LoopTrack {
      data: data,
      idx: start,
    }
  }
}

impl Track for LoopTrack {
  fn next(&mut self) -> Option<f32> {
    let r = self.data[self.idx];
    self.idx = self.idx + 1;
    if self.idx >= self.data.len() {
      self.idx = 0;
    }
    Some(r)
  }

  fn is_done(&self) -> bool {
    false
  }
}

pub struct TracksPlaying {
  tracks: Vec<Box<Track>>,
  ready: bool,
  buffer: Vec<f32>,
}

unsafe impl Sync for TracksPlaying {}

impl TracksPlaying {
  pub fn new(buffer_len: usize) -> Self {
    TracksPlaying {
      tracks: Vec::new(),
      ready: false,
      buffer: std::iter::repeat(0.0).take(buffer_len).collect(),
    }
  }

  pub fn push(&mut self, t: Box<Track>) {
    self.tracks.push(t);
  }

  pub fn refresh_buffer(&mut self) {
    if self.ready {
      return
    }

    for x in &mut self.buffer {
      *x = 0.0;
    }

    for track in &mut self.tracks {
      for buffer in &mut self.buffer {
        match track.next() {
          None => break,
          Some(x) => *buffer = *buffer + x,
        }
      }
    }

    let mut i = 0;
    while i < self.tracks.len() {
      if self.tracks[i].is_done() {
        self.tracks.swap_remove(i);
      } else {
        i += 1;
      }
    }

    self.ready = true;
  }

  #[allow(unused)]
  pub fn with_buffer<F>(&mut self, f: F)
    where F: FnOnce(&mut [f32])
  {
    if self.ready {
      f(&mut self.buffer);
      self.ready = false;
    }
  }
}