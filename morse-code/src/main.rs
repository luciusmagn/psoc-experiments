#![no_std]
#![no_main]

use cortex_m_rt::entry;
use cortex_m_semihosting::hprintln;
use panic_semihosting as _;
use psoc6_pac::Peripherals;

#[cfg(feature = "use-bootloader")]
fn init_vtor(cp: &cortex_m::Peripherals) {
    extern "C" {
        static _svectors: u32;
    }
    unsafe {
        // Manually set VTOR to ensure correct vector table
        cp.SCB.vtor.write(&_svectors as *const u32 as u32);
    }
}

fn init_led() {
    let p = unsafe { Peripherals::steal() };

    // Configure P13_7 as output with strong drive
    p.GPIO.prt13.cfg.write(|w| {
        w.in_en7().clear_bit();
        w.drive_mode7().variant(6); // Strong drive mode
        w
    });

    // Initially turn LED off (set pin HIGH)
    p.GPIO.prt13.out_set.write(|w| {
        w.out7().set_bit();
        w
    });
}

fn led_toggle() {
    let p = unsafe { Peripherals::steal() };

    // Toggle LED
    p.GPIO.prt13.out_inv.write(|w| {
        w.out7().set_bit();
        w
    });

    hprintln!("LED toggled!");
}

#[entry]
fn main() -> ! {
    hprintln!("LED Test Starting");
    let cp = unsafe { cortex_m::Peripherals::steal() };

    #[cfg(feature = "use-bootloader")]
    {
        init_vtor(&cp);
    }

    init_led();

    let mut delay =
        cortex_m::delay::Delay::new(cp.SYST, 50_000_000);

    //for byte in b"hello world"
    loop {
        led_toggle();
        delay.delay_ms(500);
    }
}
