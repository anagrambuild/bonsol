use num_bigint::BigUint;
use risc0_zkvm::{
    guest::{env, sha::Impl},
    sha::{Digest, Sha256},
};

fn main() {
    let mut pk = Vec::new();
    env::read_slice(&mut pk);
    let mut num = Vec::new();
    env::read_slice(&mut num);
    let digest = Impl::hash_bytes(&[pk.as_slice(), num.as_slice()].concat());
    let (sequence, sum, max) = calculate_sequence(&num);
    let sequence_length = sequence.len() as u64;
    let difficulty = calculate_difficulty(sequence_length, max, sum);
    env::commit_slice(digest.as_bytes());
    env::commit_slice(&[difficulty.to_le_bytes()]);
}

fn calculate_sequence(num: &[u8]) -> (Vec<BigUint>, BigUint, BigUint) {
    //sequence, sum, max
    let bignum = BigUint::from_bytes_le(num);
    let mut current = bignum.clone();
    let mut sum = BigUint::from(0u32);
    let mut max = current.clone();
    let two = BigUint::from(2u32);
    let three = BigUint::from(3u32);
    let one = BigUint::from(1u32);
    let mut sequence = Vec::new();
    let zero = BigUint::from(0u32);
    while current != one {
        if current == zero {
            break;
        }
        sequence.push(current.clone());
        sum += &current;

        if current > max {
            max = current.clone();
        }

        if &current % &two == zero {
            current /= &two;
        } else {
            current = &current * &three + one.clone();
        }
        
    }

    sequence.push(one.clone());
    sum += one.clone();
    (sequence, sum, max)
}

fn calculate_difficulty(len: u64, max: BigUint, sum: BigUint) -> u64 {
    let log2_sum = sum.bits() - 1;
    let log2_max = max.bits() - 1;
    if log2_max == 0 {
        return 0;
    }
    log2_sum * len / log2_max
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_sequence_27() {
        let input = [27u8];
        let (sequence, sum, max) = calculate_sequence(&input);

        // Check sequence length
        assert_eq!(sequence.len(), 112);

        // Check first few and last few elements
        assert_eq!(sequence[0], BigUint::from(27u32));
        assert_eq!(sequence[1], BigUint::from(82u32));
        assert_eq!(sequence[2], BigUint::from(41u32));
        assert_eq!(sequence[10], BigUint::from(214u32));
        assert_eq!(sequence[sequence.len() - 2], BigUint::from(2u32));
        assert_eq!(sequence[sequence.len() - 1], BigUint::from(1u32));

        // Check maximum value
        assert_eq!(max, BigUint::from(9232u32));

        let expected_sum = BigUint::from(101440u32);
        assert_eq!(
           sum, expected_sum
        );

        // Calculate and check difficulty
        let difficulty = calculate_difficulty(sequence.len() as u64, max, sum);
        assert!(difficulty > 0); // The exact value may vary depending on your implementation
    }

    #[test]
    fn test_calculate_sequence_simple() {
        let input = [5u8];
        let (sequence, sum, max) = calculate_sequence(&input);
        assert_eq!(
            sequence,
            vec![
                BigUint::from(5u32),
                BigUint::from(16u32),
                BigUint::from(8u32),
                BigUint::from(4u32),
                BigUint::from(2u32),
                BigUint::from(1u32)
            ]
        );
        assert_eq!(sum, BigUint::from(36u32));
        assert_eq!(max, BigUint::from(16u32));
    }

    #[test]
    fn test_calculate_sequence_power_of_two() {
        let input = [8u8];
        let (sequence, sum, max) = calculate_sequence(&input);
        assert_eq!(
            sequence,
            vec![
                BigUint::from(8u32),
                BigUint::from(4u32),
                BigUint::from(2u32),
                BigUint::from(1u32)
            ]
        );
        assert_eq!(sum, BigUint::from(15u32));
        assert_eq!(max, BigUint::from(8u32));
    }

    #[test]
    fn test_calculate_sequence_large_number() {
        let input = [255u8, 255u8]; // 65535 in little-endian
        let (sequence, sum, max) = calculate_sequence(&input);
        assert!(sequence.len() > 100); // Expect a long sequence
        assert!(sum > BigUint::from(1_000_000u32)); // Expect a large sum
        assert!(max > BigUint::from(100_000u32)); // Expect a large max value
    }

    #[test]
    fn test_calculate_sequence_one() {
        let input = [1u8];
        let (sequence, sum, max) = calculate_sequence(&input);
        assert_eq!(sequence, vec![BigUint::from(1u32)]);
        assert_eq!(sum, BigUint::from(1u32));
        assert_eq!(max, BigUint::from(1u32));
    }

    #[test]
    fn test_calculate_difficulty_simple() {
        let len = 6;
        let max = BigUint::from(16u32);
        let sum = BigUint::from(36u32);
        let difficulty = calculate_difficulty(len, max, sum);
        assert_eq!(difficulty, 7); 
    }

    #[test]
    fn test_calculate_difficulty_large_numbers() {
        let len = 1000;
        let max = BigUint::from(1000000u32);
        let sum = BigUint::from(1000000000u32);
        let difficulty = calculate_difficulty(len, max, sum);
        assert!(difficulty > 1000);
    }

    #[test]
    fn test_calculate_difficulty_edge_case() {
        let len = 1;
        let max = BigUint::from(1u32);
        let sum = BigUint::from(1u32);
        let difficulty = calculate_difficulty(len, max, sum);
        assert_eq!(difficulty, 0);
    }

    #[test]
    fn test_end_to_end() {
        let input = [27u8];
        let (sequence, sum, max) = calculate_sequence(&input);
        let difficulty = calculate_difficulty(sequence.len() as u64, max, sum);
        assert!(difficulty > 0);
    }
}
