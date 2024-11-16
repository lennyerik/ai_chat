pub trait LargeLanguageModel<'c> {
    type Error: std::error::Error;
    type Response: 'c + fallible_iterator::FallibleIterator<Item = String, Error = Self::Error>;

    fn send_message(&'c mut self, message: &str) -> Result<Self::Response, Self::Error>;
}
