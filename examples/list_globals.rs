use smithay_client_toolkit::{global::NoEventGlobal, sctk};
use wayland_client::{
    protocol::{wl_compositor, wl_subcompositor},
    Connection, Dispatch,
};

#[derive(Debug)]
pub struct TestEnv {
    wl_compositor: NoEventGlobal<wl_compositor::WlCompositor>,
    wl_subcompositor: NoEventGlobal<wl_subcompositor::WlSubcompositor>,
}

sctk!(TestEnv,
    // All globals that should be provided when dispatching for WlRegistry
    globals = [
        // Provide WlCompositor
        {wl_compositor::WlCompositor => [
            // To these fields
            wl_compositor
        ]},

        // Similarly provide WlSubcompositor
        {wl_subcompositor::WlSubcompositor => [
            // To these fields
            wl_subcompositor
        ]},
    ],

    // What should be delegated to what
    delegates = [
        // `Type of thing to delegate to` ; [`delegate each of these`] => `to this field`,

        // Delegate wl_compositor to TestEnv.wl_compositor.
        NoEventGlobal<wl_compositor::WlCompositor> ; [wl_compositor::WlCompositor] => wl_compositor,
        NoEventGlobal<wl_subcompositor::WlSubcompositor> ; [wl_subcompositor::WlSubcompositor] => wl_subcompositor,
    ],
);

fn main() {
    let mut env = TestEnv {
        wl_compositor: NoEventGlobal::new(1..2),
        wl_subcompositor: NoEventGlobal::new(1..2),
    };

    let cx = Connection::connect_to_env().expect("Could not connect to compositor");

    let display = cx.handle().display();

    let mut event_queue = cx.new_event_queue();
    let qh = event_queue.handle();

    let _registry = display.get_registry(&mut cx.handle(), &qh, ()).unwrap();

    // First must be blocking.
    println!("first");
    cx.flush().unwrap();
    event_queue.blocking_dispatch(&mut env).expect("1st dispatch");

    println!("Second");
    //cx.roundtrip().unwrap();
    //cx.roundtrip().unwrap();
    println!("_");
    //cx.flush().unwrap();
    //event_queue.blocking_dispatch(&mut env).expect("2nd dispatch");

    assert!(env.wl_compositor.get().is_some());
    assert!(env.wl_subcompositor.get().is_some());

    // Make sure delegate_dispatch! is completed.
    fn test_impl<
        T: Dispatch<wl_compositor::WlCompositor, UserData = ()>
            + Dispatch<wl_subcompositor::WlSubcompositor, UserData = ()>,
    >() {
    }
    test_impl::<TestEnv>();
}
