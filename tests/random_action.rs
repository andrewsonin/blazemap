#![cfg(feature = "serde")]
#![allow(renamed_and_removed_lints)]
#![allow(illegal_floating_point_literal_pattern)]
#![allow(missing_debug_implementations)]
#![allow(unreachable_pub)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::enum_variant_names)]
#![allow(clippy::explicit_write)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::drop_non_drop)]

use std::fmt::{Debug, Formatter, Write};

use rand::Rng;

use blazemap::{
    prelude::BlazeMap,
    traits::{BlazeMapId, BlazeMapIdStatic},
};

#[derive(Debug)]
pub enum Action<K, V: Clone> {
    Clear,
    ShrinkToFit,
    Iter(Iter),
    IterMut(IterMut),
    Keys(Iter),
    Values(Iter),
    ValuesMut(IterMut),
    Drain(IterMut),
    ContainsKey { key: K },
    Get { key: K },
    GetMut { key: K },
    Insert { key: K, value: V },
    Remove { key: K },
    Entry { key: K, event: Entry<V> },
    IntoKeys(IterMut),
    IntoValues(IterMut),
    IntoIter(IterMut),
    Debug,
    Serialize,
    Drop,
}

macro_rules! process_iter_action {
    ($log_suffix:ident, $rng:ident, $event:ident, $iterator:ident) => {
        'scope: {
            match $event {
                Iter::Next => {
                    if let Some(v) = $iterator.next() {
                        let mut io = std::io::sink();
                        write!(io, "{:?}", v).unwrap();
                    }
                }
                Iter::Len => {
                    let _ = $iterator.len();
                }
                Iter::Clone => $iterator = $iterator.clone(),
                Iter::Debug => {
                    let mut io = std::io::sink();
                    write!(io, "{:?}", $iterator).unwrap();
                }
                Iter::Drop => {
                    drop($iterator);
                    break 'scope;
                }
            }
            while $iterator.len() != 0 {
                let event = IterPeekWeights::new(&(), $rng).generate($rng);
                #[cfg(all(miri, feature = "miri_action_log"))]
                {
                    println!("{} {:?}", $log_suffix, $event);
                    std::io::stdout().flush().unwrap();
                };
                match event {
                    Iter::Next => {
                        if let Some(v) = $iterator.next() {
                            let mut io = std::io::sink();
                            write!(io, "{:?}", v).unwrap();
                        }
                    }
                    Iter::Len => {
                        let _ = $iterator.len();
                    }
                    Iter::Clone => $iterator = $iterator.clone(),
                    Iter::Debug => {
                        let mut io = std::io::sink();
                        write!(io, "{:?}", $iterator).unwrap();
                    }
                    Iter::Drop => {
                        drop($iterator);
                        break 'scope;
                    }
                }
            }
        }
    };
}

macro_rules! process_iter_mut_action {
    ($log_suffix:ident, $rng:ident, $event:ident, $iterator:ident) => {
        'scope: {
            match $event {
                IterMut::Next => {
                    if let Some(v) = $iterator.next() {
                        let mut io = std::io::sink();
                        write!(io, "{:?}", v).unwrap();
                    }
                }
                IterMut::Len => {
                    let _ = $iterator.len();
                }
                IterMut::Debug => {
                    let mut io = std::io::sink();
                    write!(io, "{:?}", $iterator).unwrap();
                }
                IterMut::Drop => {
                    drop($iterator);
                    break 'scope;
                }
            }
            while $iterator.len() != 0 {
                let event = IterMutPeekWeights::new(&(), $rng).generate($rng);
                #[cfg(all(miri, target = "miri_action_log"))]
                {
                    println!("{} {:?}", $log_suffix, $event);
                    std::io::stdout().flush().unwrap();
                };
                match event {
                    IterMut::Next => {
                        if let Some(v) = $iterator.next() {
                            let mut io = std::io::sink();
                            write!(io, "{:?}", v).unwrap();
                        }
                    }
                    IterMut::Len => {
                        let _ = $iterator.len();
                    }
                    IterMut::Debug => {
                        let mut io = std::io::sink();
                        write!(io, "{:?}", $iterator).unwrap();
                    }
                    IterMut::Drop => {
                        drop($iterator);
                        break 'scope;
                    }
                }
            }
        }
    };
}

