#![no_main]
#![no_std]

use core::sync::atomic::{AtomicUsize, Ordering};
use cortex_m::asm::delay;
use defmt_rtt as _; // global logger
use hal::gpio::{EPin, Input, Output, PullUp, PushPull};
use hal::prelude::*;
use hal::usb::{Peripheral, UsbBus, UsbBusType};
use keyberon::debounce::Debouncer;
use keyberon::key_code::KbHidReport;
use keyberon::layout::{CustomEvent, Layout};
use keyberon::matrix::Matrix;
use stm32f1xx_hal as hal;
use usb_device::bus::UsbBusAllocator;
use usb_device::class::UsbClass as _;
use usb_device::prelude::*;

use panic_probe as _;

pub mod layout;

/// USB VIP for a generic keyboard from
/// https://github.com/obdev/v-usb/blob/master/usbdrv/USB-IDs-for-free.txt
const VID: u16 = 0x16c0;

/// USB PID for a generic keyboard from
/// https://github.com/obdev/v-usb/blob/master/usbdrv/USB-IDs-for-free.txt
const PID: u16 = 0x27db;

// same panicking *behavior* as `panic-probe` but doesn't print a panic message
// this prevents the panic message being printed *twice* when `defmt::panic` is invoked
#[defmt::panic_handler]
fn panic() -> ! {
    cortex_m::asm::udf()
}

static COUNT: AtomicUsize = AtomicUsize::new(0);
defmt::timestamp!("{=usize}", {
    // NOTE(no-CAS) `timestamps` runs with interrupts disabled
    let n = COUNT.load(Ordering::Relaxed);
    COUNT.store(n + 1, Ordering::Relaxed);
    n
});

/// Terminates the application and makes `probe-run` exit with exit-code = 0
pub fn exit() -> ! {
    loop {
        cortex_m::asm::bkpt();
    }
}

type UsbClass = keyberon::Class<'static, UsbBusType, ()>;
type UsbDevice = usb_device::device::UsbDevice<'static, UsbBusType>;

#[rtic::app(device = stm32f1xx_hal::pac, dispatchers=[TIM1_CC])]
mod app {
    use super::*;
    // shared resources (between tasks)
    #[shared]
    struct Shared {
        usb_dev: UsbDevice,
        usb_class: UsbClass,
        #[lock_free]
        layout: Layout<7, 10, 5, ()>,
    }

    // local resources (between tasks)
    #[local]
    struct Local {
        matrix: Matrix<EPin<Input<PullUp>>, EPin<Output<PushPull>>, 7, 10>,
        debouncer: Debouncer<[[bool; 7]; 10]>,
        timer: hal::timer::counter::CounterHz<hal::pac::TIM2>,
        delay: cortex_m::delay::Delay,
    }

