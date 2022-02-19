//! Smithay's Client Toolkit
//!
//! Smithay's Client Toolkit (SCTK) provides various utilities and abstractions for communicating with various
//! Wayland compositors.
//!
//! # Delegates
//!
//! SCTK provides "delegate types" which implement various Wayland protocols. These delegate types also have a
//! corresponding macro to implement the [`Dispatch`](wayland_client::Dispatch) trait on a data type to
//! statically assert some part of the Wayland protocol is handled by an event queue.
//!
//! The use of delegate types allows SCTK to provide abstractions to communicate with specific parts of a
//! Wayland compositor in a modular fashion.
//!
//! For more information about delegates, see the `wayland-client` documentation.
//!
//! # Protocol abstractions
//!
//! SCTK provides protocol abstractions for commonly used Wayland protocols in multiple modules:
//!
//! ## [`compositor`]
//!
//! Helpers to assist with creation of [`WlSurface`](wayland_client::protocol::wl_surface::WlSurface)s and
//! basic frame notification.
//!
//! ## [`data_device`]
//!
//! Helpers to handle data device related actions.
//!
//! These helpers assist with implementing clipboards and drag-and-drop support.
//!
//! ## [`output`]
//!
//! Handlers to notify clients about the existence of outputs and their properties.
//!
//! ## [`registry`]
//!
//! Helpers to enumerate globals provided by a Wayland compositor.
//!
//! The helpers here are used to forward events regarding creation and destruction of globals to other
//! delegate types.
//!
//! ## [`seat`]
//!
//! Abstractions and helpers for input devices, such as a keyboard and pointer.
//!
//! This module contains utilities for setting the cursor image of the pointer and loading keymaps from the
//! compositor.
//!
//! ## [`shell`]
//!
//! Abstractions over various Wayland "shells". A shell refers to a type of surface and specific semantics
//! associated with a surface.
//!
//! This is where abstractions such as a [`Window`](shell::xdg::window::Window) are located.
//!
//! ## [`shm`]
//!
//! Helpers to allocate shared memory buffers on the client.
//!
//! Shared memory buffers are a simple way to send a large number of pixels to the compositor for rendering.
//!
//! # Event Loops
//!
//! SCTK provides integration with [`calloop`](https://crates.io/crates/calloop) to provide an event loop
//! abstraction.
//!
//! Most Wayland apps will need to handle more event sources than a single Wayland connection. This is
//! necessary to handle things such as keyboard repetition, copy-paste and animated cursors.
//!
//! [`WaylandSource`](event_loop::WaylandSource) is an adapter to insert a Wayland [`EventQueue`](wayland_client::EventQueue)
//! into a calloop [`EventLoop`](calloop::EventLoop). Some modules of SCTK will provide other event sources
//! that need to be inserted into the event loop in order to function properly.
//!
//! The features provided in the [`event loop`](event_loop) module are guarded by the `calloop` feature and
//! are provided by default.

#![warn(
//    missing_docs, // Commented out for now so the project isn't all yellow.
    missing_debug_implementations
)]
#![allow(clippy::new_without_default)]

/// Re-exports of some crates, for convenience.
pub mod reexports {
    #[cfg(feature = "calloop")]
    pub use calloop;
    pub use wayland_client as client;
    pub use wayland_protocols as protocols;
}

pub mod compositor;
#[cfg(feature = "calloop")]
pub mod event_loop;
pub mod output;
pub mod registry;
pub mod seat;
pub mod shell;
pub mod shm;
