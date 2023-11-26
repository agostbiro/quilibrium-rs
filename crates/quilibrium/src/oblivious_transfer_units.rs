//! Quilibrium token quantities.

use std::ops::Div;

use lazy_static::lazy_static;
use ruint::aliases::U256;

lazy_static! {
    static ref OT_UNIT_TO_QUIL_RATIO: U256 = U256::from(8_000_000_000_u64);
}

/// The maximum divisible unit of Quilibrium.
/// Represents a single bit in an oblivious transfer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(transparent)]
#[repr(transparent)]
pub struct ObliviousTransferUnits(U256);

impl ObliviousTransferUnits {
    /// Convert to QUIL tokens (floored).
    /// One QUIL token corresponds to 8 * 10^9 oblivious transfer units.
    pub fn quil_tokens(&self) -> U256 {
        self.0.div(*OT_UNIT_TO_QUIL_RATIO)
    }
}

impl TryFrom<&[u8]> for ObliviousTransferUnits {
    type Error = QuilTokenError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        // We check the length as `ruint` ignores leading zeroes and accepts any slice length that
        // fits into a `U256`, but when a slice other than 32 bytes length is passed here, that's
        // probably a mistake.
        if value.len() != U256::BITS / 8 {
            return Err(QuilTokenError::InvalidBytes(value.into()));
        }

        U256::try_from_be_slice(value)
            .map(Self)
            .ok_or_else(|| QuilTokenError::InvalidBytes(value.into()))
    }
}

/// Errors that occur when interacting with Quilibrium token quantities.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum QuilTokenError {
    /// The provided bytes are not valid oblivious transfer units.
    #[error("The provided bytes are not valid oblivious transfer units.")]
    InvalidBytes(Vec<u8>),
}

#[cfg(test)]
mod tests {
    use std::ops::{Add, Sub};

    use anyhow::Result;
    use ruint::uint;

    use super::*;

    #[test]
    fn try_from_slice_zero() -> Result<()> {
        let n = [0_u8; 32];
        let otu: ObliviousTransferUnits = n.as_slice().try_into()?;
        assert_eq!(otu, ObliviousTransferUnits(U256::ZERO));

        Ok(())
    }

    #[test]
    fn try_from_slice_one() -> Result<()> {
        let mut n = [0_u8; 32];
        n[31] = 1;

        let otu: ObliviousTransferUnits = n.as_slice().try_into()?;
        assert_eq!(otu, ObliviousTransferUnits(U256::from(1)));

        Ok(())
    }

    #[test]
    fn try_from_slice_endiannes() -> Result<()> {
        let n = U256::from(2).pow(U256::from(192));

        let otu: ObliviousTransferUnits = n.to_be_bytes::<32>().as_slice().try_into()?;
        assert_eq!(otu, ObliviousTransferUnits(n));

        let otu: ObliviousTransferUnits = n.to_le_bytes::<32>().as_slice().try_into()?;
        assert_eq!(
            otu,
            ObliviousTransferUnits(U256::from(2).pow(U256::from(56)))
        );

        Ok(())
    }

    #[test]
    fn try_from_slice_max() -> Result<()> {
        let n = U256::MAX;
        let otu: ObliviousTransferUnits = n.to_be_bytes::<32>().as_slice().try_into()?;
        assert_eq!(otu, ObliviousTransferUnits(n));

        Ok(())
    }

    #[test]
    fn try_from_slice_too_small() -> Result<()> {
        let result: Result<ObliviousTransferUnits, _> = [0; 31].as_slice().try_into();

        assert!(matches!(result, Err(QuilTokenError::InvalidBytes(bytes)) if bytes.len() == 31));

        Ok(())
    }

    #[test]
    fn try_from_slice_too_large() -> Result<()> {
        let result: Result<ObliviousTransferUnits, _> = [0; 33].as_slice().try_into();

        assert!(matches!(result, Err(QuilTokenError::InvalidBytes(bytes)) if bytes.len() == 33));

        Ok(())
    }

    #[test]
    fn quil_tokens_zero() {
        let otu = ObliviousTransferUnits(U256::ZERO);
        assert_eq!(otu.quil_tokens(), U256::ZERO);
    }

    #[test]
    fn quil_tokens_floors_zero() {
        let otu = ObliviousTransferUnits(OT_UNIT_TO_QUIL_RATIO.sub(U256::from(1)));
        assert_eq!(otu.quil_tokens(), U256::ZERO);
    }

    #[test]
    fn quil_tokens_floors_one() {
        let otu = ObliviousTransferUnits(OT_UNIT_TO_QUIL_RATIO.add(U256::from(1)));
        assert_eq!(otu.quil_tokens(), U256::from(1));
    }

    #[test]
    fn quil_tokens_conversion_token_balance() {
        // Token balance of a node at the start of the Dawn ceremony if it took part in phase 1.
        let otu = ObliviousTransferUnits(uint!(
            0x0000000000000000000000000000000000000000000000000000005d21dba000_U256
        ));
        assert_eq!(otu.quil_tokens(), U256::from(50));
    }

    #[test]
    fn quil_tokens_conversion_token_supply() {
        // Token supply at the start of the Dawn ceremony.
        let otu = ObliviousTransferUnits(uint!(
            0x0000000000000000000000000000000000000000000000000141d2c26be86000_U256
        ));
        assert_eq!(otu.quil_tokens(), U256::from(11_323_150));
    }

    #[test]
    fn quil_tokens_max() {
        let otu = ObliviousTransferUnits(U256::MAX);
        assert_eq!(otu.quil_tokens().log2(), 255 - OT_UNIT_TO_QUIL_RATIO.log2());
    }
}
