use crate::UsbBus;
use core::cell::RefCell;
use cortex_m::{
    interrupt::{free, Mutex},
    peripheral::NVIC,
};
use usb_device::bus::UsbBusAllocator;
use usb_device::prelude::*;
use usbd_hid_device::{Hid, USB_CLASS_HID};

use crate::bsp::hal::pac::{interrupt, Interrupt};

use super::report::*;

struct Usb {
    device: UsbDevice<'static, UsbBus>,
    hid: Hid<'static, KeyboardReport, UsbBus>,
}

static USB: Mutex<RefCell<Option<Usb>>> = Mutex::new(RefCell::new(None));

#[interrupt]
unsafe fn USBCTRL_IRQ() {
    usb_interrupt();
}

fn usb_interrupt() {
    free(move |cs| {
        let mut borrow = USB.borrow(&cs).borrow_mut();
        let usb = &mut borrow.as_mut().unwrap();
        usb.device.poll(&mut [&mut usb.hid]);
    })
}

pub fn init_globals(usb_alloc: UsbBusAllocator<UsbBus>) {
    static mut USB_ALLOC: Option<UsbBusAllocator<UsbBus>> = None;
    let usb_alloc = unsafe {
        USB_ALLOC = Some(usb_alloc);
        USB_ALLOC.as_ref().unwrap()
    };

    let hid = Hid::new(&usb_alloc, 10);

    let device = UsbDeviceBuilder::new(&usb_alloc, UsbVidPid(0x1337, 0x4141))
        .product("PicoBoard")
        .device_class(USB_CLASS_HID)
        .build();

    let usb = Usb { hid, device };

    free(move |cs| {
        USB.borrow(&cs).replace(Some(usb));
    });
    unsafe {
        NVIC::unmask(Interrupt::USBCTRL_IRQ);
    }
}

pub fn send(report: &KeyboardReport) {
    free(move |cs| {
        let mut borrow = USB.borrow(&cs).borrow_mut();
        let usb = &mut borrow.as_mut().unwrap();

        usb.hid.send_report(report);
    })
}
