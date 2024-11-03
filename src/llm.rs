pub trait LargeLanguageModel<'c> {
    type Error: std::error::Error;
    type Response: 'c + Iterator<Item = Result<String, Self::Error>>;

    fn send_message(&'c mut self, message: &str) -> Result<Self::Response, Self::Error>;
}