impl Action<String, String> {
    #[inline]
    #[allow(unused_variables)]
    pub fn apply<I>(
        self,
        log_suffix: &str,
        rng: &mut impl Rng,
        map: &mut BlazeMap<I, String>,
        key_to_id: impl FnOnce(String) -> I,
    ) where
        I: BlazeMapId<OrigType = String> + BlazeMapIdStatic + Debug,
    {
        use std::io::Write;
        #[cfg(all(miri, feature = "miri_action_log"))]
        {
            println!("{log_suffix} {self:?}");
            std::io::stdout().flush().unwrap();
        };
        match self {
            Action::Clear => map.clear(),
            Action::ShrinkToFit => map.shrink_to_fit(),
            Action::Iter(event) => {
                let mut iterator = map.iter();
                process_iter_action!(log_suffix, rng, event, iterator);
            }
            Action::IterMut(event) => {
                let mut iterator = map.iter_mut();
                process_iter_mut_action!(log_suffix, rng, event, iterator);
            }
            Action::Keys(event) => {
                let mut iterator = map.keys();
                process_iter_action!(log_suffix, rng, event, iterator);
            }
            Action::Values(event) => {
                let mut iterator = map.values();
                process_iter_action!(log_suffix, rng, event, iterator);
            }
            Action::ValuesMut(event) => {
                let mut iterator = map.values_mut();
                process_iter_mut_action!(log_suffix, rng, event, iterator);
            }
            Action::Drain(event) => {
                let mut iterator = map.drain();
                process_iter_mut_action!(log_suffix, rng, event, iterator);
            }
            Action::ContainsKey { key } => {
                let mut io = std::io::sink();
                write!(io, "{:?}", map.contains_key(key_to_id(key))).unwrap();
            }
            Action::Get { key } => {
                let mut io = std::io::sink();
                write!(io, "{:?}", map.get(key_to_id(key))).unwrap();
            }
            Action::GetMut { key } => {
                let mut io = std::io::sink();
                write!(io, "{:?}", map.get_mut(key_to_id(key))).unwrap();
            }
            Action::Insert { key, value } => {
                let mut io = std::io::sink();
                write!(io, "{:?}", map.insert(key_to_id(key), value)).unwrap();
            }
            Action::Remove { key } => {
                let mut io = std::io::sink();
                write!(io, "{:?}", map.remove(key_to_id(key))).unwrap();
            }
            Action::Entry { key, event } => {
                let mut io = std::io::sink();
                let entry = map.entry(key_to_id(key));
                match event {
                    Entry::OrInsert { value } => {
                        write!(io, "{}", entry.or_insert(value)).unwrap();
                    }
                    Entry::OrInsertWith { default } => {
                        write!(io, "{}", entry.or_insert_with(default)).unwrap();
                    }
                    Entry::Key => {
                        write!(io, "{:?}", entry.key()).unwrap();
                    }
                    Entry::AndModify { f } => {
                        let _ = entry.and_modify(f);
                    }
                    Entry::OrDefault => {
                        write!(io, "{}", entry.or_default()).unwrap();
                    }
                    Entry::EntryMatch(event) => match entry {
                        blazemap::collections::blazemap::Entry::Occupied(mut entry) => {
                            match event.on_occupied {
                                OccupiedEntry::Key => write!(io, "{:?}", entry.key()).unwrap(),
                                OccupiedEntry::RemoveEntry => {
                                    write!(io, "{:?}", entry.remove_entry()).unwrap();
                                }
                                OccupiedEntry::Get => write!(io, "{}", entry.get()).unwrap(),
                                OccupiedEntry::GetMut => write!(io, "{}", entry.get_mut()).unwrap(),
                                OccupiedEntry::IntoMut => {
                                    write!(io, "{}", entry.into_mut()).unwrap();
                                }
                                OccupiedEntry::Insert { value } => {
                                    write!(io, "{}", entry.insert(value)).unwrap();
                                }
                                OccupiedEntry::Remove => write!(io, "{}", entry.remove()).unwrap(),
                                OccupiedEntry::Drop => drop(entry),
                            }
                        }
                        blazemap::collections::blazemap::Entry::Vacant(entry) => {
                            match event.on_vacant {
                                VacantEntry::Key => write!(io, "{:?}", entry.key()).unwrap(),
                                VacantEntry::Insert { value } => {
                                    write!(io, "{:?}", entry.insert(value)).unwrap();
                                }
                                VacantEntry::Drop => drop(entry),
                            }
                        }
                    },
                    Entry::Drop => drop(entry),
                }
            }
            Action::IntoKeys(event) => {
                let old = std::mem::replace(map, BlazeMap::new());
                let mut iterator = old.into_keys();
                process_iter_mut_action!(log_suffix, rng, event, iterator);
            }
            Action::IntoValues(event) => {
                let old = std::mem::replace(map, BlazeMap::new());
                let mut iterator = old.into_values();
                process_iter_mut_action!(log_suffix, rng, event, iterator);
            }
            Action::IntoIter(event) => {
                let old = std::mem::replace(map, BlazeMap::new());
                let mut iterator = old.into_iter();
                process_iter_mut_action!(log_suffix, rng, event, iterator);
            }
            Action::Debug => {
                let mut io = std::io::sink();
                write!(io, "{map:?}").unwrap();
            }
            Action::Serialize => {
                let mut io = std::io::sink();
                write!(io, "{}", serde_json::to_string(&map).unwrap()).unwrap();
            }
            Action::Drop => {
                let old = std::mem::replace(map, BlazeMap::new());
                drop(old);
            }
        }
    }
}

