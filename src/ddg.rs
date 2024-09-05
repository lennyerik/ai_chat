use super::llm;

const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/126.0.0.0 Safari/537.36";

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("network request error")]
    RequestError(#[from] Box<ureq::Error>),

    #[error("no new Vqd received with last request")]
    NoVqdReceived,
}

#[derive(Debug, Clone, Copy)]
pub enum DDGChatModel {
    GPT4oMini,
    Claude3Haiku,
    Llama370B,
    Mixtral8x7B,
}

impl From<DDGChatModel> for &str {
    fn from(model: DDGChatModel) -> Self {
        match model {
            DDGChatModel::GPT4oMini => "gpt-4o-mini",
            DDGChatModel::Claude3Haiku => "claude-3-haiku-20240307",
            DDGChatModel::Llama370B => "meta-llama/Meta-Llama-3.1-70B-Instruct-Turbo",
            DDGChatModel::Mixtral8x7B => "mistralai/Mixtral-8x7B-Instruct-v0.1",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum MessageRole {
    User,
    Assistant,
}

#[derive(Debug, Clone)]
pub struct DDGMessage {
    role: MessageRole,
    text: String,
}

#[derive(Debug)]
pub struct DDGChat {
    model: DDGChatModel,
    messages: Vec<DDGMessage>,
    current_vqd: String,
    current_response: Option<ureq::Response>,
}

impl DDGChat {
    pub fn new(model: DDGChatModel) -> Result<Self, Error> {
        let vqd_resp = ureq::get("https://duckduckgo.com/duckchat/v1/status")
            .set("User-Agent", USER_AGENT)
            .set("X-Vqd-Accept", "1")
            .call()
            .map_err(Box::new)?;
        let vqd = vqd_resp.header("X-Vqd-4").ok_or(Error::NoVqdReceived)?;

        Ok(Self {
            model,
            messages: Vec::new(),
            current_vqd: vqd.to_owned(),
            current_response: None,
        })
    }
}

impl Iterator for DDGChat {
    type Item = Result<String, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(response) = &self.current_response {
            todo!()
        } else {
            None
        }
    }
}

impl llm::LargeLanguageModel for DDGChat {
    type Error = Error;

    fn send_message(&mut self, message: &str) -> Result<(), Error> {
        self.messages.push(DDGMessage {
            role: MessageRole::User,
            text: message.to_string(),
        });

        todo!()
    }
}
