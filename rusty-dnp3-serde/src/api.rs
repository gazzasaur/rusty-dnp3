pub trait CrcCalculator {
    /// Computes the DNP3 CRC of a data block. This does not break data up, it calculates the CRC based on the entire data block passed in.
    fn compute_crc(data_block: &[u8]) -> u16;
}
