//! An image viewer example.
//!
//! This application implements a simple image viewer which is displayed using SHM buffers.

use std::{env, path::Path, process};

use smithay_client_toolkit::{
    compositor::{CompositorHandler, CompositorState},
    delegate_compositor, delegate_output, delegate_registry, delegate_shm, delegate_xdg_shell,
    delegate_xdg_window,
    output::{OutputHandler, OutputState},
    registry::{ProvidesRegistryState, RegistryState},
    shell::xdg::{
        window::{Window, WindowHandler, XdgWindowState},
        XdgShellHandler, XdgShellState,
    },
    shm::{pool::raw::RawPool, ShmHandler, ShmState},
};
use wayland_client::{
    protocol::{wl_buffer, wl_output, wl_shm, wl_surface},
    Connection, ConnectionHandle, Dispatch, QueueHandle,
};
use wayland_protocols::xdg_shell::client::xdg_surface;

fn main() {
    let path = match env::args_os().nth(1) {
        Some(v) => v,
        None => {
            println!("USAGE: ./image_viewer <PATH>");
            process::exit(1);
        }
    };

    // Open the image.
    //
    // The image crate will detect the file format which is used.
    let image = match image::open(&path) {
        Ok(image) => image,
        Err(err) => {
            println!("Failed to open image {}.", path.to_string_lossy());
            println!("Error was: {}", err);
            process::exit(1);
        }
    };

    // Convert the image to a format the compositor can display.
    //
    // The image crate uses a big endian order for the format code while Wayland is little endian.
    let image = image.to_rgba8();

    // Initialize connection to the compositor.

    let conn = match Connection::connect_to_env() {
        Ok(conn) => conn,
        Err(err) => {
            println!("{}", err);
            process::exit(1);
        }
    };

    // Create the event queue that the image viewer will dispatch events with.
    let mut queue = conn.new_event_queue();
    // And a handle to the event queue so we may create protocol objects.
    let qh = queue.handle();

    // Initialize the registry
    let registry = {
        let display = conn.handle().display();
        display.get_registry(&mut conn.handle(), &qh, ()).expect("could not create registry")
    };

    // Initialize protocol states to interface with the compositor.
    let protocols = ProtocolStates {
        registry: RegistryState::new(registry),
        compositor_state: CompositorState::new(),
        output_state: OutputState::new(),
        shm_state: ShmState::new(),
        xdg_shell_state: XdgShellState::new(),
        xdg_window_state: XdgWindowState::new(),
    };

    // Initialize the image viewer data.
    let mut image_viewer = ImageViewer { protocols, window_state: None, running: true, image };

    // Initial roundtrip, use two blocking dispatches
    queue.blocking_dispatch(&mut image_viewer).unwrap();
    queue.blocking_dispatch(&mut image_viewer).unwrap();

    /*
    Window initialization
    */

    // A surface is needed to create a window.
    let surface = image_viewer
        .protocols
        .compositor_state
        .create_surface(&mut conn.handle(), &qh)
        .expect("surface creation");

    let window = image_viewer
        .protocols
        .xdg_window_state
        .create_window(&mut conn.handle(), &qh, surface)
        .expect("window creation");

    // Indicate the window should not be smaller than the image.
    window.set_min_size(&mut conn.handle(), Some(image_viewer.image.dimensions()));

    // Set the window title to the name of the image.
    let title = Path::new(&path).file_name().unwrap().to_string_lossy();

    window.set_title(&mut conn.handle(), title);

    // Set the app id so multiple instances of the image viewer may be grouped together
    window.set_app_id(&mut conn.handle(), "io.github.smithay.client-toolkit.ImageViewerExample");

    // Finally, map the window.
    window.map(&mut conn.handle(), &qh);

    /*
    Buffer creation

    TODO: Use MultiPool based abstractions when complete.
    */

    // Create a pool large enough to hold the image.
    let len = image_viewer.image.width() * image_viewer.image.height() * 4;
    let mut pool = image_viewer
        .protocols
        .shm_state
        .new_raw_pool(len as usize, &mut conn.handle(), &qh, ())
        .expect("Pool");

    let buffer = pool
        .create_buffer(
            0,
            image_viewer.image.width() as i32,
            image_viewer.image.height() as i32,
            (image_viewer.image.width() * 4) as i32, // Size per row
            // Assume Argb8888 since all compositors must support said format with wl_shm
            wl_shm::Format::Argb8888,
            (),
            &mut conn.handle(),
            &qh,
        )
        .expect("buffer creation");

    image_viewer.window_state = Some(WindowState { window, initial_configure: true, pool, buffer });

    /*
    Main loop
    */

    // Application setup is complete. We can now enter the main loop
    loop {
        if !image_viewer.running {
            println!("exiting");
            break;
        }

        queue.blocking_dispatch(&mut image_viewer).unwrap();
    }
}

/// State objects used by the image viewer to interface with the compositor.
///
/// This type contains a few state objects which are considered as "delegate types". These types receive
/// handle specific Wayland protocols.
///
/// The fields of this type could be part of [`ImageViewer`] if you wish, but separation is done for clarity.
struct ProtocolStates {
    registry: RegistryState,
    compositor_state: CompositorState,
    output_state: OutputState,
    shm_state: ShmState,
    xdg_shell_state: XdgShellState,
    xdg_window_state: XdgWindowState,
}

