pub fn splitmix64(mut x: u64) -> u64 {
    x = x.wrapping_add(0x9e37_79b9_7f4a_7c15);
    let mut z = x;
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    z ^ (z >> 31)
}

pub fn seeded_unit_f32(seed: u64, index: u64) -> f32 {
    let bits = splitmix64(seed ^ index);
    ((bits >> 40) as f32) / ((1_u64 << 24) as f32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deterministic_rng_is_stable() {
        assert_eq!(seeded_unit_f32(781278, 42), seeded_unit_f32(781278, 42));
        assert_ne!(seeded_unit_f32(781278, 42), seeded_unit_f32(781278, 43));
    }
}
