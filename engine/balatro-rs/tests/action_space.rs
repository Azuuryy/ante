//! Random games through the fixed-size action space, replayed from fixed seeds.
//! These seeds used to end on a move the game refused, or stay in a Tarot target
//! stage that no unmasked move could leave.

use balatro_rs::{
    config::{Config, RngMode},
    game::Game,
};
use rand::{rngs::StdRng, Rng, SeedableRng};

fn play_through_action_space(seed: u64) -> Result<(), String> {
    let mut game = Game::new(Config {
        seed: Some(seed),
        rng_mode: RngMode::Fast,
        ..Config::default()
    });
    let mut choices = StdRng::seed_from_u64(seed);
    game.start();
    for step in 0..20_000 {
        if game.is_over() {
            return Ok(());
        }
        let space = game.gen_action_space();
        let mask = space.to_vec();
        let unmasked: Vec<usize> = (0..mask.len()).filter(|&i| mask[i] == 1).collect();
        if unmasked.is_empty() {
            return Err(format!("step {step}: no unmasked move in {:?}", game.stage));
        }
        let index = unmasked[choices.gen_range(0..unmasked.len())];
        let action = space
            .to_action(index, &game)
            .map_err(|e| format!("step {step}: {e:?}"))?;
        game.handle_action(action.clone())
            .map_err(|e| format!("step {step}: {action:?} refused: {e:?} in {:?}", game.stage))?;
    }
    Err(format!("did not finish, still in {:?}", game.stage))
}

#[test]
fn seeds_that_used_to_stall_or_fail_now_finish() {
    for seed in [127, 376, 459, 567, 808, 848, 1050, 1155] {
        assert_eq!(play_through_action_space(seed), Ok(()), "seed {seed}");
    }
}
