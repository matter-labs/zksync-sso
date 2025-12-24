use serial_test::serial;
use std::{env, path::PathBuf};
use tempfile::tempdir;
use zksync_sso_zksyncos_node::{
    config::SpawnConfig,
    instance::{DEFAULT_L1_PORT, DEFAULT_REPO_URL, DEFAULT_RPC_PORT},
};

const DIR_ENV: &str = "SSO_ZKSYNC_OS_SERVER_DIR";
const REPO_ENV: &str = "SSO_ZKSYNC_OS_SERVER_REPO";
const L1_PORT_ENV: &str = "SSO_ZKSYNC_OS_L1_PORT";
const RPC_PORT_ENV: &str = "SSO_ZKSYNC_OS_RPC_PORT";
const SKIP_BUILD_ENV: &str = "SSO_ZKSYNC_OS_SKIP_BUILD";
const PRINT_LOGS_ENV: &str = "SSO_ZKSYNC_OS_PRINT_LOGS";

fn set_env_var<K: AsRef<std::ffi::OsStr>, V: AsRef<std::ffi::OsStr>>(
    key: K,
    val: V,
) {
    unsafe { env::set_var(key, val) };
}

fn clear_env() {
    for key in [
        DIR_ENV,
        REPO_ENV,
        L1_PORT_ENV,
        RPC_PORT_ENV,
        SKIP_BUILD_ENV,
        PRINT_LOGS_ENV,
    ] {
        unsafe { env::remove_var(key) };
    }
}

#[test]
#[serial]
fn spawn_config_respects_env_overrides() {
    clear_env();
    let temp = tempdir().expect("temp dir");
    let checkout = temp.path().join("checkout");
    set_env_var(DIR_ENV, &checkout);
    set_env_var(REPO_ENV, "https://example.com/custom.git");
    set_env_var(L1_PORT_ENV, "9123");
    set_env_var(RPC_PORT_ENV, "4123");
    set_env_var(SKIP_BUILD_ENV, "1");
    set_env_var(PRINT_LOGS_ENV, "0");

    let cfg = SpawnConfig::default();

    assert_eq!(cfg.checkout_dir, PathBuf::from(&checkout));
    assert_eq!(cfg.repo_url, "https://example.com/custom.git");
    assert_eq!(cfg.l1_port, 9123);
    assert_eq!(cfg.rpc_port, 4123);
    assert!(cfg.skip_build);
    assert!(!cfg.print_logs);
    clear_env();
}

#[test]
#[serial]
fn spawn_config_defaults_match_expectations() {
    clear_env();
    let cfg = SpawnConfig::default();
    let expected_suffix = PathBuf::from("zksync-os-server");

    assert!(
        cfg.checkout_dir.ends_with(&expected_suffix),
        "expected checkout to end with {}",
        expected_suffix.display()
    );
    assert_eq!(cfg.repo_url, DEFAULT_REPO_URL);
    assert_eq!(cfg.l1_port, DEFAULT_L1_PORT);
    assert_eq!(cfg.rpc_port, DEFAULT_RPC_PORT);
    assert!(cfg.skip_build);
    assert!(cfg.print_logs);
}
