use risc0_zkvm::{
    guest::{env, sha::Impl},
    sha::Sha256,
};

fn main() {
    let mut private = [0u8; 4];
    env::read_slice(&mut private);
    let target = i32::from_le_bytes(private);
    let (a, b) = commit_to_range(target, 100, true);

    let digest = Impl::hash_bytes(&private);
    env::commit_slice(digest.as_bytes());
    env::commit_slice(&a.to_le_bytes());
    env::commit_slice(&b.to_le_bytes());
}

fn commit_to_range(x: i32, accuracy: u32, round_to_power_of_ten: bool) -> (i32, i32) {
    let lower_bound = x - accuracy as i32;
    let upper_bound = x + accuracy as i32;

    if round_to_power_of_ten {
        let power_of_ten = 10_i32.pow((accuracy as f32).log10().floor() as u32);
        let rounded_lower_bound = (lower_bound / power_of_ten) * power_of_ten;
        let rounded_upper_bound = ((upper_bound + power_of_ten - 1) / power_of_ten) * power_of_ten;
        return (rounded_lower_bound, rounded_upper_bound);
    }
    (lower_bound, upper_bound)
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_commit_to_range_no_rounding() {
        let x = 5;
        let accuracy = 123;

        let (lower, upper) = commit_to_range(x, accuracy, false);

        assert_eq!(lower, -118);
        assert_eq!(upper, 128);
    }

    #[test]
    fn test_commit_to_range_with_rounding() {
        let x = 1234;
        let accuracy = 100;

        let (lower, upper) = commit_to_range(x, accuracy, true);

        assert_eq!(lower, 1100);
        assert_eq!(upper, 1400);
    }

    #[test]
    fn test_commit_to_range_negative_no_rounding() {
        let x = -100;
        let accuracy = 50;

        let (lower, upper) = commit_to_range(x, accuracy, false);

        assert_eq!(lower, -150);
        assert_eq!(upper, -50);
    }

    #[test]
    fn test_commit_to_range_negative_with_rounding() {
        let x = -5678;
        let accuracy = 1000;

        let (lower, upper) = commit_to_range(x, accuracy, true);

        assert_eq!(lower, -6000);
        assert_eq!(upper, -3000);
    }

    #[test]
    fn test_commit_to_range_negative_with_rounding_2() {
        let x = -5678;
        let accuracy = 10;

        let (lower, upper) = commit_to_range(x, accuracy, true);

        assert_eq!(lower, -5680);
        assert_eq!(upper, -5690);
    }
}
