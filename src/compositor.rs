use std::sync::{
    atomic::{AtomicBool, AtomicI32, Ordering},
    Mutex,
};

use wayland_backend::client::InvalidId;
use wayland_client::{
    protocol::{
        wl_callback, wl_compositor, wl_output, wl_subcompositor, wl_subsurface, wl_surface,
    },
    ConnectionHandle, DelegateDispatch, DelegateDispatchBase, Dispatch, Proxy, QueueHandle,
};

use crate::{
    output::OutputData,
    registry::{ProvidesRegistryState, RegistryHandler},
};

/// An error caused by creating a surface.
#[derive(Debug, thiserror::Error)]
pub enum SurfaceError {
    /// The compositor global is not available.
    #[error("the compositor global is not available")]
    MissingCompositorGlobal,

    /// Protocol error.
    #[error(transparent)]
    Protocol(#[from] InvalidId),
}

#[derive(Debug, thiserror::Error)]
pub enum SubsurfaceError {
    /// The subcompositor global is not available.
    #[error("the subcompositor global is not available")]
    MissingSubcompositorGlobal,

    /// Protocol error.
    #[error(transparent)]
    Protocol(#[from] InvalidId),
}

pub trait CompositorHandler: Sized {
    fn compositor_state(&mut self) -> &mut CompositorState;

    /// The surface has either been moved into or out of an output and the output has a different scale factor.
    fn scale_factor_changed(
        &mut self,
        conn: &mut ConnectionHandle,
        qh: &QueueHandle<Self>,
        surface: &wl_surface::WlSurface,
        new_factor: i32,
    );

    /// A frame callback has been completed.
    ///
    /// This function will be called after sending a [`WlSurface::frame`](wl_surface::WlSurface::frame) request
    /// and committing the surface.
    fn frame(
        &mut self,
        conn: &mut ConnectionHandle,
        qh: &QueueHandle<Self>,
        surface: &wl_surface::WlSurface,
        time: u32,
    );
}

#[derive(Debug)]
pub struct CompositorState {
    wl_compositor: Option<(u32, wl_compositor::WlCompositor)>,
    wl_subcompositor: Option<(u32, wl_subcompositor::WlSubcompositor)>,
}

impl CompositorState {
    pub fn new() -> CompositorState {
        CompositorState { wl_compositor: None, wl_subcompositor: None }
    }

    pub fn create_surface<D>(
        &self,
        conn: &mut ConnectionHandle,
        qh: &QueueHandle<D>,
    ) -> Result<wl_surface::WlSurface, SurfaceError>
    where
        D: Dispatch<wl_surface::WlSurface, UserData = SurfaceData> + 'static,
    {
        let (_, compositor) =
            self.wl_compositor.as_ref().ok_or(SurfaceError::MissingCompositorGlobal)?;

        let surface = compositor.create_surface(
            conn,
            qh,
            SurfaceData {
                scale_factor: AtomicI32::new(1),
                outputs: Mutex::new(vec![]),
                has_role: AtomicBool::new(false),
            },
        )?;

        Ok(surface)
    }

    pub fn create_subsurface<D>(
        &self,
        conn: &mut ConnectionHandle,
        qh: &QueueHandle<D>,
        parent: &wl_surface::WlSurface,
        surface: &wl_surface::WlSurface,
    ) -> Result<Subsurface, SubsurfaceError>
    where
        D: Dispatch<wl_subsurface::WlSubsurface, UserData = ()> + 'static,
    {
        let (_, subcompositor) =
            self.wl_subcompositor.as_ref().ok_or(SubsurfaceError::MissingSubcompositorGlobal)?;

        let subsurface = subcompositor.get_subsurface(conn, surface, parent, qh, ())?;

        Ok(Subsurface { subsurface, parent: parent.clone(), surface: surface.clone() })
    }
}

/// Data associated with a [`WlSurface`](wl_surface::WlSurface).
#[derive(Debug)]
pub struct SurfaceData {
    /// The scale factor of the output with the highest scale factor.
    pub(crate) scale_factor: AtomicI32,

