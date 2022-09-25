#![no_std]
#![no_main]
#![feature(never_type)]
#![feature(strict_provenance)]
#![feature(naked_functions)]
#![feature(alloc_error_handler)]

use cortex_m_rt::entry;
use defmt::*;
use defmt_rtt as _;
use embedded_hal::digital::v2::OutputPin;
use panic_probe as _;
// use panic_abort;
extern crate alloc;

// Provide an alias for our BSP so we can switch targets quickly.
// Uncomment the BSP you included in Cargo.toml, the rest of the code does not need to change.
use rp_pico as bsp;
// use sparkfun_pro_micro_rp2040 as bsp;

use bsp::hal::{
    clocks::init_clocks_and_plls,
    gpio::{bank0::Gpio25, FunctionUart, Output, Pin, Pins, PushPull},
    pac,
    sio::Sio,
    timer::Timer,
    uart::{self, UartPeripheral},
    usb::UsbBus,
    watchdog::Watchdog,
    Clock,
};

use alloc_cortex_m::CortexMHeap;

use services::{
    post_office::PostOffice,
    task::{Task, TaskArgument},
};
use usb_device::class_prelude::UsbBusAllocator;

#[macro_use]
mod macros;

use picos_proc_macros::{add_task, task};

mod constants;
mod gw2_rotations;
mod services;
mod sync;
use alloc::boxed::Box;
use constants::HEAP_SIZE;
use gw2_rotations::condi_sb::{_run_rotationArguments, run_rotation};
use sync::Spinlock;

#[global_allocator]
static ALLOCATOR: CortexMHeap = CortexMHeap::empty();

#[entry]
fn main() -> ! {
    // info!("Program start");
    let mut pac = pac::Peripherals::take().unwrap();
    let core = pac::CorePeripherals::take().unwrap();
    let mut watchdog = Watchdog::new(pac.WATCHDOG);
    let sio = Sio::new(pac.SIO);

    // Setup the heap
    {
        use core::mem::MaybeUninit;
        static mut HEAP: [MaybeUninit<u8>; HEAP_SIZE] = [MaybeUninit::uninit(); HEAP_SIZE];
        unsafe { ALLOCATOR.init(HEAP.as_ptr() as usize, HEAP_SIZE) }
    }
    services::post_office::PostOffice::init().unwrap();
    let mut scheduler =
        services::scheduler::Scheduler::new(services::scheduler::ScheduleType::RoundRobin(0x1000));

    // Used for tracking the SysTick
    let pins = bsp::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );
    let led_pin = pins.led.into_push_pull_output();

    LED.lock().borrow_mut().replace(led_pin);
    // END Used for tracking the SysTick

    // External high-speed crystal on the pico board is 12Mhz
    let external_xtal_freq_hz = 12_000_000u32;
    let clocks = init_clocks_and_plls(
        external_xtal_freq_hz,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    // Initialze the timer peripheral
    let timer = Timer::new(pac.TIMER, &mut pac.RESETS);
    TIMER.lock().borrow_mut().replace(timer);

    // Initialize UART peripheral and task
    let pins = (
        pins.gpio0.into_mode::<FunctionUart>(),
        pins.gpio1.into_mode::<FunctionUart>(),
    );
    let uart = UartPeripheral::new(pac.UART0, pins, &mut pac.RESETS)
        .enable(
            uart::common_configs::_115200_8_N_1,
            clocks.peripheral_clock.freq(),
        )
        .unwrap();

    scheduler.add_task(services::uart::uart_task(uart)).unwrap();

    // Initialize USB
    let usb_bus = UsbBusAllocator::new(UsbBus::new(
        pac.USBCTRL_REGS,
        pac.USBCTRL_DPRAM,
        clocks.usb_clock,
        true,
        &mut pac.RESETS,
    ));
    services::usb::init_globals(usb_bus);

    // scheduler.add_task(Task::new(
    //     "Rotation".into(),
    //     run_rotation,
    //     RunRotationArguments { timer: &timer },
    // ));

    scheduler
        .add_task(Task::new(
            "Enable LED".into(),
            enable_led,
            _enable_ledArguments {
                enable: true,
                delay: 60_000_000,
            },
        ))
        .unwrap();

    // scheduler
    //     .add_task(Task::new(
    //         "Condi Soulbeast".into(),
    //         run_rotation,
    //         _run_rotationArguments {},
    //     ))
    //     .unwrap();

    scheduler
        .add_task(Task::new("Idle".into(), idle, _idleArguments {}))
        .unwrap();

    scheduler.start(core.SYST).unwrap();

    loop {}
}

use core::cell::RefCell;
use cortex_m::interrupt::Mutex;

static LED: Spinlock<RefCell<Option<Pin<Gpio25, Output<PushPull>>>>> =
    Spinlock::new(RefCell::new(None));
static TIMER: Spinlock<RefCell<Option<Timer>>> = Spinlock::new(RefCell::new(None));

#[task]
pub fn enable_led(enable: bool, delay: u32) -> ! {
    if enable {
        loop {
            LED.lock().get_mut().as_mut().unwrap().set_high().unwrap();
            debug!("LED On.");
            cortex_m::asm::delay(delay);
            LED.lock().get_mut().as_mut().unwrap().set_low().unwrap();
            debug!("LED Off.");
            cortex_m::asm::delay(delay);
        }
    }
    loop {}
}

#[task]
pub fn idle() -> ! {
    loop {}
}

#[alloc_error_handler]
fn oom(_: alloc::alloc::Layout) -> ! {
    crate::panic!("Out of memory!");
    loop {}
}

// End of file
