pub struct BundlerConfig {
    url: String,
}

impl BundlerConfig {
    pub fn new(url: String) -> Self {
        BundlerConfig { url }
    }

    pub fn url(&self) -> String {
        self.url.clone()
    }
}
