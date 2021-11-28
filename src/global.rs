//! Utilities for managing globals.
//!
//! This crate provides abstractions useful for components of smithay's client toolkit which request some\
//! globals.

use std::ops::Range;

use wayland_client::{
    ConnectionHandle, DataInit, DelegateDispatch, DelegateDispatchBase, Dispatch, Proxy,
    QueueHandle,
};

use crate::ProvidesGlobal;

/// Allows a component of smithay's client toolkit to request some global and get notified when the global is
/// created and removed.
pub trait RequestGlobal<I: Proxy + 'static> {
    /// A range of acceptable versions of the global to bind.
    ///
    /// Generally the highest version in this range will be bound unless some other object requesting the
    /// global has a lower maximum version.
    fn version(&self) -> Range<u32>;

    /// The requested global has been bound.
    ///
    /// It is guaranteed the version of the global is one of the versions specified in [`RequestGlobal::version`].
    fn new_global(&mut self, name: u32, global: I);

    /// Invoked when a global is destroyed.
    ///
    /// This is invoked for the removal of all globals, you should ensure the name matches before assuming the
    /// global is dead.
    fn remove_global(&mut self, name: u32);
}

/// A container which automatically binds a specified global in a specified version range.
///
/// This container is suitable for globals which do not dispatch any events and only one exists.
///
/// This type may also be delegate in the [`sctk`](crate::sctk) macro or used in the [`delegate_dispatch`](crate::reexports::client::delegate_dispatch)
/// macro.
#[derive(Debug)]
pub struct NoEventGlobal<I: Proxy + 'static> {
    inner: Option<I>,
    name: Option<u32>,
    version: Range<u32>,
    destroyed: bool,
}

impl<I: Proxy + 'static> NoEventGlobal<I> {
    /// Creates a new container, using the specified versions as the allowed range.
    pub fn new(version: Range<u32>) -> Self {
        Self { inner: None, name: None, version, destroyed: false }
    }

    /// Returns whether the global has been destroyed.
    pub fn destroyed(&self) -> bool {
        self.destroyed
    }
}

impl<I: Proxy + Clone + 'static> NoEventGlobal<I> {
    /// Returns the global.
    pub fn get(&self) -> Option<I> {
        self.inner.clone()
    }
}

impl<I: Proxy + 'static> DelegateDispatchBase<I> for NoEventGlobal<I> {
    type UserData = ();
}

impl<I: Proxy + 'static, D> DelegateDispatch<I, D> for NoEventGlobal<I>
where
    // The type that delegates to us must be able to dispatch the global, which is satisfied by delegating to us.
    D: Dispatch<I, UserData = Self::UserData>
        // The type which dispatches to us must also be able to provide the global if available.
        + ProvidesGlobal<I>,
{
    fn event(
        &mut self,
        _: &I,
        _: I::Event,
        _: &Self::UserData,
        _: &mut ConnectionHandle,
        _: &QueueHandle<D>,
        _: &mut DataInit<'_>,
    ) {
        unreachable!("NoEventGlobal<_> should never receive an event")
    }
}

impl<I: Proxy + 'static> RequestGlobal<I> for NoEventGlobal<I> {
    fn version(&self) -> Range<u32> {
        self.version.clone()
    }

    fn new_global(&mut self, _name: u32, global: I) {
        if self.inner.is_some() && !self.destroyed {
            // TODO: Warn in log about second global being advertised
        } else {
            self.inner = Some(global);
        }
    }

    fn remove_global(&mut self, name: u32) {
        if self.name == Some(name) {
            self.inner = None;
            self.destroyed = true;
        }
    }
}
