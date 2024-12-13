use uniffi;

uniffi::setup_scaffolding!();

#[uniffi::export]
fn greet(name: &str) -> String {
    format!("Hello, {}", name)
}

#[uniffi::export]
fn add(left: u64, right: u64) -> u64 {
    sdk::add(left, right)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }

    #[test]
    fn greeting_works() {
        let greeting = greet("Rust");
        assert_eq!("Hello, Rust", greeting);
    }
}
