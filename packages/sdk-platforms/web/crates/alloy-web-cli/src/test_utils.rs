use std::process::Command;
use std::time::Duration;
use tokio::time::sleep;

/// Helper to run CLI commands with timeout
pub fn run_cli_command(args: &[&str]) -> Result<(String, String), String> {
    // Get the absolute path to the CLI binary
    let cli_path = std::env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("alloy-web-cli");

    println!(
        "🔧 Running CLI command: {:?} with binary: {:?}",
        args, cli_path
    );

    let output = Command::new(&cli_path)
        .args(args)
        .output()
        .map_err(|e| format!("Failed to execute command: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    println!("📤 CLI command completed with status: {}", output.status);
    println!("📤 CLI stdout: {}", stdout);
    println!("📤 CLI stderr: {}", stderr);

    if output.status.success() {
        Ok((stdout, stderr))
    } else {
        Err(format!(
            "Command failed:\nstdout: {}\nstderr: {}",
            stdout, stderr
        ))
    }
}

/// Helper to wait for a port condition (either available or not available)
async fn wait_for_port_condition(
    port: u16,
    should_be_available: bool,
    max_wait_secs: u64,
    poll_interval_ms: u64,
) -> Result<(), String> {
    let start = std::time::Instant::now();
    let max_duration = Duration::from_secs(max_wait_secs);
    let status = if should_be_available {
        "available"
    } else {
        "unavailable"
    };

    println!(
        "🔍 Polling port {} every {}ms to become {}",
        port, poll_interval_ms, status
    );

    while start.elapsed() < max_duration {
        let is_available = check_port(port);
        let elapsed = start.elapsed().as_secs();
        println!(
            "⏱️  [{}s] Port {} is {}, waiting for {}",
            elapsed,
            port,
            if is_available {
                "available"
            } else {
                "unavailable"
            },
            status
        );

        if is_available == should_be_available {
            println!(
                "✅ Port {} is now {} after {} seconds",
                port, status, elapsed
            );
            return Ok(());
        }
        sleep(Duration::from_millis(poll_interval_ms)).await;
    }

    Err(format!(
        "Port {} failed to become {} after {} seconds",
        port, status, max_wait_secs
    ))
}

/// Helper to run a service command and wait for the expected port state
async fn run_service_command_and_wait(
    args: &[&str],
    port: u16,
    port_should_be_available: bool,
    success_message: Option<&str>,
    max_wait_secs: u64,
) -> Result<(String, String), String> {
    println!("🚀 Running CLI command: {:?}", args);

    // Run the command
    let result = run_cli_command(args)?;
    println!(
        "📋 Command output - stdout: {}, stderr: {}",
        result.0, result.1
    );

    // If we have a success message to check for and it's already there, return immediately
    if let Some(msg) = success_message {
        if result.0.contains(msg) || result.1.contains(msg) {
            println!("✅ Found success message '{}' in command output", msg);
            return Ok(result);
        }
    }

    // Wait for the port condition
    println!(
        "⏳ Waiting for port {} to be {} (max {} seconds)",
        port,
        if port_should_be_available {
            "available"
        } else {
            "unavailable"
        },
        max_wait_secs
    );
    let poll_interval = if port_should_be_available { 1000 } else { 500 };
    wait_for_port_condition(port, port_should_be_available, max_wait_secs, poll_interval).await?;
    println!("✅ Port condition met for port {}", port);

    // If we have a success message and didn't find it in the output, add it
    if let Some(msg) = success_message {
        if !result.0.contains(msg) && !result.1.contains(msg) {
            return Ok((format!("{}\n{}", result.0, msg), result.1));
        }
    }

    Ok(result)
}

/// Helper to start a service and wait for it to be ready
pub async fn start_service_and_wait(
    args: &[&str],
    port: u16,
    success_message: &str,
    max_wait_secs: u64,
) -> Result<(String, String), String> {
    println!(
        "🔄 Starting service with args: {:?}, waiting for port {} to be available",
        args, port
    );
    let result =
        run_service_command_and_wait(args, port, true, Some(success_message), max_wait_secs).await;
    match &result {
        Ok(_) => println!("✅ Service started successfully on port {}", port),
        Err(e) => println!("❌ Failed to start service: {}", e),
    }
    result
}

/// Helper to stop a service and wait for it to actually stop
pub async fn stop_service_and_wait(
    args: &[&str],
    port: u16,
    max_wait_secs: u64,
) -> Result<(String, String), String> {
    run_service_command_and_wait(args, port, false, None, max_wait_secs).await
}

/// Helper to check if a service is running on a port
pub fn check_port(port: u16) -> bool {
    Command::new("nc")
        .args(&["-z", "localhost", &port.to_string()])
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

/// Helper to make RPC call
pub fn rpc_call(port: u16, method: &str) -> Result<String, String> {
    let output = Command::new("curl")
        .args(&[
            "-s",
            &format!("http://localhost:{}", port),
            "-X",
            "POST",
            "-H",
            "Content-Type: application/json",
            "-d",
            &format!(
                r#"{{"jsonrpc":"2.0","method":"{}","params":[],"id":1}}"#,
                method
            ),
        ])
        .output()
        .map_err(|e| format!("Failed to execute curl: {}", e))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(format!(
            "RPC call failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ))
    }
}
