use wayland_client::{
    protocol::wl_seat, ConnectionHandle, DataInit, DelegateDispatch, DelegateDispatchBase,
    Dispatch, QueueHandle,
};

#[derive(Debug)]
pub struct Seat {
    seat: wl_seat::WlSeat,
}

impl DelegateDispatchBase<wl_seat::WlSeat> for Seat {
    type UserData = ();
}

impl<D> DelegateDispatch<wl_seat::WlSeat, D> for Seat
where
    D: Dispatch<wl_seat::WlSeat, UserData = ()>,
{
    fn event(
        &mut self,
        _proxy: &wl_seat::WlSeat,
        _event: wl_seat::Event,
        _data: &Self::UserData,
        _cxhandle: &mut ConnectionHandle,
        _qhandle: &QueueHandle<D>,
        _init: &mut DataInit<'_>,
    ) {
        todo!()
    }
}
