use std::io::Write;

use llm::LargeLanguageModel;

mod ddg;
mod llm;

fn main() -> Result<(), ddg::Error> {
    let mut chat = ddg::DDGChat::new(ddg::DDGChatModel::GPT4oMini)?;
    let resp = chat.send_message("hi!")?;

    let mut stdout = std::io::stdout().lock();
    for response_chunk in resp {
        let _ = write!(stdout, "{response_chunk}");
        let _ = stdout.flush();
    }
    println!();

    // resp.result()?;

    Ok(())
}
