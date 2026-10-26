//! Детерминированный генератор случайных чисел (xorshift64*).
//! Мир строится из зерна, поэтому генератор свой: одинаковое зерно —
//! одинаковая сеть на любой машине и в любой версии стандартной библиотеки.
//! Состояние сериализуется вместе с миром, чтобы «шум» после загрузки шёл дальше.

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Rng {
    state: u64,
}

impl Rng {
    /// Новый генератор из зерна. Нулевое состояние у xorshift запрещено —
    /// подмешиваем константу и прогреваем несколько шагов.
    pub fn new(seed: u64) -> Rng {
        let mut s = seed ^ 0x9E37_79B9_7F4A_7C15;
        if s == 0 {
            s = 0x2545_F491_4F6C_DD1D;
        }
        let mut r = Rng { state: s };
        for _ in 0..4 {
            r.next_u64();
        }
        r
    }

    /// Следующее 64-битное число.
    pub fn next_u64(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.state = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    /// Число с плавающей точкой в [0, 1).
    pub fn next_f64(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64
    }

    /// Целое в полуинтервале [lo, hi). Если hi <= lo — возвращает lo.
    pub fn range(&mut self, lo: i64, hi: i64) -> i64 {
        if hi <= lo {
            return lo;
        }
        let span = (hi - lo) as u64;
        lo + (self.next_u64() % span) as i64
    }

    /// Число с плавающей точкой в [lo, hi).
    pub fn range_f(&mut self, lo: f64, hi: f64) -> f64 {
        lo + (hi - lo) * self.next_f64()
    }

    /// Истина с вероятностью p (0..1).
    pub fn chance(&mut self, p: f64) -> bool {
        self.next_f64() < p
    }

    /// Случайный элемент непустого среза.
    pub fn pick<'a, T>(&mut self, items: &'a [T]) -> &'a T {
        assert!(!items.is_empty(), "Rng::pick: пустой список");
        let i = self.range(0, items.len() as i64) as usize;
        &items[i]
    }

    /// Перемешать срез на месте (Фишер — Йетс).
    pub fn shuffle<T>(&mut self, items: &mut [T]) {
        let n = items.len();
        if n < 2 {
            return;
        }
        for i in (1..n).rev() {
            let j = self.range(0, i as i64 + 1) as usize;
            items.swap(i, j);
        }
    }

    /// Значение около базы: base ± spread (равномерно).
    pub fn jitter(&mut self, base: f32, spread: f32) -> f32 {
        base + (self.next_f64() as f32 * 2.0 - 1.0) * spread
    }
}

impl Default for Rng {
    fn default() -> Self {
        Rng::new(1)
    }
}
