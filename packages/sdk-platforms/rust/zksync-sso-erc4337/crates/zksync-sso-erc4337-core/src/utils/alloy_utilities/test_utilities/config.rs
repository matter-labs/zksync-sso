#[derive(Debug, Clone)]
pub struct TestInfraConfig {
    pub signer_private_key: String,
}

impl Default for TestInfraConfig {
    fn default() -> Self {
        Self {
            signer_private_key: "0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d"
                .to_string(),
        }
    }
}
