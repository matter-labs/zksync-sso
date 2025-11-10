pub mod add;
pub mod installed;

use alloy::primitives::{Address, U256};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Module {
    pub address: Address,
    pub module_type: ModuleType,
}

impl Module {
    pub fn eoa_validator(address: Address) -> Self {
        Self::validator(address)
    }

    pub fn webauthn_validator(address: Address) -> Self {
        Self::validator(address)
    }

    pub fn session_key_validator(address: Address) -> Self {
        Self::validator(address)
    }

    pub fn guardian_executor(address: Address) -> Self {
        Self::executor(address)
    }

    pub fn validator(address: Address) -> Self {
        Self { address, module_type: ModuleType::Validator }
    }

    pub fn executor(address: Address) -> Self {
        Self { address, module_type: ModuleType::Executor }
    }

    pub fn fallback(address: Address) -> Self {
        Self { address, module_type: ModuleType::Fallback }
    }

    pub fn hook(address: Address) -> Self {
        Self { address, module_type: ModuleType::Hook }
    }

    pub fn prevalidation_hook_erc1271(address: Address) -> Self {
        Self { address, module_type: ModuleType::PrevalidationHookErc1271 }
    }

    pub fn prevalidation_hook_erc4337(address: Address) -> Self {
        Self { address, module_type: ModuleType::PrevalidationHookErc4337 }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ModuleType {
    Validator = 1,
    Executor = 2,
    Fallback = 3,
    Hook = 4,
    PrevalidationHookErc1271 = 8,
    PrevalidationHookErc4337 = 9,
}

impl From<ModuleType> for U256 {
    fn from(val: ModuleType) -> Self {
        U256::from(val as u8)
    }
}

impl TryFrom<U256> for ModuleType {
    type Error = eyre::Report;
    fn try_from(value: U256) -> Result<Self, Self::Error> {
        let byte_value = value.to::<u8>();
        match byte_value {
            1 => Ok(ModuleType::Validator),
            2 => Ok(ModuleType::Executor),
            3 => Ok(ModuleType::Fallback),
            4 => Ok(ModuleType::Hook),
            8 => Ok(ModuleType::PrevalidationHookErc1271),
            9 => Ok(ModuleType::PrevalidationHookErc4337),
            _ => Err(eyre::eyre!("Invalid module type")),
        }
    }
}
