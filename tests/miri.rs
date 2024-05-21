#![cfg(all(not(loom), feature = "serde"))]

use crate::random_action::{ActionPeekWeights, EventWeights};
use blazemap::{define_key_wrapper, define_key_wrapper_bounded, prelude::BlazeMap};
use rand::{prelude::StdRng, random, Rng, SeedableRng};
use std::io::Write;

mod random_action;

#[test]
fn key_wrapper() {
    define_key_wrapper! {
        struct Id(String);
        Derive(as for Original Type): {
            Default,
            Debug,
            Display,
            Ord,
            Serialize
        }
    }
    let seed: u64 = random();
    println!("`key_wrapper` random seed: {seed}");
    std::io::stdout().flush().unwrap();
    let mut rng = StdRng::seed_from_u64(seed);

    let mut input_combinations = Vec::with_capacity(6);
    for num_random_digits in 1..=3 {
        for _ in 0..2 {
            let num_actions: usize = 2_000;
            let seed: u64 = rng.gen_range(0..=u64::MAX);
            input_combinations.push((num_random_digits, num_actions, seed));
        }
    }

    #[allow(unused_variables)]
    input_combinations.iter().copied().enumerate().for_each(
        |(i, (num_random_digits, num_actions, seed))| {
            let mut rng = StdRng::seed_from_u64(seed);
            let mut map = BlazeMap::<Id, String>::new();
            for j in 1..=num_actions {
                #[cfg(miri)]
                if j % 100 == 1 {
                    println!(
                        "`key_wrapper` epoch: [{i}/{combs}], action_iter: [{j}/{num_actions}]",
                        combs = input_combinations.len()
                    );
                    std::io::stdout().flush().unwrap();
                }
                let action =
                    ActionPeekWeights::new(&num_random_digits, &mut rng).generate(&mut rng);
                action.apply("key_wrapper", &mut rng, &mut map, Id::new);
            }
        },
    );
}

#[test]
fn key_wrapper_bounded() {
    define_key_wrapper_bounded! {
        struct Id(String);
        MAX_CAP = 10_000;
        Derive(as for Original Type): {
            Default,
            Debug,
            Display,
            Ord,
            Serialize
        }
    }
    let seed: u64 = random();
    println!("`key_wrapper_bounded` random seed: {seed}");
    std::io::stdout().flush().unwrap();
    let mut rng = StdRng::seed_from_u64(seed);

    let mut input_combinations = Vec::with_capacity(6);
    for num_random_digits in 1..=3 {
        for _ in 0..2 {
            let num_actions: usize = 2_000;
            let seed: u64 = rng.gen_range(0..=u64::MAX);
            input_combinations.push((num_random_digits, num_actions, seed));
        }
    }

    #[allow(unused_variables)]
    input_combinations
        .iter()
        .copied()
        .enumerate()
        .for_each(|(i, (num_random_digits, num_actions, seed))| {
            let mut rng = StdRng::seed_from_u64(seed);
            let mut map = BlazeMap::<Id, String>::new();
            for j in 1..=num_actions {
                #[cfg(miri)]
                if j % 100 == 1 {
                    println!(
                        "`key_wrapper_bounded` epoch: [{i}/{combs}], action_iter: [{j}/{num_actions}]",
                        combs = input_combinations.len()
                    );
                    std::io::stdout().flush().unwrap();
                }
                let action =
                    ActionPeekWeights::new(&num_random_digits, &mut rng).generate(&mut rng);
                action.apply("key_wrapper_bounded", &mut rng, &mut map, Id::new);
            }
        });
}
