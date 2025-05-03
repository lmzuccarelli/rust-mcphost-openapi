use std::{
    io::{self, Write},
    sync::Arc,
};

use anyhow::Result;

use crate::{
    client::ChatClient,
    model::{CompletionRequest, Message},
    tool::{Tool as ToolTrait, ToolSet},
};

pub struct ChatSession {
    client: Arc<dyn ChatClient>,
    tool_set: ToolSet,
    model: String,
    messages: Vec<Message>,
}

impl ChatSession {
    pub fn new(client: Arc<dyn ChatClient>, tool_set: ToolSet, model: String) -> Self {
        Self {
            client,
            tool_set,
            model,
            messages: Vec::new(),
        }
    }

    pub fn add_system_prompt(&mut self, prompt: impl ToString) {
        self.messages.push(Message::system(prompt));
    }

    pub fn get_tools(&self) -> Vec<Arc<dyn ToolTrait>> {
        self.tool_set.tools()
    }

    pub async fn chat(&mut self) -> Result<()> {
        println!("welcome to use simple chat client, use 'exit' to quit");

        loop {
            print!("> ");
            io::stdout().flush()?;

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            input = input.trim().to_string();

            if input.is_empty() {
                continue;
            }

            if input == "exit" {
                break;
            }

            // prepare tool list
            //let tools = self.tool_set.tools();
            //let tool_definitions = if !tools.is_empty() {
            //    Some(
            //        tools
            //            .iter()
            //            .map(|tool| crate::model::Tool {
            //                name: tool.name(),
            //                description: tool.description(),
            //                parameters: tool.parameters(),
            //            })
            //            .collect(),
            //   )
            //} else {
            //    None
            //};

            // read tool names
            //let tool_names = tools
            //    .iter()
            //    .map(|tool| tool.name())
            //    .collect::<Vec<String>>();

            //let index = tool_names.iter().position(|x| input.contains(x));

            if input.contains("Tool: ") && input.contains("Inputs: ") {
                let (tool, args) = input.split_once("Inputs: ").unwrap();
                let tool_name = tool.split("Tool: ").nth(1).unwrap().trim();
                let params: Vec<&str> = args.split(" ").collect();
                let mut args_text = Vec::new();

                for param in params {
                    args_text.push(param);
                }

                //let idx = index.unwrap();
                //println!(
                //    "I have detected a tool call and will be executing it ... {}",
                //    tool_names[idx]
                //);
                //let tool_name = tool_names[idx].clone();

                println!("calling tool:{}:", tool_name);
                if let Some(tool) = self.tool_set.get_tool(&tool_name) {
                    // simple handle args
                    let args_str = args_text.join(" ");
                    let args = match serde_json::from_str(&args_str) {
                        Ok(v) => v,
                        Err(_) => {
                            // try to handle args as string
                            serde_json::Value::String(args_str)
                        }
                    };

                    // call tool
                    match tool.call(args).await {
                        Ok(result) => {
                            println!("tool result: {}", result);

                            // add tool result to dialog
                            //self.messages.push(Message::user(result));
                        }
                        Err(e) => {
                            println!("tool call failed: {}", e);
                            //self.messages
                            //    .push(Message::user(format!("tool call failed: {}", e)));
                        }
                    }
                } else {
                    println!("tool not found: {}", tool_name);
                }
            } else {
                self.messages.push(Message::user(&input));

                // create request
                let request = CompletionRequest {
                    model: self.model.clone(),
                    messages: self.messages.clone(),
                    temperature: Some(0.7),
                    tools: None,
                };

                println!("I'm thinking ...");

                // send request
                let response = self.client.complete(request).await?;
                if let Some(choice) = response.choices.first() {
                    println!("AI: {}", choice.message.content);
                    self.messages.push(choice.message.clone());
                }
            }
        }

        Ok(())
    }
}
