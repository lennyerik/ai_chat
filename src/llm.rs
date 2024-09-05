pub trait LargeLanguageModel: Iterator<Item = Result<String, Self::Error>> {
    type Error;

    fn send_message(&mut self, message: &str) -> Result<(), Self::Error>;
}