    #[init]
    fn init(cx: init::Context) -> (Shared, Local, init::Monotonics) {
        // defmt::info!("init");
        // prepare static datastructures for USB
        static mut USB_BUS: Option<UsbBusAllocator<UsbBusType>> = None;

        let rcc = cx.device.RCC.constrain();
        let mut flash = cx.device.FLASH.constrain();
        let mut afio = cx.device.AFIO.constrain();
        let mut gpioa = cx.device.GPIOA.split();
        let mut gpiob = cx.device.GPIOB.split();

        // setup the monotonic timer
        let mut clocks = rcc
            .cfgr
            .use_hse(8.MHz())
            .sysclk(48.MHz())
            .freeze(&mut flash.acr);

        // timer for processing keyboard events and sending a USB keyboard report
        let mut timer = cx.device.TIM2.counter_hz(&mut clocks);
        // or equivalently
        // let mut timer = hal::timer::Timer::new(cx.device.TIM2, &mut clocks).counter_hz();
        timer.start(1.kHz()).unwrap();
        timer.listen(hal::timer::Event::Update);

        // BluePill board has a pull-up resistor on the D+ line.
        // Pull the D+ pin down to send a RESET condition to the USB bus.
        // This forced reset is needed only for development, without it host
        // will not reset your device when you upload new firmware.
        let mut usb_dp = gpioa.pa12.into_push_pull_output(&mut gpioa.crh);
        usb_dp.set_low();
        delay(clocks.sysclk().raw() / 100);

        let usb_dm = gpioa.pa11;
        let usb_dp = usb_dp.into_floating_input(&mut gpioa.crh);

        // initialize USB
        let usb = Peripheral {
            usb: cx.device.USB,
            pin_dm: usb_dm,
            pin_dp: usb_dp,
        };

        unsafe {
            USB_BUS = Some(UsbBus::new(usb));
        }

        let usb_bus = unsafe { USB_BUS.as_ref().unwrap() };
        let usb_class = keyberon::new_class(&usb_bus, ());
        let usb_dev = UsbDeviceBuilder::new(usb_bus, UsbVidPid(VID, PID))
            .manufacturer("Dario Götz")
            .product("Dario Götz's 42-key split keyboard")
            .serial_number(env!("CARGO_PKG_VERSION"))
            .build();

        // disable jtag functionality on pins PB3 and PB4
        let (_pa15, pb3, pb4) = afio.mapr.disable_jtag(gpioa.pa15, gpiob.pb3, gpiob.pb4);

        let cols = [
            gpioa.pa0.into_pull_up_input(&mut gpioa.crl).erase(),
            gpioa.pa1.into_pull_up_input(&mut gpioa.crl).erase(),
            gpioa.pa2.into_pull_up_input(&mut gpioa.crl).erase(),
            gpioa.pa3.into_pull_up_input(&mut gpioa.crl).erase(),
            gpioa.pa4.into_pull_up_input(&mut gpioa.crl).erase(),
            gpioa.pa5.into_pull_up_input(&mut gpioa.crl).erase(),
            gpioa.pa6.into_pull_up_input(&mut gpioa.crl).erase(),
        ];

        let rows = [
            gpioa.pa8.into_push_pull_output(&mut gpioa.crh).erase(),
            gpioa.pa9.into_push_pull_output(&mut gpioa.crh).erase(),
            gpioa.pa10.into_push_pull_output(&mut gpioa.crh).erase(),
            pb3.into_push_pull_output(&mut gpiob.crl).erase(),
            pb4.into_push_pull_output(&mut gpiob.crl).erase(),
            gpiob.pb5.into_push_pull_output(&mut gpiob.crl).erase(),
            gpiob.pb6.into_push_pull_output(&mut gpiob.crl).erase(),
            gpiob.pb7.into_push_pull_output(&mut gpiob.crl).erase(),
            gpiob.pb8.into_push_pull_output(&mut gpiob.crh).erase(),
            gpiob.pb9.into_push_pull_output(&mut gpiob.crh).erase(),
        ];

        let matrix = cortex_m::interrupt::free(move |_cs| Matrix::new(cols, rows));

        let delay = cortex_m::delay::Delay::new(cx.core.SYST, clocks.sysclk().to_Hz());

        (
            Shared {
                // Initialization of shared resources go here
                usb_dev,
                usb_class,
                layout: Layout::new(&layout::LAYERS),
            },
            Local {
                // Initialization of local resources go here
                matrix: matrix.unwrap(),
                timer,
                debouncer: Debouncer::new([[false; 7]; 10], [[false; 7]; 10], 5),
                delay,
            },
            init::Monotonics(),
        )
    }

    // Optional idle, can be removed if not needed.
    #[idle]
    fn idle(_: idle::Context) -> ! {
        loop {
            continue;
        }
    }

    /// Check all switches for their state, register corresponding events, and
    /// spawn generation of a USB keyboard report (including layout event processing)
    #[task(binds=TIM2, priority=1, local=[debouncer, matrix, timer, delay], shared=[usb_dev, usb_class, layout])]
    fn tick(mut cx: tick::Context) {
        // defmt::info!("Processing keyboard events");
        cx.local.timer.wait().ok();
        // or equivalently
        // cx.local.timer.clear_interrupt(hal::timer::Event::Update);

        let delay = cx.local.delay;

        // scan keyboard
        for event in cx.local.debouncer.events(
            cx.local
                .matrix
                .get_with_delay(|| delay.delay_us(10))
                .unwrap(),
        ) {
            cx.shared.layout.event(event);
            // match event {
            //     Event::Press(i, j) => defmt::info!("Pressed {} {}", i, j),
            //     Event::Release(i, j) => defmt::info!("Released {} {}", i, j),
            // }
        }

        let tick = cx.shared.layout.tick();
        match tick {
            CustomEvent::Release(()) => unsafe { cortex_m::asm::bootload(0x1FFF0000 as _) },
            _ => (),
        }

        // if this is the USB-side, send a USB keyboard report
        let report: KbHidReport = cx.shared.layout.keycodes().collect();
        if cx
            .shared
            .usb_class
            .lock(|k| k.device_mut().set_keyboard_report(report.clone()))
        {
            while let Ok(0) = cx.shared.usb_class.lock(|k| k.write(report.as_bytes())) {}
        }
    }

    // USB events
    #[task(binds = USB_HP_CAN_TX, priority = 3, shared = [usb_dev, usb_class])]
    fn usb_tx(cx: usb_tx::Context) {
        (cx.shared.usb_dev, cx.shared.usb_class).lock(|mut usb_dev, mut usb_class| {
            usb_poll(&mut usb_dev, &mut usb_class);
        });
    }

    #[task(binds = USB_LP_CAN_RX0, priority = 3, shared = [usb_dev, usb_class])]
    fn usb_rx(cx: usb_rx::Context) {
        (cx.shared.usb_dev, cx.shared.usb_class).lock(|mut usb_dev, mut usb_class| {
            usb_poll(&mut usb_dev, &mut usb_class);
        });
    }

    fn usb_poll(usb_dev: &mut UsbDevice, keyboard: &mut UsbClass) {
        if usb_dev.poll(&mut [keyboard]) {
            keyboard.poll();
        }
    }
}
