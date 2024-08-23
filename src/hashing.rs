// This code is adjusted from an earlier version I wrote: https://github.com/microbecode/stark-from-zero/blob/master/src/hashing.rs

pub fn hash(input: u128) -> u128 {
    let mut hash: u128 = 3;
    let mut num = input.wrapping_mul(100003); // biggish prime to make all inputs of at least certain size
    while num != 0 {
        let digit = num % 10;
        hash = hash.wrapping_mul(113); // Shift by a prime number so digit positions make a difference
        hash = hash.wrapping_add(digit);
        num /= 10;
    }
    hash
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    #[test]
    fn small_are_not_identical() {
        let mut found_hashes = HashMap::<u128, bool>::new();
        for i in 0..10000 {
            let hash = hash(i);
            assert!(!found_hashes.contains_key(&hash));
            found_hashes.insert(hash, true);
            // println!("hash {} {}", i, hash);
        }
    }

    #[test]
    fn test_simple_hash() {
        assert!(hash(2) > 0);
        assert!(hash(3) > 0);
        assert_eq!(hash(2), hash(2));
        assert_ne!(hash(2), hash(3));

        assert_ne!(hash(234), hash(324));
        assert_ne!(hash(234), hash(432));

        assert_ne!(hash(234), hash(23));
        assert_ne!(hash(234), hash(34));
    }
}
