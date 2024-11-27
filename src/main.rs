use fallible_iterator::FallibleIterator;
use std::io::{IsTerminal, Write};

use llm::LargeLanguageModel;

mod config;
mod ddg;
mod llm;

fn run_model_and_print_response<'m, M: llm::LargeLanguageModel<'m>>(
    output_writer: &mut impl Write,
    model: &'m mut M,
    prompt: &str,
) -> Result<(), <M as LargeLanguageModel<'m>>::Error> {
    let mut response = model.send_message(prompt)?;

    while let Some(response_chunk) = response.next()? {
        let _ = write!(output_writer, "{response_chunk}");
        let _ = output_writer.flush();
    }
    println!();

    Ok(())
}

fn exit_with_error_msg(err_msg: &str) -> ! {
    eprintln!("{err_msg}");
    std::process::exit(1)
}

macro_rules! exit_with_error {
    () => {
        |e| exit_with_error_msg(&format!("{e}"))
    };
}

fn main() {
    let config = match config::Config::read_from_disk() {
        Ok(Some(config)) => config,
        Err(e) => exit_with_error_msg(&format!("Error loading configuration file: {e}")),
        Ok(None) => {
            if let Ok(paths) = config::Config::get_config_paths() {
                if let Some(path) = paths.first() {
                    println!("Writing default configuration file to {}\n", path.display());
                }
            }

            config::Config::write_default().unwrap_or_else(|e| {
                exit_with_error_msg(&format!("Error writing default configuration file: {e}"))
            })
        }
    };

    let prompt: String = std::env::args().skip(1).collect::<Vec<_>>().join(" ");
    let prompt = prompt.trim();

    if prompt.is_empty() {
        exit_with_error_msg(&format!(
            "No message supplied.\nUsage: {} <MESSAGE>",
            std::env::args().next().unwrap_or_else(|| "./ai".to_owned())
        ));
    }

    let mut model = ddg::DDGChat::new(config.ddg_chat_model.unwrap_or_default())
        .unwrap_or_else(exit_with_error!());

    let mut stdout = std::io::stdout();
    run_model_and_print_response(&mut stdout, &mut model, prompt)
        .unwrap_or_else(exit_with_error!());

    let stdin = std::io::stdin();
    if !stdin.is_terminal() {
        return;
    }

    loop {
        let _ = write!(stdout, "> ");
        let _ = stdout.flush();

        let mut input = String::new();

        if stdin.read_line(&mut input).is_err() || input.is_empty() {
            break;
        }

        run_model_and_print_response(&mut stdout, &mut model, input.trim())
            .unwrap_or_else(exit_with_error!());
    }
}
