use super::post_office::{MailboxMessageType, PostOffice};
use crate::bsp::hal::uart::{self, Enabled, UartPeripheral};
use crate::pac::UART0;
use crate::task;
use crate::Task;
use crate::{
    bsp::hal::gpio::{
        bank0::{Gpio0, Gpio1},
        pin::Function,
        pin::Uart,
        FunctionUart, Pin, Pins,
    },
    sync::Spinlock,
};
use crate::{Mutex, TaskArgument};
use core::borrow::BorrowMut;
use core::cell::RefCell;
use defmt::*;

static UART: Spinlock<
    RefCell<
        Option<
            UartPeripheral<
                Enabled,
                UART0,
                (Pin<Gpio0, Function<Uart>>, Pin<Gpio1, Function<Uart>>),
            >,
        >,
    >,
> = Spinlock::new(RefCell::new(None));

// pub struct UartArguments {
//     pub uart: RefCell<
//         Option<
//             UartPeripheral<
//                 Enabled,
//                 UART0,
//                 (Pin<Gpio0, Function<Uart>>, Pin<Gpio1, Function<Uart>>),
//             >,
//         >,
//     >,
// }
// impl TaskArgument for UartArguments {}

// pub fn uart(args: Box<UartArguments>) -> ! {
//     UART.lock()
//         .borrow_mut()
//         .replace(args.uart.borrow_mut().take());

//     debug!("UART initialization complete!");
//     loop {
//         if let Ok(Some(msg)) = PostOffice::recv_by_name("UART".into()) {
//             let mut lock = UART.lock();
//             let writer = lock.get_mut().as_mut().unwrap();
//             writer.write_full_blocking(msg.get_data());
//         }
//     }
// }
pub fn uart_task(
    uart_periph: UartPeripheral<
        Enabled,
        UART0,
        (Pin<Gpio0, Function<Uart>>, Pin<Gpio1, Function<Uart>>),
    >,
) -> Task {
    Task::new(
        "UART".into(),
        uart,
        _uartArguments {
            uart: RefCell::new(Some(uart_periph)),
        },
    )
}

#[task]
pub fn uart(
    uart: RefCell<
        Option<
            UartPeripheral<
                Enabled,
                UART0,
                (Pin<Gpio0, Function<Uart>>, Pin<Gpio1, Function<Uart>>),
            >,
        >,
    >,
) -> ! {
    UART.lock().borrow_mut().replace(uart.borrow_mut().take());

    debug!("UART initialization complete!");
    loop {
        if let Ok(Some(msg)) = PostOffice::recv_by_name("UART".into()) {
            match msg.data {
                MailboxMessageType::Uart(data) => {
                    let mut lock = UART.lock();
                    let writer = lock.get_mut().as_mut().unwrap();
                    writer.write_full_blocking(&data);
                }
                _ => {
                    debug!("Unexpected message type in UART Mailbox");
                }
            }
        }
    }
}
