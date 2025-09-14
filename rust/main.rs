use num_bigint::BigUint;
use std::{ops::Div, time::Instant};
use num_integer::Integer;
use num_traits::One;
use std::ops::Rem;


/// Macro para escrever BigUint::from(val as u32) mais facilmente
macro_rules! uint {
    ($val:expr) => {
        ::num_bigint::BigUint::from($val as u128)
    };
}

pub trait RNG {
    fn next(&mut self) -> BigUint;
}


// GERAÇÃO DE NÚMERO ALEATÓRIOS //


// LCG

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
        let m = uint!(1) << (bit_size + s); // 2^(bit_size+s)
        let a = uint!(25214903917);
        let c = uint!(11);
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

// Blum Blum Shub

pub struct BlumBlumShub {
    m: BigUint,
    x: BigUint,
    default_size: usize,
}

impl BlumBlumShub {
    /// Create a new BBS generator
    pub fn new(p: &BigUint, q: &BigUint, seed: &BigUint, bit_size: usize) -> Self {
        // m = p * q
        let m = p * q;

        // Check conditions
        assert!(p.rem(&BigUint::from(4u32)) == BigUint::from(3u32), "p % 4 must equal 3");
        assert!(q.rem(&BigUint::from(4u32)) == BigUint::from(3u32), "q % 4 must equal 3");
        assert!(seed.gcd(&m).is_one(), "seed must be coprime with m");

        let x = seed % &m;

        Self { m, x, default_size: bit_size }
    }

    /// Generate one pseudorandom bit
    pub fn generate_bit(&mut self) -> u32 {
        self.x = (&self.x * &self.x) % &self.m;
        self.x.is_odd() as u32
    }

    pub fn set_size(&mut self, bit_size: usize) {
        self.default_size = bit_size;
    }

    /// Generate an integer of `bit_size` bits (optimized)
    pub fn generate(&mut self, bit_size: usize) -> BigUint {
        // Collect bits into a u64 accumulator, flush into Vec<u32> chunks
        let mut digits: Vec<u32> = Vec::new();
        let mut acc: u64 = 0;
        let mut bits_in_acc = 0;

        for _ in 0..bit_size {
            let bit = self.generate_bit() as u64;
            acc = (acc << 1) | bit;
            bits_in_acc += 1;

            if bits_in_acc == 32 {
                digits.push((acc & 0xFFFF_FFFF) as u32);
                acc >>= 32;
                bits_in_acc = 0;
            }
        }

        // Push remaining bits if any
        if bits_in_acc > 0 {
            digits.push(acc as u32);
        }

        BigUint::new(digits)
    }
}
impl RNG for BlumBlumShub {
    fn next(&mut self) -> BigUint {
        self.generate(self.default_size)
    }
}


// TESTE DOS PRNGs //

/// Testa a geração de números por LCG.
fn test_lcg() {
    let sizes = [40, 56, 80, 128, 168, 224, 256, 512, 1024, 2048, 4096];
    let seed = BigUint::from(3u32).pow(4096); 

    let mut last_time = 0.0;

    for &bit_size in &sizes {
        let mut lcg = LCG::from_output_size(bit_size, seed.clone());

        let iterations = 1000_000u32;
        let mut elapsed_total = 0.0;

        for _ in 0..iterations {
            let start = Instant::now();
            let _ = lcg.next(); // generates number
            let duration = start.elapsed();
            elapsed_total += duration.as_secs_f64() * 1_000_000.0;
        }

        
        let avg_time = elapsed_total / iterations as f64;

        let delta_avg = if last_time > 0.0 {
            f64::max(last_time, avg_time) - f64::min(last_time, avg_time)
        }
        else {
            0.0
        };
        last_time = avg_time;

        println!(
            "LCG, {}, {:.3}µs, {}µs",
            bit_size, avg_time, delta_avg
        );
    }
}

/// Testa a geração de números por Blum Blum Shub.
fn test_bbs() {
    let sizes = [40, 56, 80, 128, 168, 224, 256, 512, 1024, 2048, 4096];

    for &bit_size in &sizes {
        let mut lcg = BlumBlumShub::new(
            &uint!(50599), 
            &uint!(51347), 
            &(uint!(46567)*uint!(41183)),
            bit_size
        );

        let iterations = 100_000u32;
        let mut elapsed_total = 0.0;

        for _ in 0..iterations {
            let start = Instant::now();
            let _ = lcg.next(); // generates number
            let duration = start.elapsed();
            elapsed_total += duration.as_secs_f64() * 1_000_000.0;
        }

        let avg_time = elapsed_total / iterations as f64;
        println!(
            "BBS, tamanho={}, tempo médio de geração={:.3}µs",
            bit_size, avg_time
        );
    }
}


// MÉTODOS DE VERIFICAÇÃO DE PRIMALIDADE //

/// Teste de Miller-Rabin.
/// 
/// Retorna `false` se `n`, definitivamente, for composto.
/// Retorna `true` se `n` provavelmente for primo.
pub fn miller_rabin<F>(n: &BigUint, rng: &mut F, t: usize) -> bool
where
    F: RNG,
{
    // Handle small cases
    if n < &uint!(2) {
        return false;
    }
    if n == &uint!(2) || n == &uint!(3) {
        return true;
    }
    if n % 2u32 == uint!(0) {
        return false;
    }

    // Write n-1 as 2^k * m, with m odd
    let mut m = n - uint!(1);
    let mut k = 0;
    while &m % 2u32 == uint!(0) {
        m = m.div(2_u32);
        k += 1;
    }

    for _ in 0..t {
        // Pick random base a in [2, n-2]
        let a = rng.next() % (n - uint!(3)) + uint!(2);

        let mut x = a.modpow(&m, n);
        if x == uint!(1) || x == n - uint!(1) {
            continue;
        }

        let mut is_composite = true;
        for _ in 0..(k - 1) {
            // x = x.modpow(&uint!(2), n);
            x = (&x * &x) % n;
            if x == n - uint!(1) {
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


/// Gera um número primo aleatório utilizando o gerador de números
/// aleatórios fornecido.
pub fn generate_prime<F>(rng: &mut F) -> BigUint where F: RNG {
    let mut n = rng.next();
    if &n % uint!(2) == uint!(0) {
        n += uint!(1);
    }
    while !miller_rabin(&n, rng, 20) {
        n += uint!(2);
    }
    return n;
}


///
pub fn test_primes() {
    let sizes = [40, 56, 80, 128, 168, 224, 256, 512, 1024, 2048, /*4096*/];
    let seed = uint!(3).pow(4000);

    for bit_size in sizes {
        let mut rng = LCG::from_output_size(bit_size, seed.clone());

        for _ in 0..5 {
            let start = Instant::now();
            let p = generate_prime(&mut rng);
            let duration = start.elapsed();
            let time = duration.as_secs_f64();
            println!("| {} | {:.3} | {} |", bit_size, time*1000.0, 0);
        }
    }
}


pub fn main() {
    test_lcg();
    // test_bbs();
    // test_primes();
}