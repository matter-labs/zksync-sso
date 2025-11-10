use crate::erc4337::account::modular_smart_account::{
    session::{
        contract::SessionLib::SessionSpec as SessionLibSessionSpec,
        period_id::get_period_id, session_lib::session_spec::SessionSpec,
    },
    signers::eoa::eoa_sign,
};
use alloy::{
    dyn_abi::SolType,
    primitives::{Address, Bytes, FixedBytes, Uint},
    sol,
};

pub fn session_signature(
    private_key_hex: &str,
    session_validator: Address,
    session_spec: &SessionSpec,
    hash: FixedBytes<32>,
) -> eyre::Result<Bytes> {
    let session_validator_bytes = session_validator.0.to_vec();
    let signature_bytes = eoa_sign(private_key_hex, hash)?;

    let period_ids =
        vec![get_period_id(&session_spec.fee_limit), Uint::from(0)];

    let fat_signature = {
        type SessionSignature =
            sol! { tuple(bytes, SessionLibSessionSpec, uint48[]) };
        let spec: SessionLibSessionSpec = session_spec.to_owned().into();
        SessionSignature::abi_encode_params(&(
            signature_bytes,
            spec.clone(),
            period_ids,
        ))
    };

    let bytes = [session_validator_bytes, fat_signature].concat();

    Ok(bytes.into())
}
