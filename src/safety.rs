//! Safety supervision types shared between library tests and firmware.

/// Brown-out reset level programmed into STM32F7 option bytes.
///
/// `Level3` ≈ 2.1 V — appropriate for a 3.3 V ECU rail during cranking sags.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BorLevel {
    Off = 0,
    Level1 = 1,
    Level2 = 2,
    Level3 = 3,
}

impl BorLevel {
    pub const TARGET: Self = Self::Level3;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn target_bor_is_not_off() {
        assert_ne!(BorLevel::TARGET, BorLevel::Off);
    }
}
