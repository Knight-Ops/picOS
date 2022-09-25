use crate::services::{
    report::{KeyboardReport, MOD_KEY, RELEASE_ALL},
    task::TaskArgument,
};

use crate::bsp::hal::timer::Timer;
use crate::debug;
use crate::services::usb;

use crate::TIMER;

pub mod condi_sb;
// pub mod condi_untamed;
// pub mod power_sb;

const SKILL_DELAY_US: u32 = 800_000;
const KEY_PRESS_DELAY: u32 = 50_000;
const COOLDOWN_FUDGE_FACTOR: u32 = 120_000;
const HAS_ALACRITY: bool = true;
const HAS_QUICKNESS: bool = true;

fn timer_delay_ms(timer: &Timer, ms: u32) {
    let end = timer.get_counter_low() + (ms * 1000);
    loop {
        if timer.get_counter_low() >= end {
            break;
        }
    }
}

fn timer_delay_us(timer: &Timer, us: u32) {
    let end = timer.get_counter_low() + us;
    loop {
        if timer.get_counter_low() >= end {
            break;
        }
    }
}

struct Skill<'a> {
    name: &'a str,
    activation_keys: KeyboardReport,
    activation: u32,
    cooldown: i32,
    ready_at: i32,
}

impl<'a> Skill<'a> {
    fn new(name: &'a str, activation_keys: KeyboardReport, activation: u32, cooldown: i32) -> Self {
        Self {
            name,
            activation_keys,
            activation: if HAS_QUICKNESS {
                ((activation as f32) * 0.67) as u32
            } else {
                activation
            },
            cooldown: if HAS_ALACRITY {
                ((cooldown as f32) * 0.8) as i32
            } else {
                cooldown
            },
            ready_at: 0,
        }
    }

    fn use_skill(&mut self, timer: &Timer) {
        usb::send(&self.activation_keys);
        timer_delay_us(&timer, KEY_PRESS_DELAY);
        usb::send(&RELEASE_ALL);
        if self.activation != 0 {
            timer_delay_us(&timer, self.activation);
        } else {
            timer_delay_us(&timer, KEY_PRESS_DELAY >> 4);
        }
        self.ready_at = (timer.get_counter_low() as i32)
            .wrapping_add(self.cooldown)
            .wrapping_add(COOLDOWN_FUDGE_FACTOR as i32);
    }

    fn use_skill_if_ready(&mut self, timer: &Timer) -> bool {
        if self.is_ready(&timer) {
            self.use_skill(&timer);
            true
        } else {
            false
        }
    }

    fn blocking_use_skill(&mut self, timer: &Timer) {
        // Block until ready
        while !self.is_ready(&timer) {}

        self.use_skill(&timer);
    }

    fn is_ready(&self, timer: &Timer) -> bool {
        let timer_val = timer.get_counter_low() as i32;
        if timer_val >= self.ready_at {
            debug!(
                "Missed perfect timing by {}ms",
                (timer_val - self.ready_at) / 120_000
            );
            true
        } else {
            false
        }
    }

    fn reduce_cooldown(&mut self, reduction: i32) {
        self.ready_at = self.ready_at.wrapping_sub(reduction);
    }
}

struct WeaponSwap {
    activation_keys: KeyboardReport,
    activation: u32,
    cooldown: i32,
    ready_at: i32,
}

impl WeaponSwap {
    fn new(activation_keys: KeyboardReport) -> Self {
        Self {
            activation_keys,
            activation: 500_000,
            cooldown: 10_500_000,
            ready_at: 0,
        }
    }

    fn use_skill(&mut self, timer: &Timer) {
        usb::send(&self.activation_keys);
        timer_delay_us(&timer, KEY_PRESS_DELAY);
        usb::send(&RELEASE_ALL);
        if self.activation != 0 {
            timer_delay_us(&timer, self.activation);
        } else {
            timer_delay_us(&timer, KEY_PRESS_DELAY >> 4);
        }
        self.ready_at = (timer.get_counter_low() as i32).wrapping_add(self.cooldown);
    }
    fn blocking_use_skill(&mut self, timer: &Timer) {
        // Block until ready
        while !self.is_ready(&timer) {}

        self.use_skill(&timer)
    }

    fn is_ready(&self, timer: &Timer) -> bool {
        if timer.get_counter_low() as i32 >= self.ready_at {
            true
        } else {
            false
        }
    }
}

struct TalentTimer {
    buff_length: i32,
    expired_at: i32,
}

impl TalentTimer {
    fn new(buff_length: i32) -> Self {
        TalentTimer {
            expired_at: 0,
            buff_length,
        }
    }

    fn is_expired(&self, timer: &Timer) -> bool {
        if timer.get_counter_low() as i32 >= self.expired_at {
            true
        } else {
            false
        }
    }

    fn reset(&mut self, timer: &Timer) {
        self.expired_at = (timer.get_counter_low() as i32).wrapping_add(self.buff_length)
    }
}
