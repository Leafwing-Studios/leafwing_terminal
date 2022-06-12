use bevy::prelude::*;

use crate::{TerminalCommandEntered, TerminalConfiguration, TerminalState};
use bevy_egui::egui::epaint::text::cursor::CCursor;
use bevy_egui::{
    egui::{
        self, text_edit::CCursorRange, Color32, Context, Frame, Id, RichText, ScrollArea, TextEdit,
    },
    EguiContext,
};
use leafwing_terminal_parser::{parse_terminal_command, ValueRawOwned};

pub(crate) fn terminal_ui(
    mut egui_context: ResMut<EguiContext>,
    config: Res<TerminalConfiguration>,
    mut state: ResMut<TerminalState>,
    mut command_entered: EventWriter<TerminalCommandEntered>,
) {
    const INPUT_HEIGHT: f32 = 30.;
    const MARGIN: f32 = 10.;

    egui::Window::new("Terminal")
        .collapsible(false)
        .fixed_pos([config.left_pos, config.top_pos])
        .fixed_size([config.width - 2. * MARGIN, config.height - 2. * MARGIN])
        .title_bar(false)
        .frame(Frame {
            fill: Color32::BLACK,
            ..Default::default()
        })
        .show(egui_context.ctx_mut(), |ui| {
            ui.vertical(|ui| {
                let scroll_height = ui.available_height() - INPUT_HEIGHT;

                // Scroll area
                ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .stick_to_bottom()
                    .max_height(scroll_height)
                    .show(ui, |ui| {
                        ui.vertical(|ui| {
                            for line in &state.scrollback {
                                ui.label(RichText::new(line).monospace());
                            }
                        });
                    });

                // Separator
                ui.separator();

                // Input
                let text_edit = TextEdit::singleline(&mut state.buf)
                    .desired_width(f32::INFINITY)
                    .lock_focus(true)
                    .font(egui::TextStyle::Monospace);

                // Handle enter
                let text_edit_response = ui.add(text_edit);
                if text_edit_response.lost_focus() && ui.input().key_pressed(egui::Key::Enter) {
                    if state.buf.trim().is_empty() {
                        state.scrollback.push(String::new());
                    } else {
                        let msg = format!("$ {}", state.buf);
                        state.scrollback.push(msg);
                        let cmd_string = state.buf.clone();
                        state.history.insert(1, cmd_string);
                        if state.history.len() > config.history_size + 1 {
                            state.history.pop_back();
                        }

                        match parse_terminal_command(&state.buf) {
                            Ok(cmd) => {
                                let command = TerminalCommandEntered {
                                    command: cmd.command.to_string(),
                                    args: cmd.args.into_iter().map(ValueRawOwned::from).collect(),
                                };

                                command_entered.send(command);
                            }
                            Err(_) => {
                                state
                                    .scrollback
                                    .push("[error] invalid argument(s)".to_string());
                            }
                        }

                        state.buf.clear();
                    }
                }

                // Handle up and down through history
                if text_edit_response.has_focus()
                    && ui.input().key_pressed(egui::Key::ArrowUp)
                    && state.history.len() > 1
                    && state.history_index < state.history.len() - 1
                {
                    if state.history_index == 0 && !state.buf.trim().is_empty() {
                        *state.history.get_mut(0).unwrap() = state.buf.clone();
                    }

                    state.history_index += 1;
                    let previous_item = state.history.get(state.history_index).unwrap().clone();
                    state.buf = previous_item;

                    set_cursor_pos(ui.ctx(), text_edit_response.id, state.buf.len());
                } else if text_edit_response.has_focus()
                    && ui.input().key_pressed(egui::Key::ArrowDown)
                    && state.history_index > 0
                {
                    state.history_index -= 1;
                    let next_item = state.history.get(state.history_index).unwrap().clone();
                    state.buf = next_item;

                    set_cursor_pos(ui.ctx(), text_edit_response.id, state.buf.len());
                }

                // Focus on input
                ui.memory().request_focus(text_edit_response.id);
            });
        });
}

fn set_cursor_pos(ctx: &Context, id: Id, pos: usize) {
    if let Some(mut state) = TextEdit::load_state(ctx, id) {
        state.set_ccursor_range(Some(CCursorRange::one(CCursor::new(pos))));
        state.store(ctx, id);
    }
}
