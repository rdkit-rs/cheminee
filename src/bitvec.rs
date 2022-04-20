#[cfg(test)]
mod tests {
    use bitvec::prelude::*;

    #[test]
    fn test_bitvec() {
        let fingerprint_one: [u8; 2] = [0b00000001, 0b10101010];
        let fingerprint_two: [u8; 2] = [0b11111111, 0b11110000];

        let fingerprint_one: &BitSlice<_, Lsb0> =
            bitvec::slice::BitSlice::from_slice(&fingerprint_one);
        let fingerprint_one: BitVec<u8> = fingerprint_one.to_bitvec();

        let fingerprint_two: &BitSlice<_, Lsb0> =
            bitvec::slice::BitSlice::from_slice(&fingerprint_two);
        let fingerprint_two: BitVec<u8> = fingerprint_two.to_bitvec();

        let equal = fingerprint_one == fingerprint_two;

        let and = fingerprint_one & fingerprint_two;

        panic!("{:?}", and);
    }
}
