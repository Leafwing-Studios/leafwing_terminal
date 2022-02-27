use bevy::prelude::*;
use leafwing_terminal::{TerminalConfiguration, TerminalPlugin};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(TerminalPlugin)
        .add_system_to_stage(CoreStage::PreUpdate, sync_full_screen)
        .run();
}

fn sync_full_screen(mut config: ResMut<TerminalConfiguration>, windows: Res<Windows>) {
    if windows.is_changed() {
        let window = windows.get_primary().unwrap();

        config.left_pos = 0.0;
        config.top_pos = 0.0;

        config.width = window.width();
        config.height = window.height();
    }
}
