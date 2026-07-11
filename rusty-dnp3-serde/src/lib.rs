pub mod crc;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() -> Result<(), anyhow::Error> {
        let result = crc::compute_checksum(&[])?;
        assert_eq!(result, 4);
        Ok(())
    }
}