#[inline]
fn generate_random_string(num_digits: u8, rng: &mut impl Rng) -> String {
    const END: &str = " -----------------------------";
    let mut result = String::with_capacity(num_digits as usize + END.len());
    for _ in 0..num_digits {
        result.write_char(rng.gen_range('0'..='9')).unwrap();
    }
    result.write_str(END).unwrap();
    result
}

#[derive(Debug, Clone)]
pub enum Iter {
    Next,
    Len,
    Clone,
    Debug,
    Drop,
}

#[derive(Debug, Clone)]
pub enum IterMut {
    Next,
    Len,
    Debug,
    Drop,
}

pub enum Entry<V> {
    OrInsert { value: V },
    OrInsertWith { default: Box<dyn FnOnce() -> V> },
    Key,
    AndModify { f: Box<dyn FnOnce(&mut V)> },
    OrDefault,
    EntryMatch(EntryMatch<V>),
    Drop,
}

impl<V: Debug + Clone> Debug for Entry<V> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        #[derive(Debug)]
        #[allow(dead_code)]
        enum Helper<V> {
            OrInsert { value: V },
            OrInsertWith,
            Key,
            AndModify,
            OrDefault,
            EntryMatch(EntryMatch<V>),
            Drop,
        }
        let res = match self {
            Entry::OrInsert { value } => Helper::OrInsert {
                value: value.clone(),
            },
            Entry::OrInsertWith { .. } => Helper::OrInsertWith,
            Entry::Key => Helper::Key,
            Entry::AndModify { .. } => Helper::AndModify,
            Entry::OrDefault => Helper::OrDefault,
            Entry::EntryMatch(value) => Helper::EntryMatch(value.clone()),
            Entry::Drop => Helper::Drop,
        };
        res.fmt(f)
    }
}

