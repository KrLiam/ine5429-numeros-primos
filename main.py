

from cmath import log
from math import ceil
from time import time
from typing import Callable


class LCG:
    @classmethod
    def from_output_size(cls, bit_size: int, seed: int) -> "LCG":
        s = 16
        return LCG(seed, m=2**(bit_size+s), s=s)
    
    def __init__(
        self,
        seed: int,
        # valores padrões do java.util.Random.
        m: int = 2**48,       # módulo
        a: int = 25214903917, # multiplicador
        c: int = 11,          # incremento
        s: int = 16,          # shift
    ):
        self.m = m
        self.a = a
        self.c = c
        self.s = s
        
        # inicializa sequência
        self.x = seed % m

    def __call__(self) -> int:
        self.x = (self.a*self.x + self.c) % self.m
        # ignorar os s bits menos significantes
        return self.x >> self.s


def test_lcg():
    sizes = [40, 56, 80, 128, 168, 224, 256, 512, 1024, 2048, 4096]
    seed = 3**4096 # escolhe uma seed grande

    for bit_size in sizes:
        lcg = LCG.from_output_size(bit_size, seed)

        elapsed = 0.0
        iterations = 100_000
        for _ in range(iterations):
            t = time()
            lcg()
            elapsed += (time() - t)*1_000_000
        elapsed /= iterations

        print(f"LCG, tamanho={bit_size}, tempo médio de geração={elapsed:.3f}µs")


def mdc(a: int, b: int) -> int:
    """Algoritmo de Euclides."""
    while b:
        a, b = b, a % b
    return a

def choose_primes(nums: list[int]):
    for n in nums:
        s = ceil(log(n, 2).real)
        if n % 4 == 3 and s == 16:
            print(n)


# Números primos que são mod 4 = 3
PRIMES = [
    50599,
    51347,
    46567,
    41183,
    39847,
    60647,
    49787,
    34543,
    48571,
]

class BlumBlumShub:
    def __init__(self, p: int, q: int, seed: int) -> None:
        m = p * q

        assert p % 4 == 3
        assert q % 4 == 3
        assert mdc(seed, m) == 1

        self.m =m
        self.x = seed % self.m
    
    def generate_bit(self) -> int:
        self.x = self.x**2 % self.m
        return self.x % 2
    
    def __call__(self, bit_size: int = 32) -> int:
        n = self.generate_bit()
        for _ in range(bit_size - 1):
            n = (n << 1) | self.generate_bit()
        return n


def test_bbs():
    bbs = BlumBlumShub(PRIMES[0], PRIMES[1], PRIMES[2]*PRIMES[3])
    sizes = [40, 56, 80, 128, 168, 224, 256, 512, 1024, 2048, 4096]

    for bit_size in sizes:
        elapsed = 0.0
        iterations = 5_000
        for _ in range(iterations):
            t = time()
            bbs(bit_size)
            elapsed += (time() - t)*1_000_000
        elapsed /= iterations

        print(f"BBS, tamanho={bit_size}, tempo médio de geração={elapsed:.3f}µs")


def test_frequency():
    ...

def test_distribution(f: Callable[[], int], max_value: int, bin_count: int, samples: int):
    bins: list[int] = [0 for _ in range(bin_count)]
    bin_size = max_value / bin_count

    for _ in range(samples):
        n = f()
        bins[int(n // bin_size)] += 1
    
    expected = samples / bin_count
    x = 0.0
    for observed in bins:
        x += (observed - expected)**2 / expected
    
    # print("Bins:", *bins, sep=" ")
    print(f"Chi-square: {x}")

    
if __name__ == "__main__":
    test_lcg()
    test_bbs()
    test_distribution(LCG.from_output_size(32, 3), 2**32, 2**16, 1_000_000)