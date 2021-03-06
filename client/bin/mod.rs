//! Client binary

#![deny(missing_docs)]
#![deny(warnings)]
#![feature(global_allocator)]
#![feature(allocator_api)]

extern crate env_logger;
#[macro_use]
extern crate log;

extern crate client_lib;

use std::borrow::Borrow;
use std::env;

// Use the system allocator because if we use jemalloc then we deadlock pretty quickly in a jemalloc mutex.
#[global_allocator]
static ALLOCATOR: std::heap::System = std::heap::System;

fn main() {
  env_logger::init().unwrap();

  let mut args = env::args();
  args.next().unwrap();
  let listen_url = args.next().unwrap_or_else(|| String::from("ipc:///tmp/client.ipc"));
  let server_url = args.next().unwrap_or_else(|| String::from("ipc:///tmp/server.ipc"));
  assert!(args.next().is_none());

  info!("Sending to {}.", server_url);
  info!("Listening on {}.", listen_url);

  client_lib::run(listen_url.borrow(), server_url.borrow());
}
