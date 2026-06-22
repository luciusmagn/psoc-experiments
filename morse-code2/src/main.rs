#![no_std]
#![no_main]

use panic_semihosting as _;
use psoc6_pac::Peripherals;
use systick_monotonic::*;

mod morse_encoding;
use morse_encoding::{
    char_to_morse_pulses, execute_morse_pulse, Pulse,
};

/*
* 1. build with use-bootloader function
*
* $OPENOCD_ROOT/bin/openocd \
     -f interface/cmsis-dap.cfg \
     -f target/psoc6_2m.cfg \
     -c "program psoc6-cm0-bootloader/target/thumbv6m-none-eabi/release/psoc6-cm0-bootloader verify reset exit"
*
*
*/

fn real_main() {}

#[rtic::app(device = psoc6_pac, peripherals = true, dispatchers = [SCB_12_INTERRUPT])]
mod app {
    use cortex_m_semihosting::hprintln;

    use super::*;

    #[monotonic(binds = SysTick, default = true)]
    type MyMono = Systick<100>;

    #[shared]
    struct Shared {}

    #[local]
    struct Local {}

    #[init]
    fn init(
        cx: init::Context,
    ) -> (Shared, Local, init::Monotonics) {
        // Fixup VTOR for bootloader
        extern "C" {
            static _svectors: u32;
        }
        unsafe {
            cx.core
                .SCB
                .vtor
                .write(&_svectors as *const u32 as u32);
        }

        hprintln!("eeeee");
        // Configure LED
        cx.device.GPIO.prt13.cfg.write(|w| {
            w.in_en7().clear_bit();
            w.drive_mode7().variant(6); // Strong drive mode
            w
        });

        hprintln!("eeeeeaceae");
        // Initially turn LED off
        cx.device.GPIO.prt13.out_set.write(|w| {
            w.out7().set_bit();
            w
        });
        hprintln!("oraere");
        // UART Configuration for SCB5
        let scb = cx.device.SCB5;

        hprintln!("kodork");
        // Configure UART mode
        scb.ctrl.modify(|_, w| w.mode().uart());

        // Set UART configuration
        /*
        scb.uart_ctrl.modify(|_, w| {
            w.mode().modify(|_, w| {
                w.variant(MODE_A);
            });
            w.stop_bits().variant(1);
            w
        });
        */

        hprintln!("rforatde");
        // Enable RX/TX
        scb.uart_tx_ctrl
            .modify(|_, w| w.parity_enabled().clear_bit());
        scb.uart_rx_ctrl
            .modify(|_, w| w.parity_enabled().clear_bit());

        hprintln!("hehoo");
        let mut delay = cortex_m::delay::Delay::new(
            cx.core.SYST,
            50_000_000,
        );

        hprintln!("ohodo");
        let p = unsafe { Peripherals::steal() };
        let mut rx_buffer = [0u8; 64];
        let mut rx_index = 0;

        hprintln!("oahek");
        loop {
            // Check for incoming UART data
            if scb.intr_rx.read().trigger().bit_is_set() {
                hprintln!("bit is set");
                let received_byte =
                    scb.rx_fifo_rd.read().data().bits() as u8;

                // Process received byte
                if received_byte == b'\r'
                    || received_byte == b'\n'
                {
                    // Process the received string
                    if rx_index > 0 {
                        let message = core::str::from_utf8(
                            &rx_buffer[..rx_index],
                        )
                        .unwrap_or("ERROR");

                        // Blink out the received message in Morse code
                        for c in message.chars() {
                            for pulse in char_to_morse_pulses(c)
                            {
                                execute_morse_pulse(
                                    &p, &mut delay, pulse,
                                );
                            }
                            execute_morse_pulse(
                                &p,
                                &mut delay,
                                &Pulse::LetterPause,
                            );
                        }

                        // Reset buffer
                        rx_index = 0;
                    }
                } else if rx_index < rx_buffer.len() {
                    // Store received character
                    rx_buffer[rx_index] = received_byte;
                    rx_index += 1;
                }

                // Clear the RX trigger
                scb.intr_rx.write(|w| w.trigger().set_bit());
            }
        }
    }
}
