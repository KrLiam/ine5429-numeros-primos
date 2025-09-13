use num_bigint::BigUint;
use num_traits::{Zero, One};
use std::ops::{Div};


pub trait RNG {
    fn next(&mut self) -> BigUint;
}


pub struct LCG {
    m: BigUint,
    a: BigUint,
    c: BigUint,
    s: u32,
    x: BigUint,
}

impl LCG {
    pub fn from_output_size(bit_size: u32, seed: BigUint) -> Self {
        let s = 16;
        let m = BigUint::one() << (bit_size + s); // 2^(bit_size+s)
        let a = BigUint::from(25214903917u64);
        let c = BigUint::from(11u64);
        Self::new(seed, m, a, c, s)
    }

    pub fn new(seed: BigUint, m: BigUint, a: BigUint, c: BigUint, s: u32) -> Self {
        let x = seed % &m;
        Self { m, a, c, s, x }
    }
}
impl RNG for LCG {
    fn next(&mut self) -> BigUint {
        self.x = (&self.a * &self.x + &self.c) % &self.m;
        &self.x >> self.s
    }
}


/// Miller-Rabin primality test
/// Returns false if `n` is definitely composite,
/// true if `n` is probably prime.
pub fn miller_rabin<F>(n: &BigUint, rng: &mut F, t: usize) -> bool
where
    F: RNG,
{
    // Handle small cases
    if n < &BigUint::from(2u32) {
        return false;
    }
    if n == &BigUint::from(2u32) || n == &BigUint::from(3u32) {
        return true;
    }
    if n % 2u32 == BigUint::zero() {
        return false;
    }

    // Write n-1 as 2^k * m, with m odd
    let mut m = n - BigUint::one();
    let mut k = 0usize;
    while &m % 2u32 == BigUint::zero() {
        m = m.div(2u32);
        k += 1;
    }

    for _ in 0..t {
        // Pick random base a in [2, n-2]
        let a = rng.next() % (n - BigUint::from(3u32)) + BigUint::from(2u32);

        let mut x = a.modpow(&m, n);
        if x == BigUint::one() || x == n - BigUint::one() {
            continue;
        }

        let mut is_composite = true;
        for _ in 0..(k - 1) {
            x = x.modpow(&BigUint::from(2u32), n);
            if x == n - BigUint::one() {
                is_composite = false;
                break;
            }
        }

        if is_composite {
            return false; // definitely composite
        }
    }

    true // probably prime
}


pub fn generate_prime<F>(rng: &mut F) -> BigUint where F: RNG {
    let mut n = rng.next();
    if &n % BigUint::from(2u32) == BigUint::from(0u32) {
        n += BigUint::from(1u32);
    }
    while !miller_rabin(&n, rng, 20) {
        n += BigUint::from(2u32);
    }
    return n;
}


pub fn main() {
    let seed = BigUint::from(3_u32).pow(4096);
    let mut lcg = LCG::from_output_size(4096, seed);

    // for _ in 0..10_000_000 {
    //     lcg.next();
    // }
    // println!("{}", lcg.x);

    for _ in 0..10 {
        let n = generate_prime(&mut lcg);
        println!("{}", n);
    }
}