use hound;
use portaudio;
use std;
use std::io::Write;
use std::sync::{Mutex};
use stopwatch;
use thread_scoped;

use common::protocol;

use client;
use server;
use record_book;
use update_thread::update_thread;
use view_thread::view_thread;

struct Track {
  data: Vec<f32>,
  idx: usize,
}

impl Track {
  pub fn new(data: Vec<f32>) -> Self {
    Track {
      data: data,
      idx: 0,
    }
  }

  pub fn is_done(&self) -> bool {
    self.idx >= self.data.len()
  }
}

impl Iterator for Track {
  type Item = f32;
  fn next(&mut self) -> Option<f32> {
    if self.is_done() {
      None
    } else {
      let r = self.data[self.idx];
      self.idx = self.idx + 1;
      Some(r)
    }
  }
}

struct TracksPlaying {
  tracks: Vec<Track>,
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

  pub fn push(&mut self, t: Track) {
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

#[allow(missing_docs)]
pub fn run(listen_url: &str, server_url: &str) {
  let voxel_updates = Mutex::new(std::collections::VecDeque::new());
  let view_updates0 = Mutex::new(std::collections::VecDeque::new());
  let view_updates1 = Mutex::new(std::collections::VecDeque::new());

  let buffer_size = 1 << 10;
  let mut tracks_playing: TracksPlaying = TracksPlaying::new(buffer_size);

  let quit = Mutex::new(false);
  let quit = &quit;

  let server = server::new(&server_url, &listen_url);

  let client = connect_client(&listen_url, &server);
  let client = &client;

  {
    let monitor_thread = {
      unsafe {
        thread_scoped::scoped(|| {
          while !*quit.lock().unwrap() {
            info!("Outstanding voxel updates: {}", voxel_updates.lock().unwrap().len());
            info!("Outstanding view0 updates: {}", view_updates0.lock().unwrap().len());
            info!("Outstanding view1 updates: {}", view_updates1.lock().unwrap().len());
            std::thread::sleep(std::time::Duration::from_secs(1));
          }
        })
      }
    };

    let audio_thread = {
      unsafe {
        let tracks_playing: *mut TracksPlaying = std::mem::transmute(&mut tracks_playing);
        let tracks_playing: &mut TracksPlaying = std::mem::transmute(tracks_playing);
        thread_scoped::scoped(move || {
          let sample_rate = 44100.0;
          let channels = 2;

          let portaudio = portaudio::PortAudio::new().unwrap();
          let params = portaudio.default_output_stream_params(channels).unwrap();
          let settings = portaudio::OutputStreamSettings::new(params, sample_rate, buffer_size as u32);

          let callback = {
            let tracks_playing: *mut TracksPlaying = std::mem::transmute(&mut *tracks_playing);
            let tracks_playing: &mut TracksPlaying = std::mem::transmute(tracks_playing);
            move |portaudio::OutputStreamCallbackArgs { buffer, .. }| {
              for x in buffer.iter_mut() {
                *x = 0.0;
              }
              tracks_playing.with_buffer(|b| {
                assert!(2 * b.len() == buffer.len());
                for (i, x) in buffer.iter_mut().enumerate() {
                  *x = b[i / 2];
                }
              });
              portaudio::StreamCallbackResult::Continue
            }
          };

          let mut stream = portaudio.open_non_blocking_stream(settings, callback).unwrap();
          stream.start().unwrap();

          let ambient_track = load_ambient_track();
          tracks_playing.push(ambient_track);

          while !*quit.lock().unwrap() && stream.is_active() == Ok(true) {
            tracks_playing.refresh_buffer();
            std::thread::sleep(std::time::Duration::from_millis(10));
          }

          println!("done");
          stream.stop().unwrap();
          stream.close().unwrap();
        })
      }
    };

    let update_thread = {
      let client = &client;
      let view_updates0 = &view_updates0;
      let view_updates1 = &view_updates1;
      let voxel_updates = &voxel_updates;
      let server = server.clone();
      unsafe {
        thread_scoped::scoped(move || {
          update_thread(
            quit,
            client,
            &mut || { server.listen.try() },
            &mut || { voxel_updates.lock().unwrap().pop_front() },
            &mut |up| { view_updates0.lock().unwrap().push_back(up) },
            &mut |up| { view_updates1.lock().unwrap().push_back(up) },
  	        &mut |up| { server.talk.tell(&up) },
            &mut |request_time, updates, reason| { voxel_updates.lock().unwrap().push_back((request_time, updates, reason)) },
          );

          let mut recorded = record_book::thread_local::clone();
          recorded.block_loads.sort_by(|x, y| x.loaded_at.cmp(&y.loaded_at));

          let mut file = std::fs::File::create("block_loads.out").unwrap();

          file.write_all(b"records = [").unwrap();
          for (i, record) in recorded.block_loads.iter().enumerate() {
            if i > 0 {
              file.write_all(b", ").unwrap();
            }
            file.write_fmt(format_args!("[{}; {}; {}; {}]", record.requested_at, record.responded_at, record.processed_at, record.loaded_at)).unwrap();
          }
          file.write_all(b"];\n").unwrap();
          file.write_fmt(format_args!("plot([1:{}], records);", recorded.block_loads.len())).unwrap();

          stopwatch::clone()
        })
      }
    };

    {
      let client = &client;
      let server = server.clone();
      view_thread(
        client,
        &mut || { view_updates0.lock().unwrap().pop_front() },
        &mut || { view_updates1.lock().unwrap().pop_front() },
        &mut |server_update| { server.talk.tell(&server_update) },
      );

      stopwatch::clone().print();
    }

    // View thread returned, so we got a quit event.
    *quit.lock().unwrap() = true;

    audio_thread.join();
    monitor_thread.join();

    let stopwatch = update_thread.join();

    stopwatch.print();
  }
}

fn connect_client(listen_url: &str, server: &server::T) -> client::T {
  // TODO: Consider using RPCs to solidify the request-response patterns.
  server.talk.tell(&protocol::ClientToServer::Init(listen_url.to_owned()));
  loop {
    match server.listen.wait() {
      protocol::ServerToClient::LeaseId(client_id) => {
        server.talk.tell(&protocol::ClientToServer::AddPlayer(client_id));
        let client_id = client_id;
        loop {
          match server.listen.wait() {
            protocol::ServerToClient::PlayerAdded(player_id, position) => {
              return client::new(client_id, player_id, position);
            },
            msg => {
              // Ignore other messages in the meantime.
              warn!("Ignoring: {:?}", msg);
            },
          }
        }
      },
      msg => {
        // Ignore other messages in the meantime.
        warn!("Ignoring: {:?}", msg);
      },
    }
  }
}

fn load_ambient_track() -> Track {
  let mut reader = hound::WavReader::open("Assets/rainforest_ambience-GlorySunz-1938133500.wav").unwrap();
  let data: Vec<f32> =
    reader.samples::<i16>()
    .map(|s| {
      s.unwrap() as f32 / 32768.0
    })
    .collect();
  Track::new(data)
}
