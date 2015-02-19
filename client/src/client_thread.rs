//! The client's "main" thread.

use client::{Client, LOD_THRESHOLDS};
use client_update::{ViewToClient, apply_view_to_client, apply_server_to_client};
use common::block_position::BlockPosition;
use common::communicate::{ClientToServer, ServerToClient};
use common::cube_shell::cube_diff;
use common::lod::LODIndex;
use common::process_events::process_channel;
use common::stopwatch::TimerSet;
use common::surroundings_loader::{LODChange, SurroundingsLoader};
use common::terrain_block::LOD_QUALITY;
use std::iter::range_inclusive;
use std::num;
use std::old_io::timer;
use std::sync::mpsc::{Sender, Receiver};
use std::time::duration::Duration;
use terrain_buffers;
use view_update::ClientToView;
use view_update::ClientToView::*;

#[allow(missing_docs)]
pub fn client_thread(
  my_url: String,
  ups_from_server: &Receiver<ServerToClient>,
  ups_to_server: &Sender<ClientToServer>,
  ups_from_view: &Receiver<ViewToClient>,
  ups_to_view: &Sender<ClientToView>,
) {
  let timers = TimerSet::new();
  let timers = &timers;

  ups_to_server.send(ClientToServer::Init(my_url)).unwrap();

  let client = Client::new();

  let mut surroundings_loader = {
    let mut load_distance = load_distance(terrain_buffers::POLYGON_BUDGET as i32);

    // TODO: Remove this once our RAM usage doesn't skyrocket with load distance.
    let max_load_distance = 10;
    if load_distance > max_load_distance {
      info!("load_distance {} capped at {}", load_distance, max_load_distance);
      load_distance = max_load_distance;
    } else {
      info!("load_distance {}", load_distance);
    }

    SurroundingsLoader::new(
      load_distance,
      Box::new(move |last, cur| {
        let mut vec = Vec::new();
        for &r in LOD_THRESHOLDS.iter() {
          vec.push_all(cube_diff(last, cur, r).as_slice());
        }
        vec.push_all(cube_diff(last, cur, load_distance).as_slice());
        vec
      }),
    )
  };

  loop {
    let quit =
      !process_channel(
        ups_from_view,
        |update| {
          apply_view_to_client(update, ups_to_server)
        },
      );
    if quit {
      break;
    }

    process_channel(
      ups_from_server,
      |update| {
        apply_server_to_client(update, &client, &ups_to_view);
        true
      },
    );

    let block_position = BlockPosition::from_world_position(&client.player_position.lock().unwrap().clone());

    surroundings_loader.update(
      block_position,
      |lod_change| {
        match lod_change {
          LODChange::Load(block_position, distance) => {
            timers.time("request_block", || {
              let lod = lod_index(distance);
              ups_to_server.send(ClientToServer::RequestBlock(block_position, lod)).unwrap();
            });
          },
          LODChange::Unload(block_position) => {
            client.loaded_blocks
              .lock().unwrap()
              .remove(&block_position)
              // If it wasn't loaded, don't unload anything.
              .map(|(block, prev_lod)| {
                timers.time("remove_block", || {
                  for id in block.ids.iter() {
                    ups_to_view.send(RemoveTerrain(*id)).unwrap();
                  }

                  ups_to_view.send(RemoveBlockData(block_position, prev_lod)).unwrap();
                });
              });
          },
        };
      },
    );

    timer::sleep(Duration::milliseconds(0));
  }

  timers.print();

  debug!("client exiting.");
}

fn lod_index(distance: i32) -> LODIndex {
  assert!(distance >= 0);
  let mut lod = 0;
  while
    lod < LOD_THRESHOLDS.len()
    && LOD_THRESHOLDS[lod] < distance
  {
    lod += 1;
  }
  LODIndex(num::cast(lod).unwrap())
}

fn load_distance(mut polygon_budget: i32) -> i32 {
  // TODO: This should try to account for VRAM not used on a per-poly basis.

  let mut load_distance = 0;
  let mut prev_threshold = 0;
  let mut prev_square = 0;
  for (&threshold, &quality) in LOD_THRESHOLDS.iter().zip(LOD_QUALITY.iter()) {
    let polygons_per_block = (quality * quality * 4) as i32;
    for i in range_inclusive(prev_threshold, threshold) {
      let i = 2 * i + 1;
      let square = i * i;
      let polygons_in_layer = (square - prev_square) * polygons_per_block;
      polygon_budget -= polygons_in_layer;
      if polygon_budget < 0 {
        break;
      }

      load_distance += 1;
      prev_square = square;
    }
    prev_threshold = threshold + 1;
  }

  let mut width = 2 * prev_threshold + 1;
  loop {
    let square = width * width;
    // The "to infinity and beyond" quality.
    let quality = LOD_QUALITY[LOD_THRESHOLDS.len()];
    let polygons_per_block = (quality * quality * 4) as i32;
    let polygons_in_layer = (square - prev_square) * polygons_per_block;
    polygon_budget -= polygons_in_layer;

    if polygon_budget < 0 {
      break;
    }

    width += 2;
    load_distance += 1;
    prev_square = square;
  }

  load_distance
}
