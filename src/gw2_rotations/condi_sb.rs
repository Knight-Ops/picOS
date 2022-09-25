use crate::services::{
    report::{KeyboardReport, MOD_KEY, RELEASE_ALL},
    task::TaskArgument,
};

use crate::bsp::hal::timer::Timer;
use crate::services::usb;

use cortex_m::interrupt::{free, Mutex};

use super::{Skill, TalentTimer, HAS_ALACRITY, HAS_QUICKNESS, KEY_PRESS_DELAY, SKILL_DELAY_US};
use crate::task;
use crate::TIMER;

// pub struct RunRotationArguments {}
// unsafe impl Send for RunRotationArguments {}
// unsafe impl Sync for RunRotationArguments {}
// impl TaskArgument for RunRotationArguments {}

#[task]
pub fn run_rotation() -> ! {
    let mut timer_lock = TIMER.lock();
    let timer = timer_lock.get_mut().as_ref().unwrap();

    let mut crossfire = Skill::new(
        "crossfire",
        KeyboardReport::new(None, Some(30), None, None, None, None, None),
        540_000,
        0,
    );

    let mut poison_volley = Skill::new(
        "poison volley",
        KeyboardReport::new(None, Some(31), None, None, None, None, None),
        250_000,
        6_500_000,
    );

    let mut crippling_shot = Skill::new(
        "crippling shot",
        KeyboardReport::new(None, Some(33), None, None, None, None, None),
        540_000,
        9_600_000,
    );

    let mut concussion_shot = Skill::new(
        "concussion shot",
        KeyboardReport::new(None, Some(34), None, None, None, None, None),
        250_000,
        20_000_000,
    );

    let mut maul = Skill::new(
        "maul",
        KeyboardReport::new(
            Some(MOD_KEY::LEFT_ALT),
            Some(31),
            None,
            None,
            None,
            None,
            None,
        ),
        540_000,
        16_000_000,
    );

    let mut primal_cry = Skill::new(
        "primal cry",
        KeyboardReport::new(
            Some(MOD_KEY::LEFT_ALT),
            Some(32),
            None,
            None,
            None,
            None,
            None,
        ),
        1_250_000,
        20_000_000,
    );

    let mut sharpening_stone = Skill::new(
        "sharpening stone",
        KeyboardReport::new(None, Some(20), None, None, None, None, None),
        0,
        30_000_000,
    );

    let mut vulture_stance = Skill::new(
        "vulture stance",
        KeyboardReport::new(None, Some(8), None, None, None, None, None),
        0,
        30_000_000,
    );

    let mut vipers_nest = Skill::new(
        "vipers nest",
        KeyboardReport::new(None, Some(21), None, None, None, None, None),
        0,
        20_000_000,
    );

    let mut one_wolf_pack = Skill::new(
        "one wolf pack",
        KeyboardReport::new(
            Some(MOD_KEY::LEFT_ALT),
            Some(20),
            None,
            None,
            None,
            None,
            None,
        ),
        0,
        80_000_000,
    );

    let mut twice_as_vicious = TalentTimer::new(10_000_000);

    // delay.delay_ms(3000);
    timer_delay_ms(&timer, 3000);

    loop {
        if one_wolf_pack.is_ready(&timer) {
            one_wolf_pack.use_skill(&timer);
        } else if vipers_nest.is_ready(&timer) {
            vipers_nest.use_skill(&timer)
        } else if vulture_stance.is_ready(&timer) {
            vulture_stance.use_skill(&timer)
            // usb::send(&KeyboardReport::new(
            //     None,
            //     Some(20),
            //     Some(8),
            //     Some(21),
            //     None,
            //     None,
            //     None,
            // ));
            // timer_delay_us(&timer, KEY_PRESS_DELAY);
            // usb::send(&RELEASE_ALL);
            // timer_delay_us(&timer, KEY_PRESS_DELAY >> 2);
            // vulture_stance.ready_at =
            //     (timer.get_counter_low() as i32).wrapping_add(vulture_stance.cooldown);
        } else if sharpening_stone.is_ready(&timer) {
            sharpening_stone.use_skill(&timer)
        } else if poison_volley.is_ready(&timer) {
            poison_volley.use_skill(&timer);
        } else if crippling_shot.is_ready(&timer) {
            crippling_shot.use_skill(&timer);
        } else if maul.is_ready(&timer) {
            maul.use_skill(&timer);
        } else if twice_as_vicious.is_expired(&timer) {
            if primal_cry.is_ready(&timer) {
                primal_cry.use_skill(&timer);
                twice_as_vicious.reset(&timer);
            } else if concussion_shot.is_ready(&timer) {
                concussion_shot.use_skill(&timer);
                twice_as_vicious.reset(&timer);
            }
        } else {
            // crossfire.use_skill(&timer, &mut delay)
            timer_delay_ms(&timer, 540);
        }
    }
}

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
