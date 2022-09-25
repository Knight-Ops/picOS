use usbd_hid_device::{Hid, HidReport};

pub static RELEASE_ALL: KeyboardReport = KeyboardReport {
    bytes: [0, 0, 0, 0, 0, 0, 0, 0],
};

#[repr(u8)]
pub enum MOD_KEY {
    NONE = 0,
    LEFT_CTRL = 1,
    LEFT_SHIFT = 2,
    LEFT_ALT = 4,
    LEFT_GUI = 8,
    RIGHT_CTRL = 16,
    RIGHT_SHIFT = 32,
    RIGHT_ALT = 64,
    RIGHT_GUI = 128,
}

pub struct KeyboardReport {
    bytes: [u8; 8],
}

impl KeyboardReport {
    pub fn new(
        modifier: Option<MOD_KEY>,
        key1: Option<u8>,
        key2: Option<u8>,
        key3: Option<u8>,
        key4: Option<u8>,
        key5: Option<u8>,
        key6: Option<u8>,
    ) -> Self {
        KeyboardReport {
            bytes: [
                modifier.unwrap_or(MOD_KEY::NONE) as u8,
                0,
                key1.unwrap_or(0),
                key2.unwrap_or(0),
                key3.unwrap_or(0),
                key4.unwrap_or(0),
                key5.unwrap_or(0),
                key6.unwrap_or(0),
            ],
        }
    }
}

impl AsRef<[u8]> for KeyboardReport {
    fn as_ref(&self) -> &[u8] {
        &self.bytes
    }
}

impl HidReport for KeyboardReport {
    const DESCRIPTOR: &'static [u8] = &[
        0x05, 0x01, //USAGE_PAGE (Generic Desktop)
        0x09, 0x06, //USAGE (Keyboard)
        0xA1, 0x01, //COLLECTION (Application)
        0x05, 0x07, //USAGE_PAGE (Keyboard)
        0x19, 0xE0, //USAGE_MINIMUM (Keyboard LeftControl)
        0x29, 0xE7, //USAGE_MAXIMUM (Keyboard Right GUI)
        0x15, 0x00, //LOGICAL_MINIMUM (0)
        0x25, 0x01, //LOGICAL_MAXIMUM (1)
        0x75, 0x01, //REPORT_SIZE (1)
        0x95, 0x08, //REPORT_COUNT (8)
        0x81, 0x02, //INPUT (Data,Var,Abs)
        0x95, 0x01, //REPORT_COUNT (1)
        0x75, 0x08, //REPORT_SIZE (8)
        0x81, 0x03, //INPUT (Cnst,Var,Abs)
        0x95, 0x05, //REPORT_COUNT (5)
        0x75, 0x01, //REPORT_SIZE (1)
        // 0x05, 0x08, //USAGE_PAGE (LEDs)
        // 0x19, 0x01, //USAGE_MINIMUM (Num Lock)
        // 0x29, 0x05, //USAGE_MAXIMUM (Kana)
        // 0x91, 0x02, //OUTPUT (Data,Var,Abs)
        // 0x95, 0x01, //REPORT_COUNT (1)
        // 0x75, 0x03, //REPORT_SIZE (3)
        // 0x91, 0x03, //OUTPUT (Cnst,Var,Abs)
        0x95, 0x06, //REPORT_COUNT (6)
        0x75, 0x08, //REPORT_SIZE (8)
        0x15, 0x00, //LOGICAL_MINIMUM (0)
        0x25, 0x65, //LOGICAL_MAXIMUM (101)
        0x05, 0x07, //USAGE_PAGE (Keyboard)
        0x19, 0x00, //USAGE_MINIMUM (Reserved (no event indicated))
        0x29, 0x65, //USAGE_MAXIMUM (Keyboard Application)
        0x81, 0x00, //INPUT (Data,Ary,Abs)
        0xC0, //END_COLLECTION
    ];
}
