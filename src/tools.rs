use std::{
    error::Error,
    fmt::Display,
    fs::File,
    io::{self, Read},
    path::Path,
    str::FromStr,
};

use colored::Colorize;
use serde_json::Value;

// ---------- tool errors ------------

#[derive(Debug)]
pub enum ParseToolError {
    Empty,
    UnknownTool(String),
    MalformedArguments,
}

impl Display for ParseToolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseToolError::Empty => write!(f, "no tool was passed"),
            ParseToolError::UnknownTool(s) => write!(f, "unknown tool: {}", s),
            ParseToolError::MalformedArguments => write!(f, "malformed arguments, not JSON"),
        }
    }
}

impl std::error::Error for ParseToolError {}

#[derive(Debug)]
pub struct ReadToolError {
    source: io::Error,
}

impl Display for ReadToolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "could not read file: {}", self.source)
    }
}

impl From<io::Error> for ReadToolError {
    fn from(source: io::Error) -> Self {
        Self { source }
    }
}

impl std::error::Error for ReadToolError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.source)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ToolError {
    #[error(transparent)]
    Parse(#[from] ParseToolError),
    #[error(transparent)]
    Read(#[from] ReadToolError),
}

// ----------- tool definitions --------

pub enum Tool {
    READ,
}

impl FromStr for Tool {
    type Err = ParseToolError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim().to_lowercase();
        match s.as_str() {
            "read" => Ok(Tool::READ),
            "" => Err(ParseToolError::Empty),
            _ => Err(ParseToolError::UnknownTool(s)),
        }
    }
}

impl Display for Tool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Tool::READ => write!(f, "read"),
        }
    }
}

pub fn read_tool(filepath: &Path) -> Result<(), ReadToolError> {
    let mut file = File::open(&filepath)?;
    let mut file_contents = String::new();
    file.read_to_string(&mut file_contents)?;

    // TODO: For now we just print, we should return the string back to the harness context
    println!("{}", file_contents);

    Ok(())
}

// ----------- tool handling --------

pub fn call_tool(name: &str, args: &str) -> Result<(), ToolError> {
    let tool: Tool = name.parse()?;
    let parsed_args: Value =
        serde_json::from_str(args).map_err(|_| ParseToolError::MalformedArguments)?;

    println!("{} {}", ">> tool call:".purple(), tool.to_string().purple());

    match tool {
        Tool::READ => {
            let file_path = parsed_args
                .get("file_path")
                .and_then(|v| v.as_str())
                .ok_or(ParseToolError::MalformedArguments)?;
            read_tool(Path::new(file_path))?;
        }
    }
    Ok(())
}
