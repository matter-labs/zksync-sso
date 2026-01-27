use eyre::{Result, WrapErr};
use serde_json::json;
use std::{
    env, fs,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};
use url::Url;

const ACCOUNT_ABSTRACTION_REPO: &str =
    "https://github.com/eth-infinitism/account-abstraction";
const ACCOUNT_ABSTRACTION_TAG: &str = "v0.8.0";
pub(crate) const ENTRYPOINT_ADDRESS: &str =
    "0x4337084D9E255Ff0702461CF8895CE9E3b5Ff108";
const FUNDER_PRIVATE_KEY: &str =
    "0x7726827caac94a7f9e1b160f7ea819f172f7b6f9d2a97f992c38edeab82d4110";
const FUNDED_WALLET_9: &str = "0xa0Ee7A142d267C1f36714E4a8F75612F20a79720";
const FUNDED_WALLET_1: &str = "0x70997970C51812dc3A010C7d01b50e0d17dc79C8";
const FUND_AMOUNT_WEI: u128 = 10_000_000_000_000_000_000;

pub(crate) fn setup_zksync_os(
    checkout_dir: &Path,
    l2_rpc_url: &Url,
    print_logs: bool,
    deploy_entrypoint: bool,
    fund_wallet: bool,
    deploy_test_contracts: bool,
) -> Result<()> {
    assert!(deploy_entrypoint, "deploy_entrypoint must be true");
    assert!(fund_wallet, "fund_wallet must be true");
    if !deploy_entrypoint && !fund_wallet && !deploy_test_contracts {
        return Ok(());
    }

    if deploy_entrypoint {
        let entrypoint_deployed = rpc_has_code(l2_rpc_url, ENTRYPOINT_ADDRESS)?;
        assert!(
            !entrypoint_deployed,
            "EntryPoint should not be deployed at this stage"
        );
        if !entrypoint_deployed {
            let account_abstraction_dir =
                resolve_account_abstraction_dir(checkout_dir);
            ensure_account_abstraction_repo(&account_abstraction_dir)?;
            checkout_account_abstraction_tag(&account_abstraction_dir)?;
            ensure_zksyncos_network(&account_abstraction_dir)?;
            deploy_entrypoint_contract(&account_abstraction_dir, print_logs)?;
        }
    }
    let entrypoint_deployed = rpc_has_code(l2_rpc_url, ENTRYPOINT_ADDRESS)?;
    assert!(entrypoint_deployed, "EntryPoint should be deployed");

    if fund_wallet {
        for address in resolve_fund_targets() {
            let balance = rpc_get_balance(l2_rpc_url, &address)?;
            if balance < FUND_AMOUNT_WEI {
                fund_wallet_balance(l2_rpc_url, &address, print_logs)?;
            }
            let new_balance = rpc_get_balance(l2_rpc_url, &address)?;
            eyre::ensure!(
                new_balance >= FUND_AMOUNT_WEI,
                "failed to fund {address}: balance {new_balance} below {FUND_AMOUNT_WEI}"
            );
        }
    }

    if deploy_test_contracts {
        let contracts_dir = resolve_contracts_dir()?;
        deploy_test_contracts_to_l2(&contracts_dir, print_logs)?;
    }

    Ok(())
}

fn resolve_account_abstraction_dir(checkout_dir: &Path) -> PathBuf {
    checkout_dir
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("account-abstraction")
}

fn ensure_account_abstraction_repo(path: &Path) -> Result<()> {
    if path.join("package.json").exists() {
        return Ok(());
    }

    let status = Command::new("git")
        .args(["clone", "--depth", "1", ACCOUNT_ABSTRACTION_REPO])
        .current_dir(path.parent().unwrap_or_else(|| Path::new(".")))
        .status()
        .wrap_err("failed to clone account-abstraction repository")?;

    eyre::ensure!(
        status.success(),
        "git clone account-abstraction exited with {:?}",
        status
    );
    Ok(())
}

