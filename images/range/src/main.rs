use risc0_zkvm::{
    guest::{env, sha::Impl, rand::zkvm_getrandom},
    sha::Sha256,
};

fn main() {
    let mut private = [0u8; 4];
    env::read_slice(&mut private);
    let mut public = [0u8; 4];
    env::read_slice(&mut public);
    let rand = [0u8; 8];
    zkvm_getrandom(&mut rand);
    let target = i32::from_le_bytes(private);
    let range = u32::from_le_bytes(public);
    let (a, b) = commit_to_range(target, 100, true);

    let digest = Impl::hash_bytes(&private);
    env::commit_slice(digest.as_bytes());
    env::commit_slice(&a.to_le_bytes());
    env::commit_slice(&b.to_le_bytes());
}

fn commit_to_range(x: i32, accuracy: u32, round_to_power_of_ten: bool, random_value: u64) -> (i32, i32) {
    let random_offset = (random_value % (2 * accuracy as u64 + 1) as u64) as i32 - accuracy as i32;
    
    let lower_bound = x - accuracy as i32 + random_offset;
    let upper_bound = x + accuracy as i32 + random_offset;

    if round_to_power_of_ten {
        let power_of_ten = 10_i32.pow((accuracy as f32).log10().floor() as u32);
        let rounded_lower_bound = (lower_bound / power_of_ten) * power_of_ten;
        let rounded_upper_bound = ((upper_bound + power_of_ten - 1) / power_of_ten) * power_of_ten;
        (rounded_lower_bound, rounded_upper_bound)
    } else {
        (lower_bound, upper_bound)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_commit_to_range_no_rounding() {
        let x = 5;
        let accuracy = 123;
        let random_value = 42;

        let (lower, upper) = commit_to_range(x, accuracy, false, random_value);

        assert!(lower >= -118 && lower <= 128);
        assert!(upper >= -118 && upper <= 128);
        assert_eq!(upper - lower, 246); // The range size should remain the same
    }

    #[test]
    fn test_commit_to_range_with_rounding() {
        let x = 1234;
        let accuracy = 100;
        let random_value = 42;

        let (lower, upper) = commit_to_range(x, accuracy, true, random_value);

        assert!(lower >= 1100 && lower <= 1300);
        assert!(upper >= 1200 && upper <= 1400);
        assert_eq!(upper - lower, 300); // The range size should remain the same after rounding
    }

    #[test]
    fn test_commit_to_range_negative_no_rounding() {
        let x = -100;
        let accuracy = 50;
        let random_value = 42;

        let (lower, upper) = commit_to_range(x, accuracy, false, random_value);

        assert!(lower >= -150 && lower <= -50);
        assert!(upper >= -150 && upper <= -50);
        assert_eq!(upper - lower, 100); // The range size should remain the same
    }

    #[test]
    fn test_commit_to_range_negative_with_rounding() {
        let x = -5678;
        let accuracy = 1000;
        let random_value = 42;

        let (lower, upper) = commit_to_range(x, accuracy, true, random_value);

        assert!(lower >= -7000 && lower <= -5000);
        assert!(upper >= -4000 && upper <= -2000);
        assert_eq!(upper - lower, 3000); // The range size should remain the same after rounding
    }

    #[test]
    fn test_commit_to_range_consistency() {
        let x = 1000;
        let accuracy = 100;
        let random_value = 42;

        let (lower1, upper1) = commit_to_range(x, accuracy, true, random_value);
        let (lower2, upper2) = commit_to_range(x, accuracy, true, random_value);

        assert_eq!(lower1, lower2); // Same random value should produce the same result
        assert_eq!(upper1, upper2);
    }

    #[test]
    fn test_commit_to_range_different_random_values() {
        let x = 1000;
        let accuracy = 100;

        let (lower1, upper1) = commit_to_range(x, accuracy, true, 42);
        let (lower2, upper2) = commit_to_range(x, accuracy, true, 43);

        assert_ne!(lower1, lower2); // Different random values should produce different results
        assert_ne!(upper1, upper2);
    }

    #[test]
    fn test_commit_to_range_bounds() {
        let x = 1000;
        let accuracy = 100;
        let max_random = u64::MAX;

        let (lower, upper) = commit_to_range(x, accuracy, false, max_random);

        assert!(lower >= 900 && lower <= 1100);
        assert!(upper >= 900 && upper <= 1100);
        assert_eq!(upper - lower, 200);
    }
}