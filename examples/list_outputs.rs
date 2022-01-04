//! Test application to list all available outputs.

use smithay_client_toolkit::{
    delegate_registry,
    output::{OutputHandler, OutputInfo, OutputState},
    registry::RegistryState,
};
use wayland_client::{
    delegate_dispatch, protocol::wl_output, Connection, ConnectionHandle, QueueHandle,
};
use wayland_protocols::unstable::xdg_output::v1::client::{zxdg_output_manager_v1, zxdg_output_v1};

struct ListOutputs {
    registry_state: RegistryState,
    output_state: OutputState,
}

// OutputHandler's functions are called as outputs are made available, updated and destroyed.
impl OutputHandler<Self> for ListOutputs {
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
        _qh: &QueueHandle<ListOutputs>,
        _output: wl_output::WlOutput,
    ) {
    }

    fn output_destroyed(
        &mut self,
        _conn: &mut ConnectionHandle,
        _qh: &QueueHandle<ListOutputs>,
        _output: wl_output::WlOutput,
    ) {
    }

    fn state(&mut self) -> &mut OutputState {
        &mut self.output_state
    }
}

delegate_dispatch!(ListOutputs: [wl_output::WlOutput, zxdg_output_manager_v1::ZxdgOutputManagerV1, zxdg_output_v1::ZxdgOutputV1] => OutputState);

// delegate_output!(ListOutputs => InnerApp: |app| {
//     &mut OutputDispatch(&mut app.output_state, &mut app.inner, PhantomData)
// });

// Delegate wl_registry to provide the wl_output globals to OutputState
delegate_registry!(ListOutputs: [OutputState]);

impl AsMut<RegistryState> for ListOutputs {
    fn as_mut(&mut self) -> &mut RegistryState {
        &mut self.registry_state
    }
}

fn main() {
    env_logger::init();

    let conn = Connection::connect_to_env().unwrap();

    let display = conn.handle().display();

    let mut event_queue = conn.new_event_queue();
    let qh = event_queue.handle();

    let registry = display.get_registry(&mut conn.handle(), &qh, ()).unwrap();

    let mut list_outputs = ListOutputs {
        registry_state: RegistryState::new(registry),
        output_state: OutputState::new(),
    };
    event_queue.blocking_dispatch(&mut list_outputs).unwrap();
    event_queue.blocking_dispatch(&mut list_outputs).unwrap();

    for output in list_outputs.output_state.outputs() {
        print_output(&list_outputs.output_state.info(&output).unwrap());
    }
}

fn print_output(info: &OutputInfo) {
    println!("{}", info.model);

    if let Some(name) = info.name.as_ref() {
        println!("\tname: {}", name);
    }

    if let Some(description) = info.description.as_ref() {
        println!("\tdescription: {}", description);
    }

    println!("\tmake: {}", info.make);
    println!("\tx: {}, y: {}", info.location.0, info.location.1);
    println!("\tsubpixel: {:?}", info.subpixel);
    println!("\tphysical_size: {}×{}mm", info.physical_size.0, info.physical_size.1);
    println!("\tmodes:");

    for mode in &info.modes {
        println!("\t\t{}", mode);
    }
}
