pub trait LargeLanguageModel<'c> {
    type Error: std::error::Error;
    type Response: 'c + LLMResponse<Self::Error>;

    fn send_message(&'c mut self, message: &str) -> Result<Self::Response, Self::Error>;
}

pub trait LLMResponse<E: std::error::Error>: Iterator<Item = String> {
    fn result(self) -> Result<(), E>;
}
