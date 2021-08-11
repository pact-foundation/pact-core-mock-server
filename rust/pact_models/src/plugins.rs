//! Pact Plugin Support

use pact_plugin_driver::plugin_manager::load_plugin;
use pact_plugin_driver::plugin_models::PluginDependency;

/// Load all the plugins specified in the Pact metadata
pub async fn load_plugins<'a>(plugins: &Vec<PluginDependency>) -> anyhow::Result<()> {
  for plugin_details in plugins {
    load_plugin(plugin_details).await?;
  }
  Ok(())
}
