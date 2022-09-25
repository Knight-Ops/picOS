use super::{
    timer_delay_ms, timer_delay_us, KeyboardReport, Skill, TalentTimer, TaskArgument, WeaponSwap,
    MOD_KEY, TIMER,
};
use crate::Box;

use crate::bsp::hal::timer::Timer;

pub struct RunRotationArguments<'a> {
    timer: &'a Timer,
}
unsafe impl<'a> Send for RunRotationArguments<'a> {}
unsafe impl<'a> Sync for RunRotationArguments<'a> {}
impl<'a> TaskArgument for RunRotationArguments<'a> {}

enum ActiveWeaponSet {
    AxeAxe,
    AxeWarhorn,
}

pub fn run_rotation(args: Box<RunRotationArguments>) -> ! {
    let mut timer_lock = TIMER.lock();
    let timer = timer_lock.get_mut().as_ref().unwrap();

    // let mut skills = alloc::vec::Vec::new();

    let mut active_weapons = ActiveWeaponSet::AxeWarhorn;

    let mut ricochet = Skill::new(
        "ricochet",
        KeyboardReport::new(None, Some(0x1e), None, None, None, None, None),
        510_000,
        0,
    );

    //skills.push(ricochet);

    let mut sundering_volley = Skill::new(
        "sundering volley",
        KeyboardReport::new(None, Some(0x1e), None, None, None, None, None),
        760_000,
        0,
    );
    // //skills.push(sundering_volley);

    let mut splitblade = Skill::new(
        "splitblade",
        KeyboardReport::new(None, Some(0x1f), None, None, None, None, None),
        850_000,
        4_750_000,
    );
    //skills.push(splitblade);

    let mut winters_bite = Skill::new(
        "winter's bite",
        KeyboardReport::new(None, Some(0x20), None, None, None, None, None),
        600_000,
        8_000_000,
    );
    //skills.push(winters_bite);

    let mut path_of_scars = Skill::new(
        "path of scars",
        KeyboardReport::new(None, Some(0x21), None, None, None, None, None),
        // This one is acting up, so we are increasing activation time
        875_000,
        12_000_000,
    );
    //skills.push(path_of_scars);

    let mut whirling_defense = Skill::new(
        "whirling defense",
        KeyboardReport::new(None, Some(0x22), None, None, None, None, None),
        // Whirling defense is just filler, so we don't need to wait on the whole activation
        500_000,
        20_000_000,
    );
    //skills.push(whirling_defense);

    let mut venomous_outburst = Skill::new(
        "venomous outburst",
        KeyboardReport::new(
            Some(MOD_KEY::LEFT_ALT),
            Some(0x1e),
            None,
            None,
            None,
            None,
            None,
        ),
        200_000,
        10_000_000,
    );

    //skills.push(venomous_outburst);

    let mut rending_vines = Skill::new(
        "rending vines",
        KeyboardReport::new(
            Some(MOD_KEY::LEFT_ALT),
            Some(0x1f),
            None,
            None,
            None,
            None,
            None,
        ),
        200_000,
        15_000_000,
    );
    //skills.push(rending_vines);

    let mut enveloping_haze = Skill::new(
        "enveloping haze",
        KeyboardReport::new(
            Some(MOD_KEY::LEFT_ALT),
            Some(0x20),
            None,
            None,
            None,
            None,
            None,
        ),
        200_000,
        25_000_000,
    );
    //skills.push(enveloping_haze);

    let mut heal_as_one = Skill::new(
        "we heal as one",
        KeyboardReport::new(None, Some(0x6), None, None, None, None, None),
        1_000_000,
        16_000_000,
    );
    //skills.push(heal_as_one);

    let mut call_lightning = Skill::new(
        "call lightning",
        KeyboardReport::new(None, Some(0x14), None, None, None, None, None),
        300_000,
        20_000_000,
    );
    //skills.push(call_lightning);

    let mut exploding_spores = Skill::new(
        "exploding spores",
        KeyboardReport::new(None, Some(0x8), None, None, None, None, None),
        300_000,
        25_250_000,
    );
    //skills.push(exploding_spores);

    let mut sharpening_stone = Skill::new(
        "sharpening stone",
        KeyboardReport::new(None, Some(0x15), None, None, None, None, None),
        0,
        30_000_000,
    );
    //skills.push(sharpening_stone);

    let mut entangle = Skill::new(
        "entangle",
        KeyboardReport::new(
            Some(MOD_KEY::LEFT_ALT),
            Some(0x14),
            None,
            None,
            None,
            None,
            None,
        ),
        800_000,
        60_000_000,
    );
    //skills.push(entangle);

    let mut weapon_swap = WeaponSwap::new(KeyboardReport::new(
        None,
        Some(0x35),
        None,
        None,
        None,
        None,
        None,
    ));

    let mut ambush = TalentTimer::new(15_000_000);
    let mut unleash = Skill::new(
        "unleash",
        KeyboardReport::new(
            Some(MOD_KEY::LEFT_ALT),
            Some(0x22),
            None,
            None,
            None,
            None,
            None,
        ),
        50_000,
        1_000_000,
    );

    timer_delay_ms(&timer, 3000);

    loop {
        splitblade.use_skill(&timer);
        winters_bite.use_skill(&timer);
        path_of_scars.use_skill(&timer);
        exploding_spores.use_skill(&timer);
        splitblade.use_skill(&timer);
        // filler
        select_filler(
            &timer,
            &mut sundering_volley,
            &mut ambush,
            &mut entangle,
            &mut whirling_defense,
            &mut unleash,
        );
        splitblade.use_skill(&timer);
        winters_bite.use_skill(&timer);
        //filler
        select_filler(
            &timer,
            &mut sundering_volley,
            &mut ambush,
            &mut entangle,
            &mut whirling_defense,
            &mut unleash,
        );
        path_of_scars.use_skill(&timer);
        call_lightning.use_skill(&timer);
        splitblade.use_skill(&timer);
        heal_as_one.use_skill(&timer);
    }

    // ~16k dps
    // loop {
    //     let mut skill_used = false;
    //     if weapon_swap.is_ready(&timer) {
    //         weapon_swap.use_skill(&timer);
    //     }
    //     skills
    //         .iter_mut()
    //         .filter(|x| x.name == "sharpening stone")
    //         .for_each(|x| {
    //             x.use_skill_if_ready(&timer);
    //         });
    //     skills
    //         .iter_mut()
    //         .filter(|x| x.name == "venomous outburst")
    //         .for_each(|x| {
    //             x.use_skill_if_ready(&timer);
    //         });
    //     skills
    //         .iter_mut()
    //         .filter(|x| x.name == "rending vines")
    //         .for_each(|x| {
    //             x.use_skill_if_ready(&timer);
    //         });
    //     skills
    //         .iter_mut()
    //         .filter(|x| x.name == "enveloping haze")
    //         .for_each(|x| {
    //             x.use_skill_if_ready(&timer);
    //         });

    //     // Actual rotation
    //     skills
    //         .iter_mut()
    //         .filter(|x| x.name == "splitblade")
    //         .for_each(|x| {
    //             if x.use_skill_if_ready(&timer) {
    //                 skill_used = true;
    //             }
    //         });
    //     if skill_used {
    //         continue;
    //     }

    //     skills
    //         .iter_mut()
    //         .filter(|x| x.name == "winter's bite")
    //         .for_each(|x| {
    //             if x.use_skill_if_ready(&timer) {
    //                 skill_used = true;
    //             }
    //         });
    //     if skill_used {
    //         continue;
    //     }

    //     skills
    //         .iter_mut()
    //         .filter(|x| x.name == "path of scars")
    //         .for_each(|x| {
    //             if x.use_skill_if_ready(&timer) {
    //                 skill_used = true;
    //             }
    //         });
    //     if skill_used {
    //         skills.iter_mut().for_each(|x| x.reduce_cooldown(4_000_000));
    //         continue;
    //     }

    //     skills
    //         .iter_mut()
    //         .filter(|x| x.name == "exploding spores")
    //         .for_each(|x| {
    //             if x.use_skill_if_ready(&timer) {
    //                 skill_used = true;
    //             }
    //         });
    //     if skill_used {
    //         skills.iter_mut().for_each(|x| x.reduce_cooldown(4_000_000));
    //         continue;
    //     }

    //     skills
    //         .iter_mut()
    //         .filter(|x| x.name == "call lightning")
    //         .for_each(|x| {
    //             if x.use_skill_if_ready(&timer) {
    //                 skill_used = true;
    //             }
    //         });
    //     if skill_used {
    //         skills.iter_mut().for_each(|x| x.reduce_cooldown(4_000_000));
    //         continue;
    //     }

    //     if ambush.is_expired(&timer) {
    //         unleash.use_skill(&timer);
    //         sundering_volley.use_skill(&timer);
    //         ambush.reset(&timer);
    //         unleash.use_skill(&timer);
    //         continue;
    //     }

    //     skills
    //         .iter_mut()
    //         .filter(|x| x.name == "entangle")
    //         .for_each(|x| {
    //             if x.use_skill_if_ready(&timer) {
    //                 skill_used = true;
    //             }
    //         });
    //     if skill_used {
    //         continue;
    //     }

    //     skills
    //         .iter_mut()
    //         .filter(|x| x.name == "whirling defense")
    //         .for_each(|x| {
    //             if x.use_skill_if_ready(&timer) {
    //                 skill_used = true;
    //             }
    //         });
    //     if skill_used {
    //         continue;
    //     }
    // }
}

fn select_filler(
    timer: &Timer,
    ambush: &mut Skill,
    ambush_timer: &mut TalentTimer,
    entangle: &mut Skill,
    whirling_defense: &mut Skill,
    unleash: &mut Skill,
) {
    if ambush_timer.is_expired(&timer) {
        unleash.use_skill(&timer);
        ambush.use_skill(&timer);
        ambush_timer.reset(&timer);
        unleash.use_skill(&timer);
    }
    entangle.use_skill(&timer)
}