#[derive(Debug, Clone)]
pub struct EntryMatch<V> {
    on_occupied: OccupiedEntry<V>,
    on_vacant: VacantEntry<V>,
}

#[derive(Debug, Clone)]
pub enum OccupiedEntry<V> {
    Key,
    RemoveEntry,
    Get,
    GetMut,
    IntoMut,
    Insert { value: V },
    Remove,
    Drop,
}

#[derive(Debug, Clone)]
pub enum VacantEntry<V> {
    Key,
    Insert { value: V },
    Drop,
}

pub trait EventWeights {
    type Config;
    type Event;
    fn new(config: &Self::Config, rng: &mut impl Rng) -> Self;
    fn generate(&self, rng: &mut impl Rng) -> Self::Event;
}

pub struct ActionPeekWeights {
    random_string_len: u8,
}

struct IterPeekWeights;

struct IterMutPeekWeights;

struct EntryPeekWeights {
    random_string_len: u8,
}

struct OccupiedEntryPeekWeights {
    random_string_len: u8,
}

struct VacantEntryPeekWeights {
    random_string_len: u8,
}

impl ActionPeekWeights {
    const CLEAR: f64 = 0.4;
    const SHRINK_TO_FIT: f64 = 5.0;
    const ITER: f64 = 10.0;
    const ITER_MUT: f64 = 15.0;
    const KEYS: f64 = 20.0;
    const VALUES: f64 = 25.0;
    const VALUES_MUT: f64 = 30.0;
    const DRAIN: f64 = 31.0;
    const CONTAINS_KEY: f64 = 40.0;
    const GET: f64 = 50.0;
    const GET_MUT: f64 = 60.0;
    const INSERT: f64 = 70.0;
    const REMOVE: f64 = 80.0;
    const ENTRY: f64 = 100.0;
    const INTO_KEYS: f64 = 101.0;
    const INTO_VALUES: f64 = 102.0;
    const INTO_ITER: f64 = 103.0;
    const DEBUG: f64 = 120.0;
    const SERIALIZE: f64 = 125.0;
    const DROP: f64 = 125.5;

    const MAX_WEIGHT: f64 = Self::DROP;
}

impl EventWeights for ActionPeekWeights {
    type Config = u8;
    type Event = Action<String, String>;

    #[inline]
    fn new(random_string_len: &u8, _rng: &mut impl Rng) -> Self {
        Self {
            random_string_len: *random_string_len,
        }
    }

    #[inline]
    fn generate(&self, rng: &mut impl Rng) -> Self::Event {
        match rng.gen_range(0.0..Self::MAX_WEIGHT) {
            ..=Self::CLEAR => Action::Clear,
            ..=Self::SHRINK_TO_FIT => Action::ShrinkToFit,
            ..=Self::ITER => Action::Iter(IterPeekWeights::new(&(), rng).generate(rng)),
            ..=Self::ITER_MUT => Action::IterMut(IterMutPeekWeights::new(&(), rng).generate(rng)),
            ..=Self::KEYS => Action::Keys(IterPeekWeights::new(&(), rng).generate(rng)),
            ..=Self::VALUES => Action::Values(IterPeekWeights::new(&(), rng).generate(rng)),
            ..=Self::VALUES_MUT => {
                Action::ValuesMut(IterMutPeekWeights::new(&(), rng).generate(rng))
            }
            ..=Self::DRAIN => Action::Drain(IterMutPeekWeights::new(&(), rng).generate(rng)),
            ..=Self::CONTAINS_KEY => {
                let key = generate_random_string(self.random_string_len, rng);
                Action::ContainsKey { key }
            }
            ..=Self::GET => {
                let key = generate_random_string(self.random_string_len, rng);
                Action::Get { key }
            }
            ..=Self::GET_MUT => {
                let key = generate_random_string(self.random_string_len, rng);
                Action::GetMut { key }
            }
            ..=Self::INSERT => {
                let key = generate_random_string(self.random_string_len, rng);
                let value = generate_random_string(self.random_string_len, rng);
                Action::Insert { key, value }
            }
            ..=Self::REMOVE => {
                let key = generate_random_string(self.random_string_len, rng);
                Action::Remove { key }
            }
            ..=Self::ENTRY => {
                let key = generate_random_string(self.random_string_len, rng);
                Action::Entry {
                    key,
                    event: EntryPeekWeights::new(&self.random_string_len, rng).generate(rng),
                }
            }
            ..=Self::INTO_KEYS => Action::IntoKeys(IterMutPeekWeights::new(&(), rng).generate(rng)),
            ..=Self::INTO_VALUES => {
                Action::IntoValues(IterMutPeekWeights::new(&(), rng).generate(rng))
            }
            ..=Self::INTO_ITER => Action::IntoIter(IterMutPeekWeights::new(&(), rng).generate(rng)),
            ..=Self::DEBUG => Action::Debug,
            ..=Self::SERIALIZE => Action::Serialize,
            ..=Self::DROP => Action::Drop,
            value => unreachable!("`{}` isn't in range", value),
        }
    }
}

