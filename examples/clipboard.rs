use std::error::Error;

use smithay_client_toolkit::{
    compositor::{CompositorHandler, CompositorState},
    data_device::{DataDeviceHandler, DataDeviceState},
    delegate_compositor, delegate_data_device, delegate_keyboard, delegate_output,
    delegate_registry, delegate_seat, delegate_shm,
    output::{OutputHandler, OutputState},
    registry::{ProvidesRegistryState, RegistryState},
    seat::{keyboard::KeyboardHandler, Capability, SeatHandler, SeatState},
    shm::{ShmHandler, ShmState},
};
use wayland_client::{
    protocol::{wl_data_device, wl_keyboard, wl_output, wl_seat, wl_surface},
    Connection, ConnectionHandle, QueueHandle,
};

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    let conn = Connection::connect_to_env()?;
    let display = conn.handle().display();

    let mut event_queue = conn.new_event_queue();
    let qh = event_queue.handle();
    let registry = display.get_registry(&mut conn.handle(), &qh, ())?;

    let mut clipboard = Clipboard {
        registry_state: RegistryState::new(registry),
        seat_state: SeatState::new(),
        data_device_state: DataDeviceState::new(),
        compositor_state: CompositorState::new(),
        output_state: OutputState::new(),
        shm_state: ShmState::new(),

        seat: None,
        data_device: None,
    };

    event_queue.blocking_dispatch(&mut clipboard)?;
    event_queue.blocking_dispatch(&mut clipboard)?;

    // TODO: Window creation

    // TODO: Roundtrip once and attempt to setup the data device.

    Ok(())
}

struct Clipboard {
    registry_state: RegistryState,
    seat_state: SeatState,
    data_device_state: DataDeviceState,
    compositor_state: CompositorState,
    output_state: OutputState,
    shm_state: ShmState,

    seat: Option<wl_seat::WlSeat>,
    data_device: Option<wl_data_device::WlDataDevice>,
}

impl SeatHandler for Clipboard {
    fn seat_state(&mut self) -> &mut SeatState {
        &mut self.seat_state
    }

    fn new_seat(
        &mut self,
        conn: &mut ConnectionHandle,
        qh: &QueueHandle<Self>,
        seat: wl_seat::WlSeat,
    ) {
        if self.seat.is_some() {
            log::warn!("Ignoring secondary seat");
            return;
        }

        println!("New seat");
        self.seat = Some(seat);
    }

    fn new_capability(
        &mut self,
        _conn: &mut ConnectionHandle,
        _qh: &QueueHandle<Self>,
        _seat: wl_seat::WlSeat,
        _capability: Capability,
    ) {
    }

    fn remove_capability(
        &mut self,
        _conn: &mut ConnectionHandle,
        _qh: &QueueHandle<Self>,
        _seat: wl_seat::WlSeat,
        _capability: Capability,
    ) {
    }

    fn remove_seat(
        &mut self,
        _conn: &mut ConnectionHandle,
        _qh: &QueueHandle<Self>,
        seat: wl_seat::WlSeat,
    ) {
        if self.seat.as_ref() == Some(&seat) {
            log::info!("Seat destroyed");
            self.seat.take();
        }
    }
}

impl KeyboardHandler for Clipboard {
    fn keyboard_focus(
        &mut self,
        conn: &mut ConnectionHandle,
        qh: &QueueHandle<Self>,
        keyboard: &wl_keyboard::WlKeyboard,
        surface: &wl_surface::WlSurface,
    ) {
        todo!()
    }

    fn keyboard_release_focus(
        &mut self,
        conn: &mut ConnectionHandle,
        qh: &QueueHandle<Self>,
        keyboard: &wl_keyboard::WlKeyboard,
        surface: &wl_surface::WlSurface,
    ) {
        todo!()
    }

    fn keyboard_press_key(
        &mut self,
        conn: &mut ConnectionHandle,
        qh: &QueueHandle<Self>,
        keyboard: &wl_keyboard::WlKeyboard,
        time: u32,
        key: u32,
    ) {
        todo!()
    }

    fn keyboard_release_key(
        &mut self,
        conn: &mut ConnectionHandle,
        qh: &QueueHandle<Self>,
        keyboard: &wl_keyboard::WlKeyboard,
        time: u32,
        key: u32,
    ) {
        todo!()
    }

    fn keyboard_update_modifiers(
        &mut self,
        conn: &mut ConnectionHandle,
        qh: &QueueHandle<Self>,
        keyboard: &wl_keyboard::WlKeyboard,
        // TODO: Other params
    ) {
        todo!()
    }

    fn keyboard_update_repeat_info(
        &mut self,
        conn: &mut ConnectionHandle,
        qh: &QueueHandle<Self>,
        keyboard: &wl_keyboard::WlKeyboard,
        rate: u32,
        delay: u32,
    ) {
        todo!()
    }
}

impl DataDeviceHandler for Clipboard {
    fn data_device_state(&mut self) -> &mut DataDeviceState {
        &mut self.data_device_state
    }
}

impl CompositorHandler for Clipboard {
    fn compositor_state(&mut self) -> &mut CompositorState {
        &mut self.compositor_state
    }

    fn scale_factor_changed(
        &mut self,
        conn: &mut ConnectionHandle,
        qh: &QueueHandle<Self>,
        surface: &wl_surface::WlSurface,
        new_factor: i32,
    ) {
    }

    fn frame(
        &mut self,
        conn: &mut ConnectionHandle,
        qh: &QueueHandle<Self>,
        surface: &wl_surface::WlSurface,
        time: u32,
    ) {
    }
}

impl OutputHandler for Clipboard {
    fn output_state(&mut self) -> &mut OutputState {
        &mut self.output_state
    }

    fn new_output(
        &mut self,
        conn: &mut ConnectionHandle,
        qh: &QueueHandle<Self>,
        output: wl_output::WlOutput,
    ) {
    }

    fn update_output(
        &mut self,
        conn: &mut ConnectionHandle,
        qh: &QueueHandle<Self>,
        output: wl_output::WlOutput,
    ) {
    }

    fn output_destroyed(
        &mut self,
        conn: &mut ConnectionHandle,
        qh: &QueueHandle<Self>,
        output: wl_output::WlOutput,
    ) {
    }
}

impl ShmHandler for Clipboard {
    fn shm_state(&mut self) -> &mut ShmState {
        &mut self.shm_state
    }
}

delegate_shm!(Clipboard);
delegate_output!(Clipboard);
delegate_compositor!(Clipboard);
delegate_seat!(Clipboard);
delegate_keyboard!(Clipboard);
delegate_data_device!(Clipboard);

delegate_registry!(Clipboard: [
    SeatState,
    DataDeviceState,
    OutputState,
    CompositorState,
    ShmState,
]);

impl ProvidesRegistryState for Clipboard {
    fn registry(&mut self) -> &mut RegistryState {
        &mut self.registry_state
    }
}
