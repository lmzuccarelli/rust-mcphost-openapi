use colored::Colorize;
use custom_logger as log;
use std::str::FromStr;
use std::sync::Arc;

use crate::{
    chat::ChatSession,
    client::OpenAIClient,
    config::Config,
    tool::{Tool, ToolSet, get_mcp_tools},
};
use anyhow::Result;

mod chat;
mod client;
mod config;
mod error;
mod model;
mod tool;

//default config path
const DEFAULT_CONFIG_PATH: &str = "./config.toml";

#[tokio::main]
async fn main() -> Result<()> {
    // load config
    let config = Config::load(DEFAULT_CONFIG_PATH).await?;
    let res_log_level = log::LevelFilter::from_str(&config.log_level);

    // setup logging
    log::Logging::new()
        .with_level(res_log_level.unwrap())
        .init()
        .expect("should initialize");

    // create openai client
    let api_key = config
        .openai_key
        .clone()
        .unwrap_or_else(|| std::env::var("OPENAI_API_KEY").expect("need set api key"));

    let url = config.chat_url.clone();
    let model = config.model_name.clone();
    log::info!("url   : {:?}", url.as_ref().unwrap());
    log::info!("model : {:?}", model.unwrap());
    let openai_client = Arc::new(OpenAIClient::new(api_key, url, config.proxy));

    // create tool set
    let mut tool_set = ToolSet::default();

    // load mcp
    if config.mcp.is_some() {
        let mcp_clients = config.create_mcp_clients().await?;

        for (name, client) in mcp_clients {
            log::info!(
                "{}",
                format!("loading mcp tools: {}", name).bright_blue().bold()
            );
            let server = client.peer().clone();
            let tools = get_mcp_tools(server).await?;

            for tool in tools {
                let t = format!("  tool: {:<14} : ", tool.name())
                    .bright_white()
                    .bold();
                let description = format!("description: {}", tool.description())
                    .bright_magenta()
                    .bold();
                let parameters =
                    format!("{:<22} :  paramaters : {}", "", tool.parameters()).magenta();
                log::info!("{} {}", t, description);
                log::debug!("{}", parameters);
                tool_set.add_tool(tool);
            }
        }
        println!("");
        log::info!(
            "To use a tool use the following format: Tool: <name> Inputs: <args> (both fields are mandatory)"
        );
        log::info!("If there are no args keep Inputs: (with no args)\n");
    }

    // create chat session
    let mut session = ChatSession::new(
        openai_client,
        tool_set,
        config
            .model_name
            .unwrap_or_else(|| "gpt-4o-mini".to_string()),
    );

    // build system prompt with tool info
    let mut system_prompt =
        "you are a assistant, you can help user to complete various tasks. you have the following tools to use:\n".to_string();

    // add tool info to system prompt
    for tool in session.get_tools() {
        system_prompt.push_str(&format!(
            "\ntool name: {}\ndescription: {}\nparameters: {}\n",
            tool.name(),
            tool.description(),
            serde_json::to_string_pretty(&tool.parameters()).unwrap_or_default()
        ));
    }

    // add tool call format guidance
    system_prompt.push_str(
        "\nif you need to call tool, please use the following format:\n\
        Tool: <tool name>\n\
        Inputs: <inputs>\n",
    );

    // add system prompt
    session.add_system_prompt(system_prompt);

    // start chat
    session.chat().await?;

    Ok(())
}
