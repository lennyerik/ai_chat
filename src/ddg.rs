use std::io::BufReader;

use super::llm;

const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/126.0.0.0 Safari/537.36";

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("network request error")]
    RequestError(#[from] Box<ureq::Error>),

    #[error("no new Vqd received with last request")]
    NoVqdReceived,
}

#[derive(serde::Serialize, Debug, Clone, Copy)]
pub enum DDGChatModel {
    #[serde(rename = "gpt-4o-mini")]
    GPT4oMini,

    #[serde(rename = "claude-3-haiku-20240307")]
    Claude3Haiku,

    #[serde(rename = "meta-llama/Meta-Llama-3.1-70B-Instruct-Turbo")]
    Llama370B,

    #[serde(rename = "mistralai/Mixtral-8x7B-Instruct-v0.1")]
    Mixtral8x7B,
}

#[derive(serde::Serialize, Debug, Clone, Copy)]
pub enum MessageRole {
    #[serde(rename = "user")]
    User,

    #[serde(rename = "asssistant")]
    Assistant,
}

#[derive(serde::Serialize, Debug, Clone)]
pub struct DDGMessage {
    role: MessageRole,
    content: String,
}

#[derive(Debug)]
pub struct DDGChat {
    model: DDGChatModel,
    messages: Vec<DDGMessage>,
    current_vqd: String,
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
        })
    }
}

impl<'c> llm::LargeLanguageModel<'c> for DDGChat {
    type Error = Error;
    type Response = DDGResponse<'c>;

    fn send_message(&'c mut self, message: &str) -> Result<Self::Response, Self::Error> {
        self.messages.push(DDGMessage {
            role: MessageRole::User,
            content: message.to_string(),
        });

        let net_resp = ureq::post("https://duckduckgo.com/duckchat/v1/chat")
            .set("User-Agent", USER_AGENT)
            .set("X-Vqd-4", &self.current_vqd)
            .send_json(ureq::json!({
                "model": self.model,
                "messages": self.messages,
            }))
            .map_err(Box::new)?;

        self.current_vqd = net_resp
            .header("X-Vqd-4")
            .unwrap_or(&self.current_vqd)
            .to_string();

        Ok(DDGResponse {
            chat: self,
            content: String::new(),
            reader: BufReader::new(net_resp.into_reader()),
        })
    }
}

pub struct DDGResponse<'c> {
    chat: &'c mut DDGChat,
    content: String,
    reader: BufReader<Box<dyn std::io::Read + Send + Sync + 'static>>,
}

impl Iterator for DDGResponse<'_> {
    type Item = Result<String, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}
