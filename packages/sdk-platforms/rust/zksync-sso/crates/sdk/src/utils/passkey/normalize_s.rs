use alloy::primitives::{FixedBytes, U256, hex};
use eyre::Result;
use log::debug;

pub fn normalize_s(s: FixedBytes<32>) -> Result<FixedBytes<32>> {
    debug!("Input s value (hex): {}", hex::encode(s.as_slice()));

    let n = U256::from_str_radix(
        "FFFFFFFF00000000FFFFFFFFFFFFFFFFBCE6FAADA7179E84F3B9CAC2FC632551",
        16,
    )?;
    debug!("Curve order n: {n:x}");

    let half_n: U256 = n >> 1;
    debug!("Half of curve order (n/2): {half_n:x}");

    let s_num = U256::from_be_bytes(s.0);
    debug!("s as bigint: {s_num:x}");

    let needs_normalization = s_num > half_n;
    debug!("Needs normalization: {needs_normalization}");

    if needs_normalization {
        let normalized = n - s_num;
        println!("Normalized s value: {normalized:x}");
        Ok(FixedBytes::from(normalized.to_be_bytes::<32>()))
    } else {
        println!("S value already normalized");
        Ok(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy::primitives::{FixedBytes, U256};
    use eyre::Result;

    #[test]
    fn test_normalize_s_matches_typescript() -> Result<()> {
        let s_hex =
            "d6ed7dbc7e7dd2b97e1565aec0255217a6c3b03446e1623d52eff2de92aac55f";
        let decoded: Vec<u8> = hex::decode(s_hex)?;
        let bytes: [u8; 32] =
            decoded.try_into().map_err(|_| eyre::eyre!("Invalid length"))?;
        let s = FixedBytes::from(bytes);

        let normalized = normalize_s(s)?;

        assert_eq!(
            hex::encode(normalized.as_slice()),
            "2912824281822d4781ea9a513fdaade816234a7960363c47a0c9d7e469b85ff2"
        );

        assert_eq!(normalized.len(), 32);

        let n = U256::from_str_radix(
            "FFFFFFFF00000000FFFFFFFFFFFFFFFFBCE6FAADA7179E84F3B9CAC2FC632551",
            16,
        )?;
        let half_n: U256 = n >> 1;
        let normalized_value = U256::from_be_bytes(normalized.0);
        assert!(normalized_value <= half_n, "Value not properly normalized");

        Ok(())
    }
}
