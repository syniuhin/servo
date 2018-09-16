/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![deny(unsafe_code)]

#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate enum_iterator;
extern crate ipc_channel;
#[macro_use]
extern crate log;
#[macro_use]
extern crate malloc_size_of;
#[macro_use]
extern crate malloc_size_of_derive;
#[macro_use]
extern crate servo_channel;
extern crate webrender_api;

mod background_hang_monitor;
pub mod constellation_msg;
