use crate::error::Error;
use crate::input::*;
use crate::output::Output;
use crate::sketch::Flag;

const SIGNIFICAND_WIDTH: i64 = 53;
const SIGNIFICAND_MASK: i64 = 0x000fffffffffffff;
const EXPONENT_MASK: i64 = 0x7FF0000000000000;
const EXPONENT_SHIFT: i64 = SIGNIFICAND_WIDTH - 1;
const EXPONENT_BIAS: i64 = 1023;
const ONE: i64 = 0x3ff0000000000000;
const VAR_DOUBLE_ROTATE_DISTANCE: u32 = 6;
const UNSIGNED_VAR_LONG_LENGTHS: [i64; 65] = [
    9, 9, 9, 9, 9, 9, 9, 9, 8, 8, 8, 8, 8, 8, 8, 7, 7, 7, 7, 7, 7, 7, 6, 6, 6, 6, 6, 6, 6, 5, 5, 5,
    5, 5, 5, 5, 4, 4, 4, 4, 4, 4, 4, 3, 3, 3, 3, 3, 3, 3, 2, 2, 2, 2, 2, 2, 2, 1, 1, 1, 1, 1, 1, 1,
    1,
];
const VAR_DOUBLE_LENGTHS: [i64; 65] = [
    9, 9, 9, 9, 9, 9, 9, 9, 8, 8, 8, 8, 8, 8, 8, 7, 7, 7, 7, 7, 7, 7, 6, 6, 6, 6, 6, 6, 6, 5, 5, 5,
    5, 5, 5, 5, 4, 4, 4, 4, 4, 4, 4, 3, 3, 3, 3, 3, 3, 3, 2, 2, 2, 2, 2, 2, 2, 1, 1, 1, 1, 1, 1, 1,
    1,
];

pub fn decode_signed_var_long(input: &mut Input) -> Result<i64, Error> {
    Ok(zig_zag_decode(decode_unsigned_var_long(input)?))
}

pub fn decode_unsigned_var_long(input: &mut Input) -> Result<i64, Error> {
    let mut value: i64 = 0;
    let mut shift = 0;
    loop {
        let next = input.read_byte()? as i8;
        if next >= 0 || shift == 56 {
            return Ok(value | (next as i64) << shift);
        }
        value |= (next as i64 & 127) << shift;
        shift += 7;
    }
}

pub fn decode_var_double(input: &mut Input) -> Result<f64, Error> {
    let mut bits: i64 = 0;
    let mut shift = 8 * 8 - 7;
    loop {
        let next = input.read_byte()? as i8;
        if shift == 1 {
            bits |= (next as u8) as i64;
            break;
        }
        if next >= 0 {
            bits |= (next as i64) << shift;
            break;
        }
        bits |= ((next as i64) & 127) << shift;
        shift -= 7;
    }
    Ok(var_bits_to_double(bits))
}

pub fn i64_to_i32_exact(value: i64) -> Result<i32, Error> {
    let v = value as i32;
    if value != v as i64 {
        return Err(Error::InvalidArgument("Value is not valid i32."));
    }
    Ok(v)
}

pub fn i32_to_usize_exact(value: i32) -> Result<usize, Error> {
    if value < 0 {
        return Err(Error::InvalidArgument("Value should be grate than 0."));
    }
    Ok(value as usize)
}

pub fn get_exponent(long_bits: i64) -> i64 {
    ((long_bits & EXPONENT_MASK) >> EXPONENT_SHIFT) - EXPONENT_BIAS
}

pub fn get_significand_plus_one(long_bits: i64) -> f64 {
    let raw = (long_bits & SIGNIFICAND_MASK) | ONE;
    f64::from_bits(raw as u64)
}

pub fn build_double(exponent: i64, significand_plus_one: f64) -> f64 {
    let significand_plus_one = 1.0_f64.max(significand_plus_one);
    let raw = (((exponent + EXPONENT_BIAS) << EXPONENT_SHIFT) & EXPONENT_MASK)
        | (f64::to_bits(significand_plus_one) as i64 & SIGNIFICAND_MASK);
    f64::from_bits(raw as u64)
}

fn zig_zag_decode(value: i64) -> i64 {
    ((value as u64) >> 1) as i64 ^ (-(value & 1))
}

fn var_bits_to_double(bits: i64) -> f64 {
    f64::from_bits((i64::rotate_right(bits, 6) + f64::to_bits(1.0) as i64) as u64) - 1.0
}

