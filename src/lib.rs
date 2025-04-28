// src/lib.rs
use zed_extension_api as zed;
use zed::{ContextServerId, Project, Command, Result};

struct SopsExtension;

impl zed::Extension for SopsExtension {
    fn new() -> Self {
        SopsExtension
    }

    fn context_server_command(
        &mut self,
        context_server_id: &ContextServerId,
        _project: &Project,
    ) -> Result<Command> {
        // Only respond to our specific context server ID
        if context_server_id.as_ref() != "sops" {
            return Err("unknown context server".into());
        }

        Ok(Command {
            command: "sops_context_server".to_string(),
            args: Vec::new(),
            env: Vec::new(),
        })
    }
}

zed::register_extension!(SopsExtension);
