// Note: Contract addresses and node URL are now loaded from config.json
// Use ConfigLoader::load() to get the configuration

// Test account addresses from test_api_create_and_revoke_session
pub const OWNER_ADDRESS: &str = "0x6a34Ea49c29BF7Cce95F51E7F0f419831Ad5dBC6";
pub const TRANSFER_SESSION_TARGET: &str =
    "0xdeBbD4CE2Bd6BD869D3ac93666A0D5F4fc06FC72";
pub const SESSION_OWNER_ADDRESS: &str =
    "0x9BbC92a33F193174bf6Cc09c4b4055500d972479";

// Test keys and salts
pub const OWNER_PRIVATE_KEY: &str =
    "0x5de4111afa1a4b94908f83103c3a57e0c3c9e9da2dd5a02a84e9fde30d7e96c3";
pub const DEPLOY_PRIVATE_KEY: &str =
    "0x47e179ec197488593b187f80a00eb0da91f1b9d0b13f8733639f19c30a34926a";
pub const RANDOM_SALT_STR: &str = "sdk-test-factory-replication-010";

// Test session configuration
pub const EXPIRES_AT: u64 = 1749040108u64;
pub const FEE_LIMIT_LIFETIME: &str = "100000000000000000"; // 0.1 ETH
pub const MAX_VALUE_PER_USE: &str = "10000000000000000"; // 0.01 ETH

// Second session configuration (from test_api_create_session)
pub const SECOND_SESSION_OWNER_ADDRESS: &str =
    "0x90F79bf6EB2c4f870365E785982E1f101E93b906";
pub const SECOND_SESSION_OWNER_PRIVATE_KEY: &str =
    "0x5de4111afa1a4b94908f83103c3a57e0c3c9e9da2dd5a02a84e9fde30d7e96c3";
pub const SECOND_SESSION_EXPIRES_AT: u64 = 1767225600u64; // January 1st, 2026 00:00:00 UTC
pub const SECOND_SESSION_FEE_LIMIT: &str = "50000000000000000"; // 0.05 ETH
pub const SECOND_SESSION_MAX_VALUE_PER_USE: &str = "5000000000000000"; // 0.005 ETH
pub const VITALIK_ADDRESS: &str = "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045";
pub const EXPECTED_SECOND_SESSION_HASH: &str =
    "0x86c2b7aabf2f8c9cf63dee08c5dbdc102bb1dae07512184f2d538e4dba6c50fc";

// Session send test configuration (from test_api_session_send_integration)
pub const SESSION_SEND_TEST_PRIVATE_KEY: &str =
    "0x47e179ec197488593b187f80a00eb0da91f1b9d0b13f8733639f19c30a34926a";
pub const SESSION_SEND_TEST_OWNER_PRIVATE_KEY: &str =
    "0x5de4111afa1a4b94908f83103c3a57e0c3c9e9da2dd5a02a84e9fde30d7e96c3";
// Use a different session owner for send test to avoid conflicts with deploy_account
// This is Rich Wallet #5 (index 5) - not used elsewhere
pub const SESSION_SEND_TEST_SESSION_OWNER_PRIVATE_KEY: &str =
    "0x8b3a350cf5c34c9194ca85829a2df0ec3153be0318b5e2d3348e872092edffba";
pub const SESSION_SEND_TEST_SESSION_OWNER_ADDRESS: &str =
    "0x9965507D1a55bcC2695C58ba16FB37d819B0A4dc";
pub const SESSION_SEND_TEST_OWNER_ADDRESS: &str =
    "0x90F79bf6EB2c4f870365E785982E1f101E93b906";
pub const SESSION_SEND_TEST_TRANSFER_TARGET: &str =
    "0xdeBbD4CE2Bd6BD869D3ac93666A0D5F4fc06FC72";