    /// The outputs the surface is currently inside.
    pub(crate) outputs: Mutex<Vec<wl_output::WlOutput>>,

    /// Whether the surface has a role object.
    pub(crate) has_role: AtomicBool,
}

#[derive(Debug)]
pub struct Subsurface {
    subsurface: wl_subsurface::WlSubsurface,
    parent: wl_surface::WlSurface,
    surface: wl_surface::WlSurface,
}

impl Subsurface {
    pub fn parent(&self) -> &wl_surface::WlSurface {
        &self.parent
    }

    pub fn wl_surface(&self) -> &wl_surface::WlSurface {
        &self.surface
    }

    pub fn wl_subsurface(&self) -> &wl_subsurface::WlSubsurface {
        &self.subsurface
    }

    pub fn destroy(self, conn: &mut ConnectionHandle) {
        self.subsurface.destroy(conn);
    }
}

#[macro_export]
macro_rules! delegate_compositor {
    ($ty: ty) => {
        type __WlCompositor = $crate::reexports::client::protocol::wl_compositor::WlCompositor;
        type __WlSubcompositor = $crate::reexports::client::protocol::wl_subcompositor::WlSubcompositor;
        type __WlSubsurface = $crate::reexports::client::protocol::wl_subsurface::WlSubsurface;
        type __WlSurface = $crate::reexports::client::protocol::wl_surface::WlSurface;
        type __WlCallback = $crate::reexports::client::protocol::wl_callback::WlCallback;

        $crate::reexports::client::delegate_dispatch!($ty:
            [
                __WlCompositor,
                __WlSubcompositor,
                __WlSubsurface,
                __WlSurface,
                __WlCallback
            ] => $crate::compositor::CompositorState
        );
    };
}

impl DelegateDispatchBase<wl_surface::WlSurface> for CompositorState {
    type UserData = SurfaceData;
}

impl<D> DelegateDispatch<wl_surface::WlSurface, D> for CompositorState
where
    D: Dispatch<wl_surface::WlSurface, UserData = Self::UserData>
        + Dispatch<wl_output::WlOutput, UserData = OutputData>
        + CompositorHandler,
{
    fn event(
        state: &mut D,
        surface: &wl_surface::WlSurface,
        event: wl_surface::Event,
        data: &Self::UserData,
        conn: &mut ConnectionHandle,
        qh: &QueueHandle<D>,
    ) {
        let mut outputs = data.outputs.lock().unwrap();

        match event {
            wl_surface::Event::Enter { output } => {
                outputs.push(output);
            }

            wl_surface::Event::Leave { output } => {
                outputs.retain(|o| o != &output);
            }

            _ => unreachable!(),
        }

        // Compute the new max of the scale factors for all outputs this surface is displayed on.
        let current = data.scale_factor.load(Ordering::SeqCst);

        let largest_factor = outputs
            .iter()
            .filter_map(|output| output.data::<OutputData>().map(OutputData::scale_factor))
            .reduce(i32::max);

        // Drop the mutex before we send of any events.
        drop(outputs);

        // If no scale factor is found, because the surface has left it's only output, do not change the scale factor.
        if let Some(factor) = largest_factor {
            data.scale_factor.store(factor, Ordering::SeqCst);

            if current != factor {
                state.scale_factor_changed(conn, qh, surface, factor);
            }
        }
    }
}

impl DelegateDispatchBase<wl_compositor::WlCompositor> for CompositorState {
    type UserData = ();
}

impl<D> DelegateDispatch<wl_compositor::WlCompositor, D> for CompositorState
where
    D: Dispatch<wl_compositor::WlCompositor, UserData = Self::UserData> + CompositorHandler,
{
    fn event(
        _: &mut D,
        _: &wl_compositor::WlCompositor,
        _: wl_compositor::Event,
        _: &(),
        _: &mut ConnectionHandle,
        _: &QueueHandle<D>,
    ) {
        unreachable!("wl_compositor has no events")
    }
}

impl DelegateDispatchBase<wl_subcompositor::WlSubcompositor> for CompositorState {
    type UserData = ();
}

impl<D> DelegateDispatch<wl_subcompositor::WlSubcompositor, D> for CompositorState
where
    D: Dispatch<wl_subcompositor::WlSubcompositor, UserData = Self::UserData>,
{
    fn event(
        _: &mut D,
        _: &wl_subcompositor::WlSubcompositor,
        _: wl_subcompositor::Event,
        _: &(),
        _: &mut ConnectionHandle,
        _: &QueueHandle<D>,
    ) {
        unreachable!("wl_compositor has no events")
    }
}

impl DelegateDispatchBase<wl_subsurface::WlSubsurface> for CompositorState {
    type UserData = ();
}

impl<D> DelegateDispatch<wl_subsurface::WlSubsurface, D> for CompositorState
where
    D: Dispatch<wl_subsurface::WlSubsurface, UserData = Self::UserData>,
{
    fn event(
        _: &mut D,
        _: &wl_subsurface::WlSubsurface,
        _: wl_subsurface::Event,
        _: &Self::UserData,
        _: &mut ConnectionHandle,
        _: &QueueHandle<D>,
    ) {
        unreachable!("wl_subsurface has no events")
    }
}

impl DelegateDispatchBase<wl_callback::WlCallback> for CompositorState {
    type UserData = wl_surface::WlSurface;
}

impl<D> DelegateDispatch<wl_callback::WlCallback, D> for CompositorState
where
    D: Dispatch<wl_callback::WlCallback, UserData = Self::UserData> + CompositorHandler,
{
    fn event(
        state: &mut D,
        _: &wl_callback::WlCallback,
        event: wl_callback::Event,
        surface: &Self::UserData,
        conn: &mut ConnectionHandle,
        qh: &QueueHandle<D>,
    ) {
        match event {
            wl_callback::Event::Done { callback_data } => {
                state.frame(conn, qh, surface, callback_data);
            }

            _ => unreachable!(),
        }
    }
}

impl<D> RegistryHandler<D> for CompositorState
where
    D: Dispatch<wl_compositor::WlCompositor, UserData = ()>
        + Dispatch<wl_subcompositor::WlSubcompositor, UserData = ()>
        + CompositorHandler
        + ProvidesRegistryState
        + 'static,
{
    fn new_global(
        state: &mut D,
        conn: &mut ConnectionHandle,
        qh: &QueueHandle<D>,
        name: u32,
        interface: &str,
        version: u32,
    ) {
        match interface {
            "wl_compositor" => {
                let compositor = state
                    .registry()
                    .bind_cached::<wl_compositor::WlCompositor, _, _, _>(conn, qh, name, || {
                        (u32::min(version, 4), ())
                    })
                    .expect("Failed to bind global");

                state.compositor_state().wl_compositor = Some((name, compositor));
            }

            "wl_subcompositor" => {
                let subcompositor = state
                    .registry()
                    .bind_cached::<wl_subcompositor::WlSubcompositor, _, _, _>(
                        conn,
                        qh,
                        name,
                        || (1, ()),
                    )
                    .expect("Failed to bind global");

                state.compositor_state().wl_subcompositor = Some((name, subcompositor));
            }

            _ => (),
        }
    }

    fn remove_global(state: &mut D, _conn: &mut ConnectionHandle, _qh: &QueueHandle<D>, name: u32) {
        if state
            .compositor_state()
            .wl_compositor
            .as_ref()
            .map(|(compositor_name, _)| *compositor_name == name)
            .unwrap_or(false)
        {
            state.compositor_state().wl_compositor.take();
        }
    }
}
