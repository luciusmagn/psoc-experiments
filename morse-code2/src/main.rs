#![no_std]
#![no_main]

use cortex_m::delay::Delay;
use panic_semihosting as _;
use psoc6_pac::Peripherals;
use systick_monotonic::*;

enum Pulse {
    Short,
    Long,
    LetterPause,
    WordPause,
}

/*

$OPENOCD_ROOT/bin/openocd \
     -f interface/cmsis-dap.cfg \
     -f target/psoc6_2m.cfg \
     -c "program psoc6-cm0-bootloader/target/thumbv6m-none-eabi/release/psoc6-cm0-bootloader verify reset exit"

arm-none-eabi-objcopy -O binary \
       ../morse-code2/target/thumbv7em-none-eabihf/release/morse-code2 \
       ../psoc6-cm0-bootloader/src/app.bin

cargo build --release

*/

fn char_to_morse_pulses(c: char) -> &'static [Pulse] {
    match c.to_ascii_uppercase() {
        'A' => &[Pulse::Short, Pulse::Long],
        'B' => &[
            Pulse::Long,
            Pulse::Short,
            Pulse::Short,
            Pulse::Short,
        ],
        'C' => &[
            Pulse::Long,
            Pulse::Short,
            Pulse::Long,
            Pulse::Short,
        ],
        'D' => &[Pulse::Long, Pulse::Short, Pulse::Short],
        'E' => &[Pulse::Short],
        'F' => &[
            Pulse::Short,
            Pulse::Short,
            Pulse::Long,
            Pulse::Short,
        ],
        'G' => &[Pulse::Long, Pulse::Long, Pulse::Short],
        'H' => &[
            Pulse::Short,
            Pulse::Short,
            Pulse::Short,
            Pulse::Short,
        ],
        'I' => &[Pulse::Short, Pulse::Short],
        'J' => &[
            Pulse::Short,
            Pulse::Long,
            Pulse::Long,
            Pulse::Long,
        ],
        'K' => &[Pulse::Long, Pulse::Short, Pulse::Long],
        'L' => &[
            Pulse::Short,
            Pulse::Long,
            Pulse::Short,
            Pulse::Short,
        ],
        'M' => &[Pulse::Long, Pulse::Long],
        'N' => &[Pulse::Long, Pulse::Short],
        'O' => &[Pulse::Long, Pulse::Long, Pulse::Long],
        'P' => &[
            Pulse::Short,
            Pulse::Long,
            Pulse::Long,
            Pulse::Short,
        ],
        'Q' => &[
            Pulse::Long,
            Pulse::Long,
            Pulse::Short,
            Pulse::Long,
        ],
        'R' => &[Pulse::Short, Pulse::Long, Pulse::Short],
        'S' => &[Pulse::Short, Pulse::Short, Pulse::Short],
        'T' => &[Pulse::Long],
        'U' => &[Pulse::Short, Pulse::Short, Pulse::Long],
        'V' => &[
            Pulse::Short,
            Pulse::Short,
            Pulse::Short,
            Pulse::Long,
        ],
        'W' => &[Pulse::Short, Pulse::Long, Pulse::Long],
        'X' => &[
            Pulse::Long,
            Pulse::Short,
            Pulse::Short,
            Pulse::Long,
        ],
        'Y' => &[
            Pulse::Long,
            Pulse::Short,
            Pulse::Long,
            Pulse::Long,
        ],
        'Z' => &[
            Pulse::Long,
            Pulse::Long,
            Pulse::Short,
            Pulse::Short,
        ],
        '0' => &[
            Pulse::Long,
            Pulse::Long,
            Pulse::Long,
            Pulse::Long,
            Pulse::Long,
        ],
        '1' => &[
            Pulse::Short,
            Pulse::Long,
            Pulse::Long,
            Pulse::Long,
            Pulse::Long,
        ],
        '2' => &[
            Pulse::Short,
            Pulse::Short,
            Pulse::Long,
            Pulse::Long,
            Pulse::Long,
        ],
        '3' => &[
            Pulse::Short,
            Pulse::Short,
            Pulse::Short,
            Pulse::Long,
            Pulse::Long,
        ],
        '4' => &[
            Pulse::Short,
            Pulse::Short,
            Pulse::Short,
            Pulse::Short,
            Pulse::Long,
        ],
        '5' => &[
            Pulse::Short,
            Pulse::Short,
            Pulse::Short,
            Pulse::Short,
            Pulse::Short,
        ],
        '6' => &[
            Pulse::Long,
            Pulse::Short,
            Pulse::Short,
            Pulse::Short,
            Pulse::Short,
        ],
        '7' => &[
            Pulse::Long,
            Pulse::Long,
            Pulse::Short,
            Pulse::Short,
            Pulse::Short,
        ],
        '8' => &[
            Pulse::Long,
            Pulse::Long,
            Pulse::Long,
            Pulse::Short,
            Pulse::Short,
        ],
        '9' => &[
            Pulse::Long,
            Pulse::Long,
            Pulse::Long,
            Pulse::Long,
            Pulse::Short,
        ],
        ' ' => &[Pulse::WordPause],
        _ => &[], // Unknown characters
    }
}

