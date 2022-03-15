use embedded_hal::digital::v2::InputPin;

pub struct OvenDoor<Pin> {
    /// Default closed    
    pin: Pin,
}

impl<Pin> OvenDoor<Pin>
where
    Pin: InputPin,
{
    pub fn new(pin: Pin) -> Self {
        OvenDoor { pin }
    }

    pub fn state(&self) -> DoorState {
        if self.is_open() {
            DoorState::Open
        } else {
            DoorState::Closed
        }
    }

    pub fn is_open(&self) -> bool {
        self.pin.is_low().unwrap_or(false)
    }

    pub fn pin_mut(&mut self) -> &mut Pin {
        &mut self.pin
    }
}

#[derive(Debug, defmt::Format)]
pub enum DoorState {
    Open,
    Closed,
}
