use core::{
    cell::RefCell,
    sync::atomic::{AtomicU32, Ordering},
};

use critical_section::Mutex;
use fugit::{Duration, Instant};
use microbit::{
    hal::{rtc::RtcInterrupt, Rtc},
    pac::{interrupt, NVIC, RTC0},
};

type TickInstant = Instant<u64, 1, 32768>;
type TickDuration = Duration<u64, 1, 32768>;

pub struct Timer {
    end_time: TickInstant,
}

impl Timer {
    pub fn new(duration: TickDuration) -> Self {
        Self {
            end_time: Ticker::now() + duration,
        }
    }

    pub fn is_ready(&self) -> bool {
        Ticker::now() >= self.end_time
    }
}

static TICKER: Ticker = Ticker {
    ovr_count: AtomicU32::new(0),
    rtc: Mutex::new(RefCell::new(None)),
};

pub struct Ticker {
    // Overflow counter
    ovr_count: AtomicU32,
    rtc: Mutex<RefCell<Option<Rtc<RTC0>>>>,
}

impl Ticker {
    pub fn init(rtc0: RTC0, nvic: &mut NVIC) {
        let mut rtc = Rtc::new(rtc0, 0).unwrap();
        rtc.enable_counter();
        rtc.enable_event(RtcInterrupt::Overflow);
        rtc.enable_interrupt(RtcInterrupt::Overflow, Some(nvic));
        critical_section::with(|cs| TICKER.rtc.replace(cs, Some(rtc))); // Temporarily disables interrupts
    }

    pub fn now() -> TickInstant {
        let ticks = {
            loop {
                let ovr_before = TICKER.ovr_count.load(Ordering::SeqCst);
                let counter = critical_section::with(|cs| {
                    TICKER.rtc.borrow_ref(cs).as_ref().unwrap().get_counter()
                });
                let ovr_after = TICKER.ovr_count.load(Ordering::SeqCst);
                if ovr_before == ovr_after {
                    break (ovr_before as u64) << 24 | (counter as u64);
                }
            }
        };
        TickInstant::from_ticks(ticks)
    }
}

/// Interrupt event handler for RTC0
#[interrupt]
fn RTC0() {
    // Check if the interruption was a result of overflows
    critical_section::with(|cs| {
        let mut rm_rtc = TICKER.rtc.borrow_ref_mut(cs);
        let rtc = rm_rtc.as_mut().unwrap();
        if rtc.is_event_triggered(RtcInterrupt::Overflow) {
            rtc.reset_event(RtcInterrupt::Overflow);
            TICKER.ovr_count.fetch_add(1, Ordering::Relaxed);
        }
        // Clearing the event flag can take up to 4 clock cycles:
        // (see nRF52833 Product Specification section 6.1.8)
        // this should do that...
        let _ = rtc.is_event_triggered(RtcInterrupt::Overflow);
    });
}
