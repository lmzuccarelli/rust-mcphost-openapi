# Overview

A Rust openapi chat AI client using the  model context protocol (MCP SDK).

The AI chat client calls the v1/chat/completions enpoint to interact with a llama.cpp based model.

The client makes use of a simple toml config file for the llama.cpp endpoint and MCPServer endpoints and other relevant paramaters.

On start up the client will query the MCPServer/s for functions, descriptions and parameters

## Usage

Clone this repo

Execute the following commands

```
cd rust-mcphost-openapi

make build
```

The binary will be located at 

```
./target/release/rust-mcphost-openapi
```

Update the config.toml file (in this repo)

Launch the service

```
./target/reelease/rust-mcphost-openapi
```