pub fn ignore_exact_summary_statistic_flags(input: &mut Input, flag: Flag) -> Result<(), Error> {
    if flag == Flag::COUNT {
        decode_var_double(input)?;
        Ok(())
    } else if flag == Flag::SUM || flag == Flag::MIN || flag == Flag::MAX {
        input.read_double_le()?;
        Ok(())
    } else {
        Err(Error::InvalidArgument("Unknown Flag."))
    }
}

pub fn encode_var_double(output: &mut Output, value: f64) -> Result<(), Error> {
    let mut bits = double_to_var_bits(value);
    for _ in 0..8 {
        let next = (bits >> (8 * 8 - 7)) as u8;
        bits <<= 7;
        if bits == 0 {
            output.write_byte(next)?;
            return Ok(());
        }
        output.write_byte(next | 0x80)?;
    }
    output.write_byte((bits >> (8 * 7)) as u8)?;
    Ok(())
}

fn double_to_var_bits(value: f64) -> u64 {
    i64::rotate_left(
        f64::to_bits(value + 1.0) as i64 - f64::to_bits(1.0) as i64,
        VAR_DOUBLE_ROTATE_DISTANCE,
    ) as u64
}

pub fn unsigned_var_long_encoded_length(value: i64) -> i64 {
    UNSIGNED_VAR_LONG_LENGTHS[value.leading_zeros() as usize]
}

pub fn signed_var_long_encoded_length(value: i64) -> i64 {
    UNSIGNED_VAR_LONG_LENGTHS[i64::leading_zeros(zig_zag_encode(value)) as usize]
}

pub fn var_double_encoded_length(value: f64) -> i64 {
    VAR_DOUBLE_LENGTHS[i64::trailing_zeros(double_to_var_bits(value) as i64) as usize]
}

fn zig_zag_encode(value: i64) -> i64 {
    value >> (64 - 1) ^ (value << 1)
}

pub fn encode_unsigned_var_long(output: &mut Output, mut value: i64) -> Result<(), Error> {
    let length = (63_i64 - value.leading_zeros() as i64) / 7;
    let mut i = 0;
    while i < length && i < 8 {
        output.write_byte((value | 0x80) as u8)?;
        value >>= 7;
        i += 1;
    }
    output.write_byte(value as u8)?;
    Ok(())
}

