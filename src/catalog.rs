use crate::model::CommandSpec;

pub struct CommandCatalog {
  commands: Vec<CommandSpec>,
}

impl CommandCatalog {
  pub fn new(commands: Vec<CommandSpec>) -> Self {
    Self { commands }
  }

  pub fn resolve(&self, command_id: &str) -> Option<&CommandSpec> {
    self.commands.iter().find(|command| command.id == command_id)
  }

  pub fn all(&self) -> &[CommandSpec] {
    &self.commands
  }
}

pub fn default_command_catalog() -> CommandCatalog {
  CommandCatalog::new(vec![
    CommandSpec {
      id: "debug.captureScreen",
      summary: "Capture one desktop screenshot through the shared runtime path.",
      driver_id: "macos.screenshot",
      operation: "capture_screen",
    },
    CommandSpec {
      id: "debug.fixtureObserve",
      summary: "Emit a deterministic observation result without touching the real UI.",
      driver_id: "fixture.observe",
      operation: "observe_fixture_scene",
    },
  ])
}