const TIME_UNIT: u32 = 75; // milliseconds

fn execute_morse_pulse(
    p: &Peripherals,
    delay: &mut Delay,
    pulse: &Pulse,
) {
    match pulse {
        Pulse::Short => {
            // LED on
            p.GPIO.prt13.out_clr.write(|w| {
                w.out7().set_bit();
                w
            });
            delay.delay_ms(TIME_UNIT);

            // LED off
            p.GPIO.prt13.out_set.write(|w| {
                w.out7().set_bit();
                w
            });
            delay.delay_ms(TIME_UNIT);
        }
        Pulse::Long => {
            // LED on
            p.GPIO.prt13.out_clr.write(|w| {
                w.out7().set_bit();
                w
            });
            delay.delay_ms(TIME_UNIT * 3);

            // LED off
            p.GPIO.prt13.out_set.write(|w| {
                w.out7().set_bit();
                w
            });
            delay.delay_ms(TIME_UNIT);
        }
        Pulse::LetterPause => {
            // Pause between letters
            delay.delay_ms(TIME_UNIT * 3);
        }
        Pulse::WordPause => {
            // Pause between words
            delay.delay_ms(TIME_UNIT * 7);
        }
    }
}

#[rtic::app(device = psoc6_pac, peripherals = true, dispatchers = [SCB_12_INTERRUPT])]
mod app {
    use super::*;
    use cortex_m::delay::Delay;
    use psoc6_pac::Peripherals;

    #[monotonic(binds = SysTick, default = true)]
    type MyMono = Systick<100>;

    #[shared]
    struct Shared {}

    #[local]
    struct Local {}

    #[init]
    fn init(
        mut cx: init::Context,
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

        // Initialize the monotonic timer (SysTick rate on PSoC6 is 50 MHz)
        let mono = Systick::new(cx.core.SYST, 50_000_000);

        // Configure LED
        cx.device.GPIO.prt13.cfg.write(|w| {
            w.in_en7().clear_bit();
            w.drive_mode7().variant(6); // Strong drive mode
            w
        });

        // Initially turn LED off
        cx.device.GPIO.prt13.out_set.write(|w| {
            w.out7().set_bit();
            w
        });

        (Shared {}, Local {}, init::Monotonics(mono))
    }

    #[idle]
    fn idle(_: idle::Context) -> ! {
        let mut delay = cortex_m::delay::Delay::new(
            unsafe { cortex_m::Peripherals::steal().SYST },
            50_000_000,
        );

        loop {
            let p = unsafe { Peripherals::steal() };

            for c in "NEVER KILL YOURSELF RETARD STOP".chars() {
                for pulse in char_to_morse_pulses(c) {
                    execute_morse_pulse(&p, &mut delay, pulse);
                }
                // Add letter pause after each character
                execute_morse_pulse(
                    &p,
                    &mut delay,
                    &Pulse::LetterPause,
                );
            }
        }
    }
}
