use cortex_m::delay::Delay;
use psoc6_pac::Peripherals;

const TIME_UNIT: u32 = 50; // milliseconds

pub enum Pulse {
    Short,
    Long,
    LetterPause,
    WordPause,
}

pub fn char_to_morse_pulses(c: char) -> &'static [Pulse] {
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

pub fn execute_morse_pulse(
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
