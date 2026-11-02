/*!
Snake for the ZHAW Lumatrix.

[`game`] holds the rules and is free of hardware types, so it compiles and
runs its tests on the host. Everything else drives the board and only builds
for the target.
*/

#![cfg_attr(target_os = "none", no_std)]

pub mod game;

#[cfg(target_os = "none")]
mod board;

#[cfg(target_os = "none")]
pub mod input;

#[cfg(target_os = "none")]
pub mod render;

#[cfg(target_os = "none")]
pub use board::*;
