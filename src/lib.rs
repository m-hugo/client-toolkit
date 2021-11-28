#![warn(missing_docs, missing_debug_implementations)]
#![allow(clippy::new_without_default)]

//! TODO: Docs

use wayland_client::Proxy;

/// Re-exports of some crates, for convenience.
pub mod reexports {
    #[cfg(feature = "calloop")]
    pub use calloop;
    pub use wayland_client as client;
    pub use wayland_protocols as protocols;
}

pub mod global;
pub mod seat;

/// A trait which asserts the type Wayland events are dispatched to may provide globals of the specified type.
///
/// Some parts of Smithay's client toolkit may require this trait is implemented in order to be accessible.
///
/// Unless you want to, you should use the [`sctk`] macro to avoid implementing this yourself.
pub trait ProvidesGlobal<I: Proxy + 'static> {}

/// The sctk macro.
///
/// This macro exists for the purpose of linking together the components provided in Smithay's client toolkit.
///
/// The macro does a few things:
/// - Implements [`Dispatch`](crate::reexports::client::Dispatch) for
/// [`WlRegistry`](crate::reexports::client::protocol::wl_registry::WlRegistry) on specified type.
/// - Propagates all global creation and destruction to the types which implement [`RequestGlobal`](crate::global::RequestGlobal).
/// - Implements [`ProvidesGlobal`] on the specified type for all globals specified in the macro.
/// - Applies the [`delegate_dispatch`](crate::reexports::client::delegate_dispatch) macro to all delegates specified
/// in the macro.
///
/// TODO: Example usage and explanation
#[macro_export]
macro_rules! sctk {
    (
        // The type that sctk should delegate to.
        $ty: ty,
        globals = [
            // Provides global => (to the) fields
            $({$provides_global: ty => [ $($field: ident),* $(,)? ]}),* $(,)?
        ],

        // Declares how to delegate events from some interface to some inner type.
        delegates = [
            $($delegate_to_this: ty ; [$($for_interfaces: ty),*] => $to_field: ident),* $(,)?
        ],
    ) => {
        // Implement ProvidesGlobal for every declared global type.
        $crate::sctk!(@impl_provides_global $ty => [$($provides_global),*]);

        // Use delegate_dispatch! for each global that is provided so D may dispatch to each delegate.
        $crate::sctk!(@delegate_dispatch $ty, [ $($delegate_to_this ; [$($for_interfaces),*] => $to_field),* ]);

        // Implement Delegate for handling the registry.
        impl $crate::reexports::client::Dispatch<$crate::reexports::client::protocol::wl_registry::WlRegistry> for $ty {
            type UserData = ();

            fn event(
                &mut self,
                registry: &$crate::reexports::client::protocol::wl_registry::WlRegistry,
                event: $crate::reexports::client::protocol::wl_registry::Event,
                _data: &(),
                cx: &mut $crate::reexports::client::ConnectionHandle,
                qh: &$crate::reexports::client::QueueHandle<Self>,
                _init: &mut $crate::reexports::client::DataInit<'_>,
            ) {
                // Something was either advertised or destroyed from the registry.
                // We process this using a large match statement for the thing being advertised.

                match event {
                    $crate::reexports::client::protocol::wl_registry::Event::Global { name, interface, version } => {
                        // We cannot match on the $global::interface().name because the return value of `interface()` is not const.
                        $(
                            let interface_name = <$provides_global as $crate::reexports::client::Proxy>::interface().name;

                            if &interface == interface_name {
                                // Determine which version of the global should instantiated.
                                let ranges = [
                                    $(
                                        $crate::global::RequestGlobal::version(&self.$field)
                                    ),*
                                ];

                                let acceptable_range = ranges
                                    .iter()
                                    .cloned()
                                    .reduce(|a, b| {
                                        // Find the upper bound to the start
                                        let start = u32::max(a.start, b.start);

                                        // And the lower bound of the end
                                        let end = u32::min(a.end, b.end);

                                        start..end
                                    })
                                    // Ensure the end of the range is greater than the start.
                                    .and_then(|range| if range.start > range.end { None } else { Some(range) })
                                    .expect("Requested version range never intersects");

                                // Is the global version high enough to instantiate?
                                if acceptable_range.start <= version {
                                    // Yes, bind the global.
                                    let global = registry
                                        // We pick the version of the global advertised if it is lower than the max.
                                        // If the acceptable range is larger, we pick the highest version we can request.
                                        .bind::<$provides_global, _>(cx, name, u32::min(version, acceptable_range.end - 1), qh, ())
                                        .expect("Failed to create global");

                                    // Notify everything requesting the global that the global now exists
                                    $(
                                        $crate::global::RequestGlobal::<$provides_global>::new_global(&mut self.$field, name, global.clone());
                                    )*
                                }
                            }
                        )*
                    },

                    $crate::reexports::client::protocol::wl_registry::Event::GlobalRemove { name } => {
                        // TODO: This needs testing.
                        $($(
                            $crate::global::RequestGlobal::remove_global(&mut self.$field, name);
                        )*)*
                    },

                    _ => unreachable!("wl_registry is frozen"),
                }
            }
        }
    };

    // Internal rule to implement ProvidesGlobal<I> for each provided global.
    (
        @impl_provides_global
        $ty: ty => [$($provides_global: ty),*]
    ) => {
        $(
            impl $crate::ProvidesGlobal<$provides_global> for $ty {}
        )*
    };

    // Internal rule to invoke `delegate_dispatch!` for
    (
        @delegate_dispatch
        $for: ty, [
            $($delegate_to_this: ty ; [$($for_interfaces: ty),*] => $to_field: ident),*
        ]
    ) => {
        $(
            $crate::delegate_dispatch_2!($for => $delegate_to_this ; [$($for_interfaces),*] => self.$to_field);
        )*
    };
}

