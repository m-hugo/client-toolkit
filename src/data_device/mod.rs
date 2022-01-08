pub mod offer;

use wayland_client::{
    event_created_child,
    protocol::{wl_data_device, wl_data_device_manager, wl_data_offer, wl_seat},
    ConnectionHandle, DelegateDispatch, DelegateDispatchBase, Dispatch, QueueHandle,
};

use crate::{
    registry::{ProvidesRegistryState, RegistryHandler},
    seat::{SeatData, MAX_SEAT_VERSION},
};

use self::offer::DataOfferData;

#[derive(Debug)]
pub struct DataDeviceState {
    wl_data_device_manager: Option<(u32, wl_data_device_manager::WlDataDeviceManager)>,
    seats: Vec<(u32, wl_seat::WlSeat)>,
}

impl DataDeviceState {
    pub fn new() -> DataDeviceState {
        DataDeviceState { wl_data_device_manager: None, seats: vec![] }
    }

    pub fn get_data_device<D>(
        &self,
        _conn: &mut ConnectionHandle,
        _qh: &QueueHandle<D>,
        _seat: &wl_seat::WlSeat,
    ) -> Result<wl_data_device::WlDataDevice, ()>
    where
        D: Dispatch<wl_seat::WlSeat, UserData = SeatData>
            + Dispatch<wl_data_device::WlDataDevice, UserData = SeatData>
            + 'static,
    {
        let (_, _data_device_manager) = self.wl_data_device_manager.as_ref().ok_or(())?;

        todo!()
    }
}

pub trait DataDeviceHandler: Sized {
    fn data_device_state(&mut self) -> &mut DataDeviceState;
}

#[macro_export]
macro_rules! delegate_data_device {
    ($ty: ty) => {
        type __WlDataDeviceManager = $crate::reexports::client::protocol::wl_data_device_manager::WlDataDeviceManager;
        type __WlDataDevice = $crate::reexports::client::protocol::wl_data_device::WlDataDevice;
        type __WlDataOffer = $crate::reexports::client::protocol::wl_data_offer::WlDataOffer;

        $crate::reexports::client::delegate_dispatch!($ty: [
            __WlDataDeviceManager,
            __WlDataDevice,
            __WlDataOffer
        ] => $crate::data_device::DataDeviceState);
    };
}

impl DelegateDispatchBase<wl_data_device_manager::WlDataDeviceManager> for DataDeviceState {
    type UserData = ();
}

impl<D> DelegateDispatch<wl_data_device_manager::WlDataDeviceManager, D> for DataDeviceState
where
    D: Dispatch<wl_data_device_manager::WlDataDeviceManager, UserData = Self::UserData>,
{
    fn event(
        _: &mut D,
        _: &wl_data_device_manager::WlDataDeviceManager,
        _: wl_data_device_manager::Event,
        _: &Self::UserData,
        _: &mut ConnectionHandle,
        _: &QueueHandle<D>,
    ) {
        unreachable!("wl_data_device_manager has no events")
    }
}

impl DelegateDispatchBase<wl_data_device::WlDataDevice> for DataDeviceState {
    type UserData = SeatData;
}

impl<D> DelegateDispatch<wl_data_device::WlDataDevice, D> for DataDeviceState
where
    D: Dispatch<wl_data_device::WlDataDevice, UserData = Self::UserData>
        + Dispatch<wl_data_offer::WlDataOffer, UserData = DataOfferData>
        + DataDeviceHandler
        + 'static,
{
    fn event(
        data: &mut D,
        proxy: &wl_data_device::WlDataDevice,
        event: wl_data_device::Event,
        udata: &Self::UserData,
        connhandle: &mut ConnectionHandle,
        qhandle: &QueueHandle<D>,
    ) {
        match event {
            wl_data_device::Event::DataOffer { id } => todo!(),

            wl_data_device::Event::Enter { serial, surface, x, y, id } => {
                log::error!(target: "sctk", "DND not implemented yet");
            }

            wl_data_device::Event::Leave => {
                log::error!(target: "sctk", "DND not implemented yet");
            }

            wl_data_device::Event::Motion { time, x, y } => {
                log::error!(target: "sctk", "DND not implemented yet");
            }

            wl_data_device::Event::Drop => {
                log::error!(target: "sctk", "DND not implemented yet");
            }

            wl_data_device::Event::Selection { id } => {
                // TODO: Send event indicating the clipboard contents have been advertised.
            }

            _ => unreachable!(),
        }
    }

    event_created_child!(D, wl_data_device::WlDataDevice, [
        // wl_data_device::data_offer(new_id)
        0 => (wl_data_offer::WlDataOffer, DataOfferData::new()),
    ]);
}

impl<D> RegistryHandler<D> for DataDeviceState
where
    D: Dispatch<wl_data_device_manager::WlDataDeviceManager, UserData = ()>
        + Dispatch<wl_seat::WlSeat, UserData = SeatData>
        + DataDeviceHandler
        + ProvidesRegistryState
        + 'static,
{
    fn new_global(
        data: &mut D,
        conn: &mut ConnectionHandle,
        qh: &QueueHandle<D>,
        name: u32,
        interface: &str,
        version: u32,
    ) {
        match interface {
            "wl_data_device_manager" => {
                let data_device_manager = data
                    .registry()
                    .bind_once(conn, qh, name, u32::min(version, 3), ())
                    .expect("Failed to bind global");

                data.data_device_state().wl_data_device_manager = Some((name, data_device_manager));
            }

            // We need to keep track of what seats are available.
            "wl_seat" => {
                let seat = data
                    .registry()
                    .bind_cached::<wl_seat::WlSeat, _, _, _>(conn, qh, name, || {
                        (u32::min(version, MAX_SEAT_VERSION), SeatData::new())
                    })
                    .expect("Failed to bind global");

                data.data_device_state().seats.push((name, seat));
            }

            _ => (),
        }
    }

    fn remove_global(data: &mut D, _: &mut ConnectionHandle, _: &QueueHandle<D>, name: u32) {
        let data_device_state = data.data_device_state();

        if let Some((global_name, _)) = data_device_state.wl_data_device_manager {
            if global_name == name {
                data_device_state.wl_data_device_manager.take();
            }
        }

        data_device_state.seats.retain(|(global_name, _)| global_name != &name);
    }
}
