/// Easy to convert u64 to u16, capping at max value if the u64 is too big.
pub fn u64_to_u16(input: u64) -> u16 {
    u16::try_from(input).unwrap_or(u16::MAX)
}
