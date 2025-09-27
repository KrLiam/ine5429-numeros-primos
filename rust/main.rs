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
    /// Módulo
    m: BigUint,
    /// Constante multiplicativa
    a: BigUint,
    /// Incremento
    c: BigUint,
    /// Valor de shift (para descartar os bits menos significativos)
    s: u32,
    /// Valor atual da sequência
    x: BigUint,
}

impl LCG {
    pub fn from_output_size(bit_size: u32, seed: BigUint) -> Self {
        // números gerados internamente tem 16 bits a mais do que a saída
        // para poder descartar os 16 bits menos significativos
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
    /// Gera o próximo numero da sequencia, x_{i+1} = (x_i * a + c) mod m
    fn next(&mut self) -> BigUint {
        self.x = (&self.a * &self.x + &self.c) % &self.m;
        // retorna os n-s bits mais significativos da saída
        &self.x >> self.s
    }
}

// Blum Blum Shub

pub struct BlumBlumShub {
    /// Módulo
    m: BigUint,
    /// Número atual da sequência
    x: BigUint,
    /// Tamanho padrão da saída
    default_size: usize,
}

impl BlumBlumShub {
    /// Create a new BBS generator
    pub fn new(p: &BigUint, q: &BigUint, seed: &BigUint, bit_size: usize) -> Self {
        // m = p * q
        let m = p * q;

        // Checa condições. Segundo Blum Blum Shub (1986), p % 4 == 3, q % 4 == 3 e mdc(m, seed) = 1
        assert!(p.rem(&BigUint::from(4u32)) == BigUint::from(3u32), "p % 4 must equal 3");
        assert!(q.rem(&BigUint::from(4u32)) == BigUint::from(3u32), "q % 4 must equal 3");
        assert!(seed.gcd(&m).is_one(), "seed must be coprime with m");

        let x = seed % &m;

        Self { m, x, default_size: bit_size }
    }

    /// Gera um bit aleatório. Calcula x_{i+1} = x_i^2 mod m e retorna o bit menos significativo de x
    pub fn generate_bit(&mut self) -> u32 {
        self.x = (&self.x * &self.x) % &self.m;
        self.x.is_odd() as u32
    }

    pub fn set_size(&mut self, bit_size: usize) {
        self.default_size = bit_size;
    }