impl IterPeekWeights {
    const NEXT: f64 = 10.0;
    const LEN: f64 = 10.5;
    const CLONE: f64 = 11.0;
    const DEBUG: f64 = 11.5;
    const DROP: f64 = 12.0;

    const MAX_WEIGHT: f64 = Self::DROP;
}

impl EventWeights for IterPeekWeights {
    type Config = ();
    type Event = Iter;

    #[inline]
    fn new(_config: &Self::Config, _rng: &mut impl Rng) -> Self {
        Self
    }

    #[inline]
    fn generate(&self, rng: &mut impl Rng) -> Self::Event {
        match rng.gen_range(0.0..Self::MAX_WEIGHT) {
            ..=Self::NEXT => Iter::Next,
            ..=Self::LEN => Iter::Len,
            ..=Self::CLONE => Iter::Clone,
            ..=Self::DEBUG => Iter::Debug,
            ..=Self::DROP => Iter::Drop,
            value => unreachable!("`{}` isn't in range", value),
        }
    }
}

impl IterMutPeekWeights {
    const NEXT: f64 = 10.0;
    const LEN: f64 = 10.5;
    const DEBUG: f64 = 11.0;
    const DROP: f64 = 12.5;

    const MAX_WEIGHT: f64 = Self::DROP;
}

impl EventWeights for IterMutPeekWeights {
    type Config = ();
    type Event = IterMut;

    #[inline]
    fn new(_config: &Self::Config, _rng: &mut impl Rng) -> Self {
        Self
    }

    #[inline]
    fn generate(&self, rng: &mut impl Rng) -> Self::Event {
        match rng.gen_range(0.0..Self::MAX_WEIGHT) {
            ..=Self::NEXT => IterMut::Next,
            ..=Self::LEN => IterMut::Len,
            ..=Self::DEBUG => IterMut::Debug,
            ..=Self::DROP => IterMut::Drop,
            value => unreachable!("`{}` isn't in range", value),
        }
    }
}

impl EntryPeekWeights {
    const OR_INSERT: f64 = 1.0;
    const OR_INSERT_WITH: f64 = 1.5;
    const KEY: f64 = 5.0;
    const AND_MODIFY: f64 = 7.0;
    const OR_DEFAULT: f64 = 7.5;
    const ENTRY_MATCH: f64 = 9.0;
    const DROP: f64 = 9.1;

    const MAX_WEIGHT: f64 = Self::DROP;
}

impl EventWeights for EntryPeekWeights {
    type Config = u8;
    type Event = Entry<String>;

    #[inline]
    fn new(random_string_len: &u8, _rng: &mut impl Rng) -> Self {
        Self {
            random_string_len: *random_string_len,
        }
    }