/// State of the application window.
struct WindowState {
    window: Window,
    /// Whether the window is receiving the initial configure.
    initial_configure: bool,
    pool: RawPool,
    buffer: wl_buffer::WlBuffer,
}

/// Data associated with the image viewer.
///
/// This type is available to access and modify data when Wayland events come in from the compositor.
struct ImageViewer {
    /// Protocol state objects used to interface with the compositor.
    protocols: ProtocolStates,

    /// Window state.
    ///
    /// The image viewer has a single window. This is an [`Option`] since window initialization happens after
    /// the application launches.
    window_state: Option<WindowState>,

    /// Whether the application should continue to run.
    running: bool,

    /// The image to display.
    image: image::RgbaImage,
}

// Next we need to delegate handling of specific wayland protocols to the inner delegate types in ProtocolStates.
// This is a two step process:
// 1. Use the `delegate` macros to implement handling of specific protocols to the ImageViewer type.
// 2. Implement some traits to handle the specific features.

// delegate_output adds the requirement "impl OutputHandler for ImageViewer"
//
// We must implement this trait but don't actually use any of it's functionality.
delegate_output!(ImageViewer);

impl OutputHandler for ImageViewer {
    fn output_state(&mut self) -> &mut OutputState {
        &mut self.protocols.output_state
    }

    fn new_output(
        &mut self,
        _conn: &mut ConnectionHandle,
        _qh: &QueueHandle<Self>,
        _output: wl_output::WlOutput,
    ) {
    }

    fn update_output(
        &mut self,
        _conn: &mut ConnectionHandle,
        _qh: &QueueHandle<Self>,
        _output: wl_output::WlOutput,
    ) {
    }

    fn output_destroyed(
        &mut self,
        _conn: &mut ConnectionHandle,
        _qh: &QueueHandle<Self>,
        _output: wl_output::WlOutput,
    ) {
    }
}

// delegate_compositor adds the requirement "impl CompositorHandler for ImageViewer"
delegate_compositor!(ImageViewer);

impl CompositorHandler for ImageViewer {
    fn compositor_state(&mut self) -> &mut CompositorState {
        &mut self.protocols.compositor_state
    }

    fn scale_factor_changed(
        &mut self,
        _conn: &mut ConnectionHandle,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _new_factor: i32,
    ) {
        // Don't particularly care about scale factor in this example.
    }

    fn frame(
        &mut self,
        _conn: &mut ConnectionHandle,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _time: u32,
    ) {
        // This example does not utilize frame callbacks.
        // If you need to continuously draw, this may be needed.
    }
}

// delegate_shm adds the requirement "impl ShmHandler for ImageViewer"
delegate_shm!(ImageViewer);

impl ShmHandler for ImageViewer {
    fn shm_state(&mut self) -> &mut ShmState {
        &mut self.protocols.shm_state
    }
}

// delegate_xdg_shell adds the requirement "impl XdgShellHandler for ImageViewer"
delegate_xdg_shell!(ImageViewer);

impl XdgShellHandler for ImageViewer {
    fn xdg_shell_state(&mut self) -> &mut XdgShellState {
        &mut self.protocols.xdg_shell_state
    }

    fn configure(
        &mut self,
        conn: &mut ConnectionHandle,
        qh: &QueueHandle<Self>,
        _surface: &xdg_surface::XdgSurface,
    ) {
        // Since there is only one window, we can ignore the surface.
        //
        // If there are multiple windows, you would use the surface to identify the window.
        let window_state = self.window_state.as_mut().unwrap();

        if window_state.initial_configure {
            window_state.initial_configure = false;
        }

        // Get the configure data.
        let configure = window_state.window.configure().unwrap();

        let image_dimensions = self.image.dimensions();

        // If the size is None, meaning we may choose any size, set the window size to the image dimensions.
        let size =
            if configure.new_size == None { image_dimensions } else { configure.new_size.unwrap() };

        todo!("finish this")
    }
}

// delegate_xdg_window adds the requirement "impl WindowHandler for ImageViewer"
delegate_xdg_window!(ImageViewer);

impl WindowHandler for ImageViewer {
    fn xdg_window_state(&mut self) -> &mut XdgWindowState {
        &mut self.protocols.xdg_window_state
    }

    fn request_close_window(
        &mut self,
        conn: &mut ConnectionHandle,
        qh: &QueueHandle<Self>,
        window: &Window,
    ) {
        // We were told our window has been closed.
        self.running = false;
    }
}

// Finally we need to delegate registry handling to all the delegate types we use.
//
// We need to use a delegate macro to delegate registry handling and then implement AsMut<RegistryState>
delegate_registry!(ImageViewer: [
    CompositorState,
    OutputState,
    ShmState,
    XdgShellState,
    XdgWindowState,
]);

impl ProvidesRegistryState for ImageViewer {
    fn registry(&mut self) -> &mut RegistryState {
        &mut self.protocols.registry
    }
}

// TODO: Pending changes regarding WlBuffer on pools
impl Dispatch<wl_buffer::WlBuffer> for ImageViewer {
    type UserData = ();

    fn event(
        &mut self,
        _proxy: &wl_buffer::WlBuffer,
        _event: wl_buffer::Event,
        _data: &Self::UserData,
        _conn: &mut ConnectionHandle,
        _qh: &QueueHandle<Self>,
    ) {
    }
}
