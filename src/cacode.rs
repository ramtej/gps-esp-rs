use std::collections::HashMap;

#[derive(Debug)]
pub struct SV {
    pub navstar: u8,
    pub t1: u8,
    pub t2: u8,
}

type Prn = u8;
#[derive(Default)]
pub struct SVs {
    pub svs: HashMap<Prn, SV>,
}

macro_rules! hashmap {
    ($( $key: expr => $val: expr ),*) => {{
         let mut map = ::std::collections::HashMap::new();
         $( map.insert($key, $val); )*
         map
    }}
}

/**
     1,  63,  2,  6,
     2,  56,  3,  7,
     3,  37,  4,  8,
     4,  35,  5,  9,
     5,  64,  1,  9,
     6,  36,  2, 10,
     7,  62,  1,  8,
     8,  44,  2,  9,
     9,  33,  3, 10,
    10,  38,  2,  3,
    11,  46,  3,  4,
    12,  59,  5,  6,
    13,  43,  6,  7,
    14,  49,  7,  8,
    15,  60,  8,  9,
    16,  51,  9, 10,
    17,  57,  1,  4,
    18,  50,  2,  5,
    19,  54,  3,  6,
    20,  47,  4,  7,
    21,  52,  5,  8,
    22,  53,  6,  9,
    23,  55,  1,  3,
    24,  23,  4,  6,
    25,  24,  5,  7,
    26,  26,  6,  8,
    27,  27,  7,  9,
    28,  48,  8, 10,
    29,  61,  1,  6,
    30,  39,  2,  7,
    31,  58,  3,  8,
    32,  22,  4,  9,
*/

impl SVs {
    pub fn new() -> Self {
        let svs = hashmap![
        1 => SV {navstar: 63, t1: 2, t2: 6},
        2 => SV {navstar: 56, t1: 3, t2: 7},
        3 => SV {navstar: 37, t1: 4, t2: 8},
        4 => SV {navstar: 35, t1: 5, t2: 9},
        5 => SV {navstar: 64, t1: 1, t2: 9},
        6 => SV {navstar: 36, t1: 2, t2: 10},
        7 => SV {navstar: 62, t1: 1, t2: 8},
        8 => SV {navstar: 44, t1: 2, t2: 9},
        9 => SV {navstar: 33, t1: 3, t2: 10},
        10 => SV {navstar: 38, t1: 2, t2: 3},
        11 => SV {navstar: 46, t1: 3, t2: 4},
        12 => SV {navstar: 59, t1: 5, t2: 6},
        13 => SV {navstar: 43, t1: 6, t2: 7},
        14 => SV {navstar: 49, t1: 7, t2: 8},
        15 => SV {navstar: 60, t1: 8, t2: 9},
        16 => SV {navstar: 51, t1: 9, t2: 10},
        17 => SV {navstar: 57, t1: 1, t2: 4},
        18 => SV {navstar: 50, t1: 2, t2: 5},
        19 => SV {navstar: 54, t1: 3, t2: 6},
        20 => SV {navstar: 47, t1: 4, t2: 7},
        21 => SV {navstar: 52, t1: 5, t2: 8},
        22 => SV {navstar: 53, t1: 6, t2: 9},
        23 => SV {navstar: 55, t1: 1, t2: 3},
        24 => SV {navstar: 23, t1: 4, t2: 6},
        25 => SV {navstar: 24, t1: 5, t2: 7},
        26 => SV {navstar: 26, t1: 6, t2: 8},
        27 => SV {navstar: 27, t1: 7, t2: 9},
        28 => SV {navstar: 48, t1: 8, t2: 10},
        29 => SV {navstar: 61, t1: 1, t2: 6},
        30 => SV {navstar: 39, t1: 2, t2: 7},
        31 => SV {navstar: 58, t1: 3, t2: 8},
        32 => SV {navstar: 22, t1: 4, t2: 9}
                    ];

        Self { svs }
    }
}

pub struct CACode {
    g1: [bool; 11],
    g2: [bool; 11],
    tap: [usize; 2],
}

impl CACode {
    pub fn new(t0: usize, t1: usize) -> Self {
        let mut new_ca_code = CACode {
            g1: [false; 11],
            g2: [false; 11],
            tap: [t0, t1],
        };

        for i in 1..11 {
            new_ca_code.g1[i] = true;
            new_ca_code.g2[i] = true;
        }

        new_ca_code
    }

    pub fn chip(&self) -> bool {
        self.g1[10] ^ self.g2[self.tap[0]] ^ self.g2[self.tap[1]]
    }

    pub fn clock(&mut self) {
        self.g1[0] = self.g1[3] ^ self.g1[10];
        self.g2[0] = self.g2[2] ^ self.g2[3] ^ self.g2[6] ^ self.g2[8] ^ self.g2[9] ^ self.g2[10];

        self.g1.rotate_right(1);
        self.g2.rotate_right(1);
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_cacode() {}
}
