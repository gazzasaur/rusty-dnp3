pub mod crc;
pub mod api;
pub mod datalink;
pub mod transport;
pub mod application;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() -> Result<(), anyhow::Error> {
        Ok(())
    }
}