fn checkout_account_abstraction_tag(path: &Path) -> Result<()> {
    let status = Command::new("git")
        .args(["fetch", "--tags"])
        .current_dir(path)
        .status()
        .wrap_err("failed to fetch account-abstraction tags")?;
    eyre::ensure!(
        status.success(),
        "git fetch --tags exited with {:?}",
        status
    );

    let status = Command::new("git")
        .args(["checkout", ACCOUNT_ABSTRACTION_TAG])
        .current_dir(path)
        .status()
        .wrap_err("failed to checkout account-abstraction tag")?;
    eyre::ensure!(
        status.success(),
        "git checkout {} exited with {:?}",
        ACCOUNT_ABSTRACTION_TAG,
        status
    );
    Ok(())
}

fn ensure_zksyncos_network(path: &Path) -> Result<()> {
    let config_path = path.join("hardhat.config.ts");
    let contents = fs::read_to_string(&config_path)
        .wrap_err("failed to read hardhat.config.ts")?;
    if contents.lines().any(|line| {
        line.trim_start().starts_with("zksyncos:") || line.contains("zksyncos:")
    }) {
        return Ok(());
    }

    let mut lines: Vec<String> =
        contents.lines().map(|line| line.to_string()).collect();
    let mut inserted = false;
    for idx in 0..lines.len() {
        if lines[idx].contains("networks") && lines[idx].contains('{') {
            let indent: String =
                lines[idx].chars().take_while(|c| c.is_whitespace()).collect();
            let entry = format!(
                "{indent}  zksyncos: {{ url: 'http://localhost:3050', accounts: ['{FUNDER_PRIVATE_KEY}'] }},"
            );
            lines.insert(idx + 1, entry);
            inserted = true;
            break;
        }
    }

    eyre::ensure!(
        inserted,
        "failed to locate networks block in hardhat.config.ts"
    );

    let mut updated = lines.join("\n");
    if contents.ends_with('\n') {
        updated.push('\n');
    }
    fs::write(&config_path, updated)
        .wrap_err("failed to write hardhat.config.ts")?;
    Ok(())
}

fn deploy_entrypoint_contract(path: &Path, print_logs: bool) -> Result<()> {
    let mut install_cmd = yarn_v1_command();
    install_cmd.arg("install").current_dir(path);
    run_command(install_cmd, print_logs, "yarn@1 install")?;

    let mut deploy_cmd = yarn_v1_command();
    deploy_cmd.args(["deploy", "--network", "zksyncos"]).current_dir(path);
    run_command(deploy_cmd, print_logs, "yarn@1 deploy --network zksyncos")?;
    Ok(())
}

fn yarn_v1_command() -> Command {
    let mut cmd = Command::new("npx");
    cmd.args(["-y", "yarn@1.22.22"]);
    cmd.env("COREPACK_ENABLE_STRICT", "0")
        .env("COREPACK_ENABLE_PROJECT_SPEC", "0");
    cmd
}

fn resolve_fund_targets() -> Vec<String> {
    let mut targets = Vec::new();
    push_unique(&mut targets, FUNDED_WALLET_9);
    push_unique(&mut targets, FUNDED_WALLET_1);
    if let Ok(raw) = env::var("SSO_ZKSYNC_OS_FUND_ADDRESSES") {
        for entry in raw.split([',', ' ']) {
            let trimmed = entry.trim();
            if trimmed.is_empty() {
                continue;
            }
            push_unique(&mut targets, trimmed);
        }
    }
    targets
}

fn push_unique(targets: &mut Vec<String>, address: &str) {
    if !targets.iter().any(|entry| entry == address) {
        targets.push(address.to_string());
    }
}

fn fund_wallet_balance(
    l2_rpc_url: &Url,
    address: &str,
    print_logs: bool,
) -> Result<()> {
    let mut cmd = Command::new("cast");
    cmd.args([
        "send",
        "--private-key",
        FUNDER_PRIVATE_KEY,
        "--rpc-url",
        l2_rpc_url.as_str(),
        address,
        "--value",
        "10000000000000000000",
    ]);

    run_command(cmd, print_logs, "cast send")?;
    Ok(())
}

