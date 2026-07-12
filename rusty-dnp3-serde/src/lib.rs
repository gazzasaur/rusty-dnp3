pub mod crc;
pub mod api;
pub mod datalink;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() -> Result<(), anyhow::Error> {
        Ok(())
    }
}