/// TODO: Replace this wil wayland-rs delegate_dispatch when it supports fields.
#[macro_export]
macro_rules! delegate_dispatch_2 {
    // Delegate implementation to another type using a conversion function from the from type.
    ($dispatch_from:ty => $dispatch_to:ty ; [$($interface:ty),*] => $convert:ident) => {
        $(
            impl wayland_client::Dispatch<$interface> for $dispatch_from {
                type UserData = <$dispatch_to as $crate::DelegateDispatchBase<$interface>>::UserData;
                fn event(
                    &mut self,
                    proxy: &$interface,
                    event: <$interface as wayland_client::Proxy>::Event,
                    data: &Self::UserData,
                    cxhandle: &mut wayland_client::ConnectionHandle,
                    qhandle: &wayland_client::QueueHandle<Self>,
                    init: &mut wayland_client::DataInit<'_>,
                ) {
                    <$dispatch_to as wayland_client::DelegateDispatch<$interface, Self>>::event(&mut self.$convert(), proxy, event, data, cxhandle, qhandle, init)
                }
            }
        )*
    };

    // Delegate implementation to another type using a field owned by the from type.
    ($dispatch_from:ty => $dispatch_to:ty ; [$($interface:ty),*] => self.$field:ident) => {
        $(
            impl wayland_client::Dispatch<$interface> for $dispatch_from {
                type UserData = <$dispatch_to as wayland_client::DelegateDispatchBase<$interface>>::UserData;

                fn event(
                    &mut self,
                    proxy: &$interface,
                    event: <$interface as wayland_client::Proxy>::Event,
                    data: &Self::UserData,
                    cxhandle: &mut wayland_client::ConnectionHandle,
                    qhandle: &wayland_client::QueueHandle<Self>,
                    init: &mut wayland_client::DataInit<'_>,
                ) {
                    <$dispatch_to as wayland_client::DelegateDispatch<$interface, Self>>::event(&mut self.$field, proxy, event, data, cxhandle, qhandle, init)
                }
            }
        )*
    };
}
