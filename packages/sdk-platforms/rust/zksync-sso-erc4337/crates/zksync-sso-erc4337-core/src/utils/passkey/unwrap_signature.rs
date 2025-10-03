use crate::utils::passkey::normalize_s;
use alloy::primitives::{FixedBytes, hex};
use der::{
    Decode, DecodeValue, Encode, EncodeValue, Header, Reader, Sequence, Writer,
    asn1::UintRef,
};
use eyre::Result;
use log::debug;

pub struct UnwrappedSignature {
    pub r: FixedBytes<32>,
    pub s: FixedBytes<32>,
}

#[derive(Debug, Eq, PartialEq)]
struct ECDSASignature<'a> {
    r: UintRef<'a>,
    s: UintRef<'a>,
}

impl<'a> DecodeValue<'a> for ECDSASignature<'a> {
    fn decode_value<R: Reader<'a>>(
        reader: &mut R,
        _header: Header,
    ) -> der::Result<Self> {
        let r = reader.decode()?;
        let s = reader.decode()?;
        Ok(Self { r, s })
    }
}

impl EncodeValue for ECDSASignature<'_> {
    fn value_len(&self) -> der::Result<der::Length> {
        self.r.encoded_len()? + self.s.encoded_len()?
    }

    fn encode_value(&self, writer: &mut impl Writer) -> der::Result<()> {
        self.r.encode(writer)?;
        self.s.encode(writer)?;
        Ok(())
    }
}

impl<'a> Sequence<'a> for ECDSASignature<'a> {}

fn should_remove_leading_zero(bytes: &[u8]) -> bool {
    bytes.len() > 1 && bytes[0] == 0x0 && (bytes[1] & (1 << 7)) != 0
}

pub fn unwrap_ec2_signature(signature: &[u8]) -> Result<UnwrappedSignature> {
    debug!("Input signature (hex): {}", hex::encode(signature));

    let sig = ECDSASignature::from_der(signature)
        .map_err(|e| eyre::eyre!("Failed to parse DER signature: {}", e))?;

    debug!(
        "Parsed ASN.1 signature: {{\"r\": \"{}\", \"s\": \"{}\"}}",
        hex::encode(sig.r.as_bytes()),
        hex::encode(sig.s.as_bytes())
    );

    let mut r_bytes = sig.r.as_bytes();
    let mut s_bytes = sig.s.as_bytes();

    debug!(
        "Initial r and s bytes: {{\"r\": \"{}\", \"s\": \"{}\"}}",
        hex::encode(r_bytes),
        hex::encode(s_bytes)
    );

    if should_remove_leading_zero(r_bytes) {
        debug!("Removing leading zero from r");
        r_bytes = &r_bytes[1..];
    }

    if should_remove_leading_zero(s_bytes) {
        debug!("Removing leading zero from s");
        s_bytes = &s_bytes[1..];
    }

    debug!(
        "After removing leading zeros: {{\"r\": \"{}\", \"s\": \"{}\"}}",
        hex::encode(r_bytes),
        hex::encode(s_bytes)
    );

    let r_array: [u8; 32] =
        r_bytes.try_into().map_err(|_| eyre::eyre!("Invalid r length"))?;
    let r = FixedBytes::from_slice(&r_array);

    let s_array: [u8; 32] =
        s_bytes.try_into().map_err(|_| eyre::eyre!("Invalid s length"))?;
    let s = normalize_s::normalize_s(FixedBytes::from_slice(&s_array))?;

    debug!(
        "After normalizing s: {{\"r\": \"{}\", \"s\": \"{}\"}}",
        hex::encode(r.as_slice()),
        hex::encode(s.as_slice())
    );

    Ok(UnwrappedSignature { r, s })
}

#[cfg(test)]
mod tests {
    use super::*;
    use eyre::Result;

    #[test]
    fn test_should_remove_leading_zero() {
        let bytes = [0x00, 0x80];
        assert!(should_remove_leading_zero(&bytes));

        let bytes = [0x01, 0x80];
        assert!(!should_remove_leading_zero(&bytes));

        let bytes = [0x00, 0x7F];
        assert!(!should_remove_leading_zero(&bytes));

        let bytes = [0x01, 0x7F];
        assert!(!should_remove_leading_zero(&bytes));

        let bytes = [0x00, 0xd6, 0xed, 0x7d, 0xbc];
        assert!(should_remove_leading_zero(&bytes));

        let bytes = [];
        assert!(!should_remove_leading_zero(&bytes));
    }

    #[test]
    fn test_unwrap_ec2_signature_matches_typescript() -> Result<()> {
        let signature_hex = "304502201e6bd398700475910fb66389f177f6d4aec39230e20c29f019457c0867e30778022100d6ed7dbc7e7dd2b97e1565aec0255217a6c3b03446e1623d52eff2de92aac55f";
        let signature = hex::decode(signature_hex)?;

        let unwrapped = unwrap_ec2_signature(&signature)?;

        assert_eq!(
            hex::encode(unwrapped.r.as_slice()),
            "1e6bd398700475910fb66389f177f6d4aec39230e20c29f019457c0867e30778"
        );

        assert_eq!(
            hex::encode(unwrapped.s.as_slice()),
            "2912824281822d4781ea9a513fdaade816234a7960363c47a0c9d7e469b85ff2"
        );

        assert_eq!(unwrapped.r.len(), 32);
        assert_eq!(unwrapped.s.len(), 32);

        Ok(())
    }
}
