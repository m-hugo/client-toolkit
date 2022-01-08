use std::sync::Mutex;

use wayland_client::{
    protocol::{wl_data_device_manager, wl_data_offer},
    ConnectionHandle, DelegateDispatch, DelegateDispatchBase, Dispatch, QueueHandle,
};

use crate::seat::SeatData;

use super::DataDeviceState;

#[derive(Debug)]
pub struct DataOfferData {
    mime_types: Mutex<Vec<String>>,
    source_actions: Mutex<Vec<wl_data_device_manager::DndAction>>,
}

impl DataOfferData {
    pub(crate) fn new() -> DataOfferData {
        DataOfferData { mime_types: Mutex::new(vec![]), source_actions: Mutex::new(vec![]) }
    }
}

impl DelegateDispatchBase<wl_data_offer::WlDataOffer> for DataDeviceState {
    type UserData = DataOfferData;
}

impl<D> DelegateDispatch<wl_data_offer::WlDataOffer, D> for DataDeviceState
where
    D: Dispatch<wl_data_offer::WlDataOffer, UserData = Self::UserData>,
{
    fn event(
        data: &mut D,
        offer: &wl_data_offer::WlDataOffer,
        event: wl_data_offer::Event,
        udata: &Self::UserData,
        conn: &mut ConnectionHandle,
        qh: &QueueHandle<D>,
    ) {
        match event {
            wl_data_offer::Event::Offer { mime_type } => {
                udata.mime_types.lock().unwrap().push(mime_type);
            }

            wl_data_offer::Event::SourceActions { source_actions } => todo!(),

            wl_data_offer::Event::Action { dnd_action } => todo!(),

            _ => unreachable!(),
        }
    }
}