pub const SESSION_SEND_TEST_EXPIRES_AT: u64 = 1767225600u64; // January 1, 2026, 00:00:00 UTC
pub const SESSION_SEND_TEST_UNIQUE_ACCOUNT_ID: &str = "session-send-test-002";
pub const SESSION_SEND_TEST_TRANSFER_AMOUNT: &str = "5000000000000000"; // 0.005 ETH
pub const SESSION_SEND_TEST_FUNDING_AMOUNT: &str = "1000000000000000000"; // 1 ETH
pub const SESSION_SEND_TEST_TRANSFER_AMOUNT_U64: u64 = 5000000000000000u64; // 0.005 ETH
pub const SESSION_SEND_TEST_FUNDING_AMOUNT_U64: u64 = 1000000000000000000u64; // 1 ETH
pub const SESSION_SEND_TEST_FEE_LIMIT: &str = "100000000000000000"; // 0.1 ETH
pub const SESSION_SEND_TEST_MAX_VALUE_PER_USE: &str = "10000000000000000"; // 0.01 ETH
pub const SESSION_SEND_TEST_EXPECTED_SESSION_KEY_BYTES: [u8; 32] = [
    139, 58, 53, 12, 245, 195, 76, 145, 148, 202, 133, 130, 154, 45, 240, 236,
    49, 83, 190, 3, 24, 181, 226, 211, 52, 142, 135, 32, 146, 237, 255, 186,
];

// Deterministic deployed account address (from consistent deployment with same salt/config)
pub const DEPLOYED_ACCOUNT_ADDRESS: &str =
    "0x177B4fe98b5F6ee253EFfFe1226c9C3E9f5e37cb";

/// Get test addresses as strings
pub fn get_test_addresses() -> TestAddresses {
    TestAddresses {
        owner: OWNER_ADDRESS,
        transfer_session_target: TRANSFER_SESSION_TARGET,
        session_owner: SESSION_OWNER_ADDRESS,
    }
}

pub struct TestAddresses {
    pub owner: &'static str,
    pub transfer_session_target: &'static str,
    pub session_owner: &'static str,
}

/// Create the standardized session configuration JSON used across deploy and revoke
pub fn create_session_config_json(
    session_owner_address: &str,
    transfer_session_target: &str,
    expires_at: u64,
) -> String {
    format!(
        r#"{{
        "signer": "{session_owner_address}",
        "expiresAt": "{expires_at}",
        "feeLimit": {{
            "limitType": "Lifetime",
            "limit": "{FEE_LIMIT_LIFETIME}",
            "period": "0"
        }},
        "callPolicies": [],
        "transferPolicies": [{{
            "target": "{transfer_session_target}",
            "maxValuePerUse": "{MAX_VALUE_PER_USE}",
            "valueLimit": {{
                "limitType": "Unlimited", 
                "limit": "0",
                "period": "0"
            }}
        }}]
    }}"#
    )
}

/// Create the second session configuration JSON (from test_api_create_session)
pub fn create_second_session_config_json() -> String {
    format!(
        r#"{{
        "signer": "{SECOND_SESSION_OWNER_ADDRESS}",
        "expiresAt": "{SECOND_SESSION_EXPIRES_AT}",
        "feeLimit": {{
            "limitType": "Lifetime",
            "limit": "{SECOND_SESSION_FEE_LIMIT}",
            "period": "0"
        }},
        "callPolicies": [],
        "transferPolicies": [{{
            "target": "{VITALIK_ADDRESS}",
            "maxValuePerUse": "{SECOND_SESSION_MAX_VALUE_PER_USE}",
            "valueLimit": {{
                "limitType": "Unlimited", 
                "limit": "0",
                "period": "0"
            }}
        }}]
    }}"#
    )
}

/// Create the session send test configuration JSON (from test_api_session_send_integration)
pub fn create_session_send_test_config_json() -> String {
    format!(
        r#"{{
        "signer": "{SESSION_SEND_TEST_SESSION_OWNER_ADDRESS}",
        "expiresAt": "{SESSION_SEND_TEST_EXPIRES_AT}",
        "feeLimit": {{
            "limitType": "Lifetime",
            "limit": "{SESSION_SEND_TEST_FEE_LIMIT}",
            "period": "0"
        }},
        "callPolicies": [],
        "transferPolicies": [{{
            "target": "{SESSION_SEND_TEST_TRANSFER_TARGET}",
            "maxValuePerUse": "{SESSION_SEND_TEST_MAX_VALUE_PER_USE}",
            "valueLimit": {{
                "limitType": "Unlimited", 
                "limit": "0",
                "period": "0"
            }}
        }}]
    }}"#
    )
}