pub fn encode_signed_var_long(output: &mut Output, value: i64) -> Result<(), Error> {
    encode_unsigned_var_long(output, zig_zag_encode(value))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::Input;
    use crate::output::Output;

    #[test]
    fn test_decode_var_double() {
        let args: [(f64, Vec<u8>); 16] = [
            (0.0, vec![0]),
            (1.0, vec![2]),
            (2.0, vec![3]),
            (3.0, vec![4]),
            (4.0, vec![132, 64]),
            (5.0, vec![5]),
            (6.0, vec![133, 64]),
            (7.0, vec![6]),
            (8.0, vec![134, 32]),
            (9.0, vec![134, 64]),
            (
                4.503599627370494E15,
                vec![231, 255, 255, 255, 255, 255, 255, 255, 128],
            ),
            (4.503599627370495E15, vec![104]),
            (
                4.503599627370496E15,
                vec![232, 128, 128, 128, 128, 128, 128, 128, 64],
            ),
            (
                9.00719925474099E15,
                vec![233, 255, 255, 255, 255, 255, 255, 255, 192],
            ),
            (-1.0, vec![130, 128, 128, 128, 128, 128, 128, 128, 48]),
            (-0.5, vec![254, 128, 128, 128, 128, 128, 128, 128, 63]),
        ];

        for arg in args {
            let mut input = Input::wrap(&arg.1);
            let value: f64 = decode_var_double(&mut input).unwrap();
            assert_eq!(value, arg.0);
        }
    }

    #[test]
    fn test_decode_signed_var_long() {
        let args: [(i64, Vec<u8>); 29] = [
            (0, vec![0]),
            (1, vec![2]),
            (63, vec![126]),
            (64, vec![128, 1]),
            (65, vec![130, 1]),
            (127, vec![254, 1]),
            (128, vec![128, 2]),
            (8191, vec![254, 127]),
            (8192, vec![128, 128, 1]),
            (8193, vec![130, 128, 1]),
            (
                4611686018427387902,
                vec![252, 255, 255, 255, 255, 255, 255, 255, 127],
            ),
            (
                4611686018427387903,
                vec![254, 255, 255, 255, 255, 255, 255, 255, 127],
            ),
            (
                4611686018427387904,
                vec![128, 128, 128, 128, 128, 128, 128, 128, 128],
            ),
            (
                9223372036854775806,
                vec![252, 255, 255, 255, 255, 255, 255, 255, 255],
            ),
            (
                9223372036854775807,
                vec![254, 255, 255, 255, 255, 255, 255, 255, 255],
            ),
            (-1, vec![1]),
            (-63, vec![125]),
            (-64, vec![127]),
            (-65, vec![129, 1]),
            (-127, vec![253, 1]),
            (-128, vec![255, 1]),
            (-8191, vec![253, 127]),
            (-8192, vec![255, 127]),
            (-8193, vec![129, 128, 1]),
            (
                -4611686018427387903,
                vec![253, 255, 255, 255, 255, 255, 255, 255, 127],
            ),
            (
                -4611686018427387904,
                vec![255, 255, 255, 255, 255, 255, 255, 255, 127],
            ),
            (
                -4611686018427387905,
                vec![129, 128, 128, 128, 128, 128, 128, 128, 128],
            ),
            (
                -9223372036854775807,
                vec![253, 255, 255, 255, 255, 255, 255, 255, 255],
            ),
            (
                -9223372036854775808,
                vec![255, 255, 255, 255, 255, 255, 255, 255, 255],
            ),
        ];

        for arg in args {
            let mut input = Input::wrap(&arg.1);
            let value: i64 = decode_signed_var_long(&mut input).unwrap();
            assert_eq!(arg.0, value);
        }
    }

    #[test]
    fn test_decode_unsigned_var_long() {
        let args: [(i64, Vec<u8>); 12] = [
            (0, vec![0]),
            (1, vec![1]),
            (127, vec![127]),
            (128, vec![128, 1]),
            (129, vec![129, 1]),
            (255, vec![255, 1]),
            (256, vec![128, 2]),
            (16383, vec![255, 127]),
            (16384, vec![128, 128, 1]),
            (16385, vec![129, 128, 1]),
            (-2, vec![254, 255, 255, 255, 255, 255, 255, 255, 255]),
            (-1, vec![255, 255, 255, 255, 255, 255, 255, 255, 255]),
        ];

        for arg in args {
            let mut input = Input::wrap(&arg.1);
            let value: i64 = decode_unsigned_var_long(&mut input).unwrap();
            assert_eq!(arg.0, value);
        }
    }

    #[test]
    #[should_panic]
    fn test_i64_to_i32_exact_with_panic_1() {
        i64_to_i32_exact(2147483648).unwrap();
    }

    #[test]
    #[should_panic]
    fn test_i64_to_i32_exact_with_panic_2() {
        i64_to_i32_exact(-2147483649).unwrap();
    }

    #[test]
    fn test_i64_to_i32_exact() {
        assert_eq!(i64_to_i32_exact(0).unwrap(), 0);
        assert_eq!(i64_to_i32_exact(1).unwrap(), 1);
        assert_eq!(i64_to_i32_exact(-1).unwrap(), -1);
        assert_eq!(i64_to_i32_exact(65535).unwrap(), 65535);
        assert_eq!(i64_to_i32_exact(-65535).unwrap(), -65535);
        assert_eq!(i64_to_i32_exact(2147483647).unwrap(), 2147483647);
        assert_eq!(i64_to_i32_exact(-2147483648).unwrap(), -2147483648);
    }

    #[test]
    fn test_i32_to_usize_exact() {
        assert_eq!(i32_to_usize_exact(0).unwrap(), 0);
        assert_eq!(i32_to_usize_exact(1).unwrap(), 1);
        assert_eq!(i32_to_usize_exact(65535).unwrap(), 65535);
        assert_eq!(i32_to_usize_exact(2147483647).unwrap(), 2147483647);
    }

    #[test]
    #[should_panic]
    fn test_i32_to_usize_exact_with_panic() {
        i32_to_usize_exact(-1).unwrap();
    }

    fn var_doubles() -> [(f64, Vec<u8>); 17] {
        [
            (0.0, vec![0]),
            (1.0, vec![2]),
            (2.0, vec![3]),
            (3.0, vec![4]),
            (4.0, vec![132, 64]),
            (5.0, vec![5]),
            (6.0, vec![133, 64]),
            (7.0, vec![6]),
            (8.0, vec![134, 32]),
            (9.0, vec![134, 64]),
            (
                4.503599627370494E15,
                vec![231, 255, 255, 255, 255, 255, 255, 255, 128],
            ),
            (4.503599627370495E15, vec![104]),
            (
                4.503599627370496E15,
                vec![232, 128, 128, 128, 128, 128, 128, 128, 64],
            ),
            (
                9.00719925474099E15,
                vec![233, 255, 255, 255, 255, 255, 255, 255, 192],
            ),
            (9.007199254740991E15, vec![106]),
            (-1.0, vec![130, 128, 128, 128, 128, 128, 128, 128, 48]),
            (-0.5, vec![254, 128, 128, 128, 128, 128, 128, 128, 63]),
        ]
    }

    fn unsigned_var_longs() -> [(i64, Vec<u8>); 12] {
        [
            (0, vec![0]),
            (1, vec![1]),
            (127, vec![127]),
            (128, vec![128, 1]),
            (129, vec![129, 1]),
            (255, vec![255, 1]),
            (256, vec![128, 2]),
            (16383, vec![255, 127]),
            (16384, vec![128, 128, 1]),
            (16385, vec![129, 128, 1]),
            (-2, vec![254, 255, 255, 255, 255, 255, 255, 255, 255]),
            (-1, vec![255, 255, 255, 255, 255, 255, 255, 255, 255]),
        ]
    }

    fn signed_var_longs() -> [(i64, Vec<u8>); 29] {
        [
            (0, vec![0]),
            (1, vec![2]),
            (63, vec![126]),
            (64, vec![128, 1]),
            (65, vec![130, 1]),
            (127, vec![254, 1]),
            (128, vec![128, 2]),
            (8191, vec![254, 127]),
            (8192, vec![128, 128, 1]),
            (8193, vec![130, 128, 1]),
            (
                4611686018427387902,
                vec![252, 255, 255, 255, 255, 255, 255, 255, 127],
            ),
            (
                4611686018427387903,
                vec![254, 255, 255, 255, 255, 255, 255, 255, 127],
            ),
            (
                4611686018427387904,
                vec![128, 128, 128, 128, 128, 128, 128, 128, 128],
            ),
            (
                9223372036854775806,
                vec![252, 255, 255, 255, 255, 255, 255, 255, 255],
            ),
            (
                9223372036854775807,
                vec![254, 255, 255, 255, 255, 255, 255, 255, 255],
            ),
            (-1, vec![1]),
            (-63, vec![125]),
            (-64, vec![127]),
            (-65, vec![129, 1]),
            (-127, vec![253, 1]),
            (-128, vec![255, 1]),
            (-8191, vec![253, 127]),
            (-8192, vec![255, 127]),
            (-8193, vec![129, 128, 1]),
            (
                -4611686018427387903,
                vec![253, 255, 255, 255, 255, 255, 255, 255, 127],
            ),
            (
                -4611686018427387904,
                vec![255, 255, 255, 255, 255, 255, 255, 255, 127],
            ),
            (
                -4611686018427387905,
                vec![129, 128, 128, 128, 128, 128, 128, 128, 128],
            ),
            (
                -9223372036854775807,
                vec![253, 255, 255, 255, 255, 255, 255, 255, 255],
            ),
            (
                -9223372036854775808,
                vec![255, 255, 255, 255, 255, 255, 255, 255, 255],
            ),
        ]
    }

    #[test]
    fn test_encode_var_double() {
        let values = var_doubles();
        for value in values {
            let mut output = Output::with_capacity(32);
            encode_var_double(&mut output, value.0).unwrap();
            assert_eq!(value.1, output.trim());
        }
    }

    #[test]
    fn test_var_double_encoded_length() {
        let values = var_doubles();
        for value in values {
            let len = var_double_encoded_length(value.0);
            assert_eq!(value.1.len(), len as usize);
        }
    }

    #[test]
    fn test_unsigned_var_long_encoded_length() {
        let values = unsigned_var_longs();
        for value in values {
            let len = unsigned_var_long_encoded_length(value.0);
            assert_eq!(value.1.len(), len as usize);
        }
    }

    #[test]
    fn test_signed_var_long_encoded_length() {
        let values = signed_var_longs();
        for value in values {
            let len = signed_var_long_encoded_length(value.0);
            assert_eq!(value.1.len(), len as usize);
        }
    }

    #[test]
    fn test_encode_signed_var_long() {
        let values = signed_var_longs();
        for value in values {
            let mut output = Output::with_capacity(32);
            encode_signed_var_long(&mut output, value.0).unwrap();
            assert_eq!(value.1, output.trim());
        }
    }

    #[test]
    fn test_build_double() {
        assert_eq!(build_double(0, 1.0), 1.0);
    }
}
