// From https://github.com/sfackler/rust-phf

use bytemuck::{Pod, Zeroable};
use rand::distributions::Standard;
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};
use siphasher::sip128::{Hash128, Hasher128, SipHasher13};
use std::hash::Hasher;
use std::num::Wrapping;

pub struct Hashes {
    g: u32,
    f1: u32,
    f2: u32,
}

#[derive(Clone, Copy, PartialEq)]
#[repr(C)]
pub struct Disp(u32, u32);

unsafe impl Pod for Disp {}
unsafe impl Zeroable for Disp {}

pub struct HashState {
    pub key: HashKey,
    pub disps: Vec<Disp>,
    pub map: Vec<usize>,
}

/// A central typedef for hash keys
///
/// Makes experimentation easier by only needing to be updated here.
pub type HashKey = u64;

fn displace(f1: u32, f2: u32, d1: u32, d2: u32) -> u32 {
    (Wrapping(d2) + Wrapping(f1) * Wrapping(d1) + Wrapping(f2)).0
}

/// `key` is from `phf_generator::HashState`.
pub fn hash<T: ?Sized + PhfHash>(x: &T, key: HashKey) -> Hashes {
    let mut hasher = SipHasher13::new_with_keys(0, key);
    x.phf_hash(&mut hasher);

    let Hash128 {
        h1: lower,
        h2: upper,
    } = hasher.finish128();

    Hashes {
        g: (lower >> 32) as u32,
        f1: lower as u32,
        f2: upper as u32,
    }
}

/// Return an index into `phf_generator::HashState::map`.
///
/// * `hash` is from `hash()` in this crate.
/// * `disps` is from `phf_generator::HashState::disps`.
/// * `len` is the length of `phf_generator::HashState::map`.
pub fn get_index(hashes: &Hashes, disps: &[Disp], len: usize) -> u32 {
    let Disp(d1, d2) = disps[(hashes.g % (disps.len() as u32)) as usize];
    displace(hashes.f1, hashes.f2, d1, d2) % (len as u32)
}

/// A trait implemented by types which can be used in PHF data structures.
///
/// This differs from the standard library's `Hash` trait in that `PhfHash`'s
/// results must be architecture independent so that hashes will be consistent
/// between the host and target when cross compiling.
pub trait PhfHash {
    /// Feeds the value into the state given, updating the hasher as necessary.
    fn phf_hash<H: Hasher>(&self, state: &mut H);

    /// Feeds a slice of this type into the state provided.
    fn phf_hash_slice<H: Hasher>(data: &[Self], state: &mut H)
    where
        Self: Sized,
    {
        for piece in data {
            piece.phf_hash(state);
        }
    }
}

impl<'a, T: 'a + PhfHash + ?Sized> PhfHash for &'a T {
    fn phf_hash<H: Hasher>(&self, state: &mut H) {
        (*self).phf_hash(state)
    }
}

impl PhfHash for str {
    #[inline]
    fn phf_hash<H: Hasher>(&self, state: &mut H) {
        self.as_bytes().phf_hash(state)
    }
}

impl PhfHash for [u8] {
    #[inline]
    fn phf_hash<H: Hasher>(&self, state: &mut H) {
        state.write(self);
    }
}

const DEFAULT_LAMBDA: usize = 5;

const FIXED_SEED: u64 = 1234567890;

pub fn generate_hash<H: PhfHash>(entries: &[H]) -> HashState {
    SmallRng::seed_from_u64(FIXED_SEED)
        .sample_iter(Standard)
        .find_map(|key| try_generate_hash(entries, key))
        .expect("failed to solve PHF")
}

fn try_generate_hash<H: PhfHash>(entries: &[H], key: HashKey) -> Option<HashState> {
    struct Bucket {
        idx: usize,
        keys: Vec<usize>,
    }

    let hashes: Vec<_> = entries.iter().map(|entry| hash(entry, key)).collect();

    let buckets_len = (hashes.len() + DEFAULT_LAMBDA - 1) / DEFAULT_LAMBDA;
    let mut buckets = (0..buckets_len)
        .map(|i| Bucket {
            idx: i,
            keys: vec![],
        })
        .collect::<Vec<_>>();

    for (i, hash) in hashes.iter().enumerate() {
        buckets[(hash.g % (buckets_len as u32)) as usize]
            .keys
            .push(i);
    }

    // Sort descending
    buckets.sort_by(|a, b| a.keys.len().cmp(&b.keys.len()).reverse());

    let table_len = hashes.len();
    let mut map = vec![None; table_len];
    let mut disps = vec![Disp(0, 0); buckets_len];

    // store whether an element from the bucket being placed is
    // located at a certain position, to allow for efficient overlap
    // checks. It works by storing the generation in each cell and
    // each new placement-attempt is a new generation, so you can tell
    // if this is legitimately full by checking that the generations
    // are equal. (A u64 is far too large to overflow in a reasonable
    // time for current hardware.)
    let mut try_map = vec![0u64; table_len];
    let mut generation = 0u64;

    // the actual values corresponding to the markers above, as
    // (index, key) pairs, for adding to the main map once we've
    // chosen the right disps.
    let mut values_to_add = vec![];

    'buckets: for bucket in &buckets {
        for d1 in 0..(table_len as u32) {
            'disps: for d2 in 0..(table_len as u32) {
                values_to_add.clear();
                generation += 1;

                for &key in &bucket.keys {
                    let idx = (displace(hashes[key].f1, hashes[key].f2, d1, d2)
                        % (table_len as u32)) as usize;
                    if map[idx].is_some() || try_map[idx] == generation {
                        continue 'disps;
                    }
                    try_map[idx] = generation;
                    values_to_add.push((idx, key));
                }

                // We've picked a good set of disps
                disps[bucket.idx] = Disp(d1, d2);
                for &(idx, key) in &values_to_add {
                    map[idx] = Some(key);
                }
                continue 'buckets;
            }
        }

        // Unable to find displacements for a bucket
        return None;
    }

    Some(HashState {
        key,
        disps,
        map: map.into_iter().map(|i| i.unwrap()).collect(),
    })
}
