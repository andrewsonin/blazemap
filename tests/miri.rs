#![cfg(feature = "serde")]
#![allow(clippy::explicit_write)]
#![allow(clippy::explicit_counter_loop)]

use std::io::Write;

use rand::prelude::StdRng;
use rand::{random, Rng, SeedableRng};

use blazemap::prelude::BlazeMap;
use blazemap::{define_key_wrapper, define_key_wrapper_bounded};

use crate::random_action::{ActionPeekWeights, EventWeights};

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
    writeln!(std::io::stdout(), "`key_wrapper` random seed: {seed}").unwrap();
    std::io::stdout().flush().unwrap();
    let mut rng = StdRng::seed_from_u64(seed);

    let mut input_combinations = Vec::with_capacity(300);
    for num_random_digits in 1..=3 {
        for _ in 0..3 {
            let num_actions: usize = rng.gen_range(0..10_000);
            let seed: u64 = rng.gen_range(0..=u64::MAX);
            input_combinations.push((num_random_digits, num_actions, seed));
        }
    }

    #[allow(unused_variables)]
    let mut i = 0;
    input_combinations
        .iter()
        .copied()
        .for_each(|(num_random_digits, num_actions, seed)| {
            i += 1;
            let mut rng = StdRng::seed_from_u64(seed);
            let mut map = BlazeMap::<Id, String>::new();
            #[allow(unused_variables)]
            let mut j = 0;
            for _ in 1..=num_actions {
                #[cfg(miri)]
                if j % 100 == 1 {
                    writeln!(
                        std::io::stdout(),
                        "`key_wrapper` epoch: [{i}/{combs}], action_iter: [{j}/{num_actions}]",
                        combs = input_combinations.len()
                    )
                    .unwrap();
                    std::io::stdout().flush().unwrap();
                }
                j += 1;
                let action =
                    ActionPeekWeights::new(&num_random_digits, &mut rng).generate(&mut rng);
                action.apply("key_wrapper", &mut rng, &mut map, Id::new);
            }
        });
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
    writeln!(
        std::io::stdout(),
        "`key_wrapper_bounded` random seed: {seed}"
    )
    .unwrap();
    std::io::stdout().flush().unwrap();
    let mut rng = StdRng::seed_from_u64(seed);

    let mut input_combinations = Vec::with_capacity(300);
    for num_random_digits in 1..=3 {
        for _ in 0..3 {
            let num_actions: usize = rng.gen_range(0..10_000);
            let seed: u64 = rng.gen_range(0..=u64::MAX);
            input_combinations.push((num_random_digits, num_actions, seed));
        }
    }

    #[allow(unused_variables)]
    let mut i = 0;
    input_combinations
        .iter()
        .copied()
        .for_each(|(num_random_digits, num_actions, seed)| {
            i += 1;
            let mut rng = StdRng::seed_from_u64(seed);
            let mut map = BlazeMap::<Id, String>::new();
            #[allow(unused_variables)]
                let mut j = 0;
            for _ in 1..=num_actions {
                #[cfg(miri)]
                if j % 100 == 1 {
                    writeln!(
                        std::io::stdout(),
                        "`key_wrapper_bounded` epoch: [{i}/{combs}], action_iter: [{j}/{num_actions}]",
                        combs = input_combinations.len()
                    ).unwrap();
                    std::io::stdout().flush().unwrap();
                }
                j += 1;
                let action =
                    ActionPeekWeights::new(&num_random_digits, &mut rng).generate(&mut rng);
                action.apply("key_wrapper_bounded", &mut rng, &mut map, Id::new);
            }
        });
}
