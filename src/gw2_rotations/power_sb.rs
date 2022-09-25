use super::{
    timer_delay_ms, timer_delay_us, KeyboardReport, Skill, TalentTimer, TaskArgument, WeaponSwap,
    MOD_KEY, TIMER,
};

pub struct RunRotationArguments {}
unsafe impl Send for RunRotationArguments {}
unsafe impl Sync for RunRotationArguments {}
impl TaskArgument for RunRotationArguments {}

enum ActiveWeaponSet {
    AxeAxe,
    AxeWarhorn,
}

pub fn run_rotation(args: &RunRotationArguments) -> ! {
    let mut timer_lock = TIMER.lock();
    let timer = timer_lock.get_mut().as_ref().unwrap();

    let mut active_weapons = ActiveWeaponSet::AxeWarhorn;

    let mut ricochet = Skill::new(
        KeyboardReport::new(None, Some(0x1e), None, None, None, None, None),
        510_000,
        0,
    );

    let mut splitblade = Skill::new(
        KeyboardReport::new(None, Some(0x1f), None, None, None, None, None),
        850_000,
        4_750_000,
    );

    let mut winters_bite = Skill::new(
        KeyboardReport::new(None, Some(0x20), None, None, None, None, None),
        600_000,
        8_000_000,
    );

    let mut hunters_call = Skill::new(
        KeyboardReport::new(None, Some(0x21), None, None, None, None, None),
        1_100_000,
        20_000_000,
    );

    let mut call_of_the_wild = Skill::new(
        KeyboardReport::new(None, Some(0x22), None, None, None, None, None),
        350_000,
        30_000_000,
    );

    let mut path_of_scars = Skill::new(
        KeyboardReport::new(None, Some(0x21), None, None, None, None, None),
        // This one is acting up, so we are increasing activation time
        875_000,
        12_000_000,
    );

    let mut whirling_defense = Skill::new(
        KeyboardReport::new(None, Some(0x22), None, None, None, None, None),
        3_350_000,
        20_000_000,
    );

    let mut kick = Skill::new(
        KeyboardReport::new(
            Some(MOD_KEY::LEFT_ALT),
            Some(0x1e),
            None,
            None,
            None,
            None,
            None,
        ),
        700_000,
        8_000_000,
    );

    let mut charge = Skill::new(
        KeyboardReport::new(
            Some(MOD_KEY::LEFT_ALT),
            Some(0x1f),
            None,
            None,
            None,
            None,
            None,
        ),
        1_100_000,
        12_000_000,
    );

    let mut worldy_impact = Skill::new(
        KeyboardReport::new(
            Some(MOD_KEY::LEFT_ALT),
            Some(0x20),
            None,
            None,
            None,
            None,
            None,
        ),
        850_000,
        25_000_000,
    );

    let mut signet_of_the_wild = Skill::new(
        KeyboardReport::new(None, Some(0x14), None, None, None, None, None),
        800_000,
        40_000_000,
    );

    let mut frost_trap = Skill::new(
        KeyboardReport::new(None, Some(0x8), None, None, None, None, None),
        550_000,
        30_250_000,
    );

    let mut sic_em = Skill::new(
        KeyboardReport::new(None, Some(0x15), None, None, None, None, None),
        0,
        28_000_000,
    );

    let mut one_wolf_pack = Skill::new(
        KeyboardReport::new(
            Some(MOD_KEY::LEFT_ALT),
            Some(0x14),
            None,
            None,
            None,
            None,
            None,
        ),
        300_000,
        60_000_000,
    );

    let mut weapon_swap = WeaponSwap::new(KeyboardReport::new(
        None,
        Some(0x35),
        None,
        None,
        None,
        None,
        None,
    ));

    timer_delay_ms(&timer, 3000);

    frost_trap.blocking_use_skill(&timer);
    one_wolf_pack.blocking_use_skill(&timer);
    sic_em.blocking_use_skill(&timer);
    charge.blocking_use_skill(&timer);
    hunters_call.blocking_use_skill(&timer);
    splitblade.blocking_use_skill(&timer);
    winters_bite.blocking_use_skill(&timer);

    weapon_swap.blocking_use_skill(&timer);
    active_weapons = ActiveWeaponSet::AxeAxe;

    loop {
        path_of_scars.blocking_use_skill(&timer);
        worldy_impact.blocking_use_skill(&timer);
        kick.blocking_use_skill(&timer);
        splitblade.blocking_use_skill(&timer);
        whirling_defense.blocking_use_skill(&timer);
        winters_bite.blocking_use_skill(&timer);
        charge.blocking_use_skill(&timer);
        splitblade.blocking_use_skill(&timer);
        kick.blocking_use_skill(&timer);
        path_of_scars.blocking_use_skill(&timer);
        weapon_swap.blocking_use_skill(&timer);
        active_weapons = ActiveWeaponSet::AxeWarhorn;
        splitblade.blocking_use_skill(&timer);
        winters_bite.blocking_use_skill(&timer);
        kick.blocking_use_skill(&timer);
        splitblade.blocking_use_skill(&timer);
        charge.blocking_use_skill(&timer);
        frost_trap.use_skill_if_ready(&timer);
        if one_wolf_pack.is_ready(&timer) {
            one_wolf_pack.use_skill(&timer);
        }
        frost_trap.use_skill_if_ready(&timer);
        sic_em.blocking_use_skill(&timer);
        frost_trap.use_skill_if_ready(&timer);
        hunters_call.blocking_use_skill(&timer);
        frost_trap.use_skill_if_ready(&timer);
        splitblade.blocking_use_skill(&timer);
        frost_trap.use_skill_if_ready(&timer);
        winters_bite.blocking_use_skill(&timer);
        frost_trap.use_skill_if_ready(&timer);
        weapon_swap.blocking_use_skill(&timer);
        active_weapons = ActiveWeaponSet::AxeAxe;
    }
}