fn resolve_contracts_dir() -> Result<PathBuf> {
    if let Some(dir) = env::var_os("SSO_ERC4337_CONTRACTS_DIR") {
        return Ok(PathBuf::from(dir));
    }

    let start_dir = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    if let Some(dir) = find_contracts_dir(&start_dir) {
        return Ok(dir);
    }

    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    if let Some(dir) = find_contracts_dir(&manifest_dir) {
        return Ok(dir);
    }

    Err(eyre::eyre!("Could not find erc4337-contracts directory"))
}

fn find_contracts_dir(start: &Path) -> Option<PathBuf> {
    let mut search_dir = start.to_path_buf();

    loop {
        let contracts_dir = search_dir.join("erc4337-contracts");
        if contracts_dir.is_dir() {
            return Some(contracts_dir);
        }

        let packages_dir =
            search_dir.join("packages").join("erc4337-contracts");
        if packages_dir.is_dir() {
            return Some(packages_dir);
        }

        let parent = search_dir.parent()?;
        search_dir = parent.to_path_buf();
    }
}

fn deploy_test_contracts_to_l2(
    contracts_dir: &Path,
    print_logs: bool,
) -> Result<()> {
    let mut cmd = Command::new("pnpm");
    cmd.args(["deploy-test:zksync-os"]).current_dir(contracts_dir);
    run_command(cmd, print_logs, "pnpm deploy-test:zksync-os")?;
    Ok(())
}

fn run_command(mut cmd: Command, print_logs: bool, label: &str) -> Result<()> {
    if print_logs {
        cmd.stdout(Stdio::inherit()).stderr(Stdio::inherit());
        let status = cmd
            .status()
            .wrap_err_with(|| format!("failed to run command: {label}"))?;
        eyre::ensure!(
            status.success(),
            "command {label} exited with {:?}",
            status
        );
        return Ok(());
    }

    let output = cmd
        .output()
        .wrap_err_with(|| format!("failed to run command: {label}"))?;
    if !output.status.success() {
        return Err(eyre::eyre!(
            "command {label} failed (status: {:?})\nstdout:\n{}\nstderr:\n{}",
            output.status,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    Ok(())
}

pub(crate) fn rpc_has_code(rpc_url: &Url, address: &str) -> Result<bool> {
    let result = rpc_call(rpc_url, "eth_getCode", json!([address, "latest"]))?;
    let code = result
        .as_str()
        .ok_or_else(|| eyre::eyre!("eth_getCode returned non-string"))?;
    Ok(code != "0x" && code != "0x0")
}

fn rpc_get_balance(rpc_url: &Url, address: &str) -> Result<u128> {
    let result =
        rpc_call(rpc_url, "eth_getBalance", json!([address, "latest"]))?;
    let balance = result
        .as_str()
        .ok_or_else(|| eyre::eyre!("eth_getBalance returned non-string"))?;
    parse_hex_u128(balance)
}

fn rpc_call(
    rpc_url: &Url,
    method: &str,
    params: serde_json::Value,
) -> Result<serde_json::Value> {
    let agent = ureq::AgentBuilder::new()
        .timeout_read(std::time::Duration::from_secs(2))
        .timeout_write(std::time::Duration::from_secs(2))
        .build();
    let body = json!({
        "jsonrpc": "2.0",
        "method": method,
        "params": params,
        "id": 1
    })
    .to_string();

    let response = agent
        .post(rpc_url.as_str())
        .set("Content-Type", "application/json")
        .send_string(&body)
        .wrap_err("failed to send rpc request")?;
    let payload_text =
        response.into_string().wrap_err("failed to read rpc response body")?;
    let payload: serde_json::Value = serde_json::from_str(&payload_text)
        .wrap_err("failed to parse rpc response")?;
    let result = payload
        .get("result")
        .ok_or_else(|| eyre::eyre!("rpc response missing result"))?;
    Ok(result.clone())
}

fn parse_hex_u128(value: &str) -> Result<u128> {
    let trimmed = value.trim_start_matches("0x");
    if trimmed.is_empty() {
        return Ok(0);
    }
    u128::from_str_radix(trimmed, 16).wrap_err("failed to parse hex value")
}