    #[inline]
    fn generate(&self, rng: &mut impl Rng) -> Self::Event {
        match rng.gen_range(0.0..Self::MAX_WEIGHT) {
            ..=Self::OR_INSERT => Entry::OrInsert {
                value: generate_random_string(self.random_string_len, rng),
            },
            ..=Self::OR_INSERT_WITH => {
                let random_string = generate_random_string(self.random_string_len, rng);
                Entry::OrInsertWith {
                    default: Box::new(move || random_string),
                }
            }
            ..=Self::KEY => Entry::Key,
            ..=Self::AND_MODIFY => {
                let random_string = generate_random_string(self.random_string_len, rng);
                Entry::AndModify {
                    f: Box::new(move |v| {
                        let _ = std::mem::replace(v, random_string);
                    }),
                }
            }
            ..=Self::OR_DEFAULT => Entry::OrDefault,
            ..=Self::ENTRY_MATCH => {
                let entry = EntryMatch {
                    on_occupied: OccupiedEntryPeekWeights::new(&self.random_string_len, rng)
                        .generate(rng),
                    on_vacant: VacantEntryPeekWeights::new(&self.random_string_len, rng)
                        .generate(rng),
                };
                Entry::EntryMatch(entry)
            }
            ..=Self::DROP => Entry::Drop,
            value => unreachable!("`{}` isn't in range", value),
        }
    }
}

impl OccupiedEntryPeekWeights {
    const KEY: f64 = 1.0;
    const REMOVE_ENTRY: f64 = 1.5;
    const GET: f64 = 3.0;
    const GET_MUT: f64 = 4.0;
    const INTO_MUT: f64 = 4.5;
    const INSERT: f64 = 5.5;
    const REMOVE: f64 = 6.0;
    const DROP: f64 = 6.1;

    const MAX_WEIGHT: f64 = Self::DROP;
}

impl EventWeights for OccupiedEntryPeekWeights {
    type Config = u8;
    type Event = OccupiedEntry<String>;

    #[inline]
    fn new(random_string_len: &u8, _rng: &mut impl Rng) -> Self {
        Self {
            random_string_len: *random_string_len,
        }
    }

    #[inline]
    fn generate(&self, rng: &mut impl Rng) -> Self::Event {
        match rng.gen_range(0.0..Self::MAX_WEIGHT) {
            ..=Self::KEY => OccupiedEntry::Key,
            ..=Self::REMOVE_ENTRY => OccupiedEntry::RemoveEntry,
            ..=Self::GET => OccupiedEntry::Get,
            ..=Self::GET_MUT => OccupiedEntry::GetMut,
            ..=Self::INTO_MUT => OccupiedEntry::IntoMut,
            ..=Self::INSERT => {
                let value = generate_random_string(self.random_string_len, rng);
                OccupiedEntry::Insert { value }
            }
            ..=Self::REMOVE => OccupiedEntry::Remove,
            ..=Self::DROP => OccupiedEntry::Drop,
            value => unreachable!("`{}` isn't in range", value),
        }
    }
}

impl VacantEntryPeekWeights {
    const KEY: f64 = 0.5;
    const INSERT: f64 = 1.5;
    const DROP: f64 = 1.53;

    const MAX_WEIGHT: f64 = Self::DROP;
}

impl EventWeights for VacantEntryPeekWeights {
    type Config = u8;
    type Event = VacantEntry<String>;

    #[inline]
    fn new(random_string_len: &u8, _rng: &mut impl Rng) -> Self {
        Self {
            random_string_len: *random_string_len,
        }
    }

    #[inline]
    fn generate(&self, rng: &mut impl Rng) -> Self::Event {
        match rng.gen_range(0.0..Self::MAX_WEIGHT) {
            ..=Self::KEY => VacantEntry::Key,
            ..=Self::INSERT => {
                let value = generate_random_string(self.random_string_len, rng);
                VacantEntry::Insert { value }
            }
            ..=Self::DROP => VacantEntry::Drop,
            value => unreachable!("`{}` isn't in range", value),
        }
    }
}