    /// Gera um inteiro de `bit_size` bits. Cada bit é gerado individualmente.
    pub fn generate(&mut self, bit_size: usize) -> BigUint {
        // Basicamente, gera os bits individualmente e constrói uma lista de inteiros de 32 bits
        // que são, posteriormente, unificados em um único BigUint.
        let mut digits: Vec<u32> = Vec::new();
        let mut acc: u64 = 0;
        let mut bits_in_acc = 0;

        for _ in 0..bit_size {
            // Gera um bit e acumula num inteiro
            let bit = self.generate_bit() as u64;
            acc = (acc << 1) | bit;
            bits_in_acc += 1;

            // Adiciona o inteiro com os bits acumulados quando este tiver 32 bits gerados.
            if bits_in_acc == 32 {
                digits.push((acc & 0xFFFF_FFFF) as u32);
                acc >>= 32;
                bits_in_acc = 0;
            }
        }

        if bits_in_acc > 0 {
            digits.push(acc as u32);
        }
        // Gera o número inteiro
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
pub fn test_lcg() {
    // Tamanhos testados
    let sizes = [40, 56, 80, 128, 168, 224, 256, 512, 1024, 2048, 4096];
    let seed = BigUint::from(3u32).pow(4096); 

    let mut last_time = 0.0;

    // Realiza o teste para cada tamanho a ser testado
    for &bit_size in &sizes {
        // Cria um gerador com o tamanho especificado
        let mut lcg = LCG::from_output_size(bit_size, seed.clone());

        let iterations = 1000_000u32;
        let mut elapsed_total = 0.0;

        // Gera múltiplos números pseudo-aleatórios e calcula o tempo médio de geração
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
pub fn test_bbs() {
    // Tamanhos testados
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
    // Casos triviais em que n < 2, n = 2, n = 3 ou n é par.
    if n < &uint!(2) {
        return false;
    }
    if n == &uint!(2) || n == &uint!(3) {
        return true;
    }
    if n % 2u32 == uint!(0) {
        return false;
    }

    // Escreve n-1 como sendo 2^k * m
    let mut m = n - uint!(1);
    let mut k = 0;
    while &m % 2u32 == uint!(0) {
        m = m.div(2_u32);
        k += 1;
    }

    // Realiza t iterações
    for _ in 0..t {
        // Escolhe um valor de a dentro do intervalo [2, n-2]
        let a = rng.next() % (n - uint!(3)) + uint!(2);

        // Calcula a^m mod n
        let mut x = a.modpow(&m, n);
        // n talvez seja primo se x = 1 ou x = n - 1
        if x == uint!(1) || x == n - uint!(1) {
            continue;
        }

        // n talvez seja primo se alguma destas k - 1 condições forem verdadeiras
        // x^2m = -1 mod n, x^4m = -1 mod n, x^8m = -1 mod n, ..., x^(2^k m) = -1 mod n,
        let mut is_composite = true;
        for _ in 0..(k - 1) {
            // calcula x^((2^i)m) mod n
            x = (&x * &x) % n;
            // se x^((2^i)m) = -1 mod n, talvez n seja primo
            if x == n - uint!(1) {
                is_composite = false;
                break;
            }
        }
        // se todas as condições acimas são falsas, n é composto
        if is_composite {
            return false;
        }
    }
    // n é primo
    true
}


/// Calcula o símbolo de Jacobi
fn jacobi(a: BigUint, mut n: BigUint) -> i64 {
    // Lida com casos extremos e garante que n seja ímpar e positivo
    if &n <= &uint!(0) || &n % &uint!(2) == uint!(0) {
        return 0; // Ou um erro, dependendo do comportamento desejado
    }
    if &n == &uint!(1) {
        return 1;
    }

    let mut a = a % &n;
    if &a < &uint!(0) {
        a += &n; // Garante que a seja não negativo
    }

    let mut result = 1;

    while &a != &uint!(0) {
        // Propriedade 1: (a/n) = (a mod n / n)
        // Isso é tratado pelo 'a = a % n' inicial;

        // Propriedade 2: (2/n)
        while &a % &uint!(2) == uint!(0) {
            a /= uint!(2);
            let n_mod_8 = &n % &uint!(8);
            if n_mod_8 == uint!(3) || n_mod_8 == uint!(5) {
                result = -result;
            }
        }
        // Propriedade 3: Reciprocidade Quadrática (a/n) = (n/a) * (-1)^((a-1)/2 * (n-1)/2)
        // Troca a e n
        std::mem::swap(&mut a, &mut n);
        if &a % &uint!(4) == uint!(3) && &n % &uint!(4) == uint!(3) {
            result = -result;
        }

        // Reduz a módulo n após a troca
        a %= &n;
    }

    if n == uint!(1) {
        result
    }
    else {
        0 // Se a se torna 0 antes de n se tornar 1, o símbolo é 0 (a menos que n=1 inicialmente)
    }
}


pub fn solovay_strassen<F>(n: &BigUint, rng: &mut F, t: usize) -> bool where F: RNG {
    // Lida com casos triviais, n < 2, n = 2, n = 3 ou n é par
    if *n < BigUint::from(2u32) {
        return false;
    }
    if *n == BigUint::from(2u32) || *n == BigUint::from(3u32) {
        return true;
    }
    if n.is_even() {
        return false;
    }

    let exp = (n - uint!(1)) >> 1; // (n-1)/2

    for _ in 0..t {
        // Escolhe um 'a' aleatório em [2, n-2]
        let a = loop {
            let a = rng.next() % (n - uint!(1));
            if a >= uint!(2) { break a; }
        };
        // Calcula gcd(a, n), se >1 então n é composto
        if a.gcd(n) != uint!(1) {
            return false;
        }
        // Calcula x = a^((n-1)/2) mod n
        let x = a.modpow(&exp, n);
        // Calcula (a/n), mas mapeia o resultado {-1,0,1} para (a/n) mod n
        let j = match jacobi(a.clone(), n.clone()) {
            -1 => n - uint!(1), // -1 mod n
             0 => return false, // gcd != 1, n composto
             1 => uint!(1),
             _ => unreachable!(),
        };
        // Se x != j mod n, n é composto
        if x != j {
            return false;
        }
    }
    true // provavelmente primo
}

pub fn test_solovay_strassen() {
    let mut rng = LCG::from_output_size(64, uint!(3).pow(64));

    for i in 2..500_000 {
        let i = 2*i + 1;
        let mr = miller_rabin(&uint!(i), &mut rng, 20);
        let ss = solovay_strassen(&uint!(i), &mut rng, 20);
        if ss != mr {
            println!("{} {} {}", i, ss, mr);
        }
    }
}


/// Gera um número primo aleatório utilizando o gerador de números
/// aleatórios fornecido.
pub fn generate_prime<F, P>(rng: &mut F, test: P) -> BigUint
where
    F: RNG,
    P: Fn(&BigUint, &mut F, usize) -> bool
{
    let mut n = rng.next();
    if &n % uint!(2) == uint!(0) {
        n += uint!(1);
    }
    while !test(&n, rng, 20) {
        n += uint!(2);
    }
    return n;
}


/// Função de teste para gerar tabela de números primos de tamanhos variados
pub fn test_primes() {
    let sizes = [40, 56, 80, 128, 168, 224, 256, 512, 1024, 2048, 4096];
    let seed = uint!(3).pow(32000);

    for bit_size in sizes {
        let mut rng = LCG::from_output_size(bit_size, seed.clone());

        for _ in 0..5 {
            let start = Instant::now();
            let _p = generate_prime(&mut rng, miller_rabin);
            let duration = start.elapsed();
            let time = duration.as_secs_f64();
            println!("| Miller-Rabin | {} | {:.3} | {} |", bit_size, time*1000.0, _p);
        }
        for _ in 0..5 {
            let start = Instant::now();
            let _p = generate_prime(&mut rng, solovay_strassen);
            let duration = start.elapsed();
            let time = duration.as_secs_f64();
            println!("| Solovay-Strassen | {} | {:.3} | {} |", bit_size, time*1000.0, _p);
        }
    }
}

/// Verifica se n é primo utilizando o método de divisão por tentativa.
/// Utilizado como base para testar acurácia dos testes de primalidade probabilísticos.
pub fn is_prime_trial(n: &BigUint) -> bool {
    if *n < BigUint::from(2u32) {
        return false;
    }
    if *n == BigUint::from(2u32) || *n == BigUint::from(3u32) {
        return true;
    }
    if n.is_even() {
        return false;
    }

    let mut i = BigUint::from(3u32);
    let limit = n.sqrt();
    while &i <= &limit {
        if n % &i == uint!(0) {
            return false;
        }
        i += 2u32;
    }

    true
}

/// Verifica os resultados obtidos pelos testes de primalidade para os primeiros
/// 1_000_000 de inteiros positivos. Irá imprimir todas as convergências.
pub fn test_prime_testers() {
    let mut rng = LCG::from_output_size(32, uint!(3_i32.pow(20)));
    let mut mh_sum = 0;
    let mut ss_sum= 0;
    let mut both_sum = 0;
    for i in 1..1_000_000 {
        let base = is_prime_trial(&uint!(i));
        let mh = miller_rabin(&uint!(i), &mut rng, 10);
        let ss = solovay_strassen(&uint!(i), &mut rng, 10);
        if base != mh {
            mh_sum += 1;
        }
        if base != ss {
            ss_sum += 1;
        }
        if base != ss && base != mh {
            both_sum += 1;
        }
        if base != ss || base != mh {
            println!("{}, BASE={} MH={}, SH={}", i, base as u32, mh as u32, ss as u32);
        }
    }
    println!("{} {} {}", mh_sum, ss_sum, both_sum);
}

pub fn main() {
    test_lcg();
    test_bbs();
    test_primes();
    test_prime_testers();
}