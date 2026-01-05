//! Zellij plugin implementation for Faster
//!
//! Displays task queue and allows command input directly in Zellij

use zellij_tile::prelude::*;
use std::collections::BTreeMap;

#[derive(Default)]
struct State {
    tasks: Vec<String>,
    input_mode: bool,
    current_input: String,
    selected_index: usize,
}

register_plugin!(State);

impl ZellijPlugin for State {
    fn load(&mut self, _configuration: BTreeMap<String, String>) {
        subscribe(&[
            EventType::Key,
            EventType::Timer,
        ]);

        // Poll for updates every second
        set_timeout(1.0);

        // Load initial tasks
        self.refresh_tasks();
    }

    fn update(&mut self, event: Event) -> bool {
        match event {
            Event::Key(key) => {
                self.handle_key(key)
            }
            Event::Timer(_) => {
                self.refresh_tasks();
                set_timeout(1.0); // Continue polling
                true
            }
            _ => false,
        }
    }

    fn render(&mut self, rows: usize, cols: usize) {
        let mut text = String::new();

        // Header
        text.push_str(&format!("┌─ Faster Queue {}┐\n", "─".repeat(cols - 18)));

        // Tasks
        if self.tasks.is_empty() {
            text.push_str("│ No tasks in queue                             │\n");
        } else {
            for (i, task) in self.tasks.iter().enumerate() {
                let marker = if i == self.selected_index { "→" } else { " " };
                let task_display = if task.len() > cols - 10 {
                    format!("{}...", &task[..cols - 13])
                } else {
                    task.clone()
                };
                text.push_str(&format!("│ {} {}{}│\n",
                    marker,
                    task_display,
                    " ".repeat(cols - task_display.len() - 6)
                ));
            }
        }

        // Input area
        let padding = rows.saturating_sub(self.tasks.len() + 4);
        for _ in 0..padding {
            text.push_str(&format!("│{}│\n", " ".repeat(cols - 2)));
        }

        // Command input
        if self.input_mode {
            text.push_str(&format!("│ > {}{}│\n",
                self.current_input,
                " ".repeat(cols - self.current_input.len() - 5)
            ));
        } else {
            text.push_str(&format!("│ Press 'i' to add command, 'r' to refresh      │\n"));
        }

        // Footer
        text.push_str(&format!("└{}┘", "─".repeat(cols - 2)));

        print!("{}", text);
    }
}

impl State {
    fn handle_key(&mut self, key: Key) -> bool {
        if self.input_mode {
            match key {
                Key::Char('\n') => {
                    self.submit_command();
                    self.input_mode = false;
                    self.current_input.clear();
                    true
                }
                Key::Esc => {
                    self.input_mode = false;
                    self.current_input.clear();
                    true
                }
                Key::Char(c) => {
                    self.current_input.push(c);
                    true
                }
                Key::Backspace => {
                    self.current_input.pop();
                    true
                }
                _ => false,
            }
        } else {
            match key {
                Key::Char('i') => {
                    self.input_mode = true;
                    true
                }
                Key::Char('r') => {
                    self.refresh_tasks();
                    true
                }
                Key::Char('j') | Key::Down => {
                    if self.selected_index < self.tasks.len().saturating_sub(1) {
                        self.selected_index += 1;
                    }
                    true
                }
                Key::Char('k') | Key::Up => {
                    if self.selected_index > 0 {
                        self.selected_index -= 1;
                    }
                    true
                }
                Key::Char('d') => {
                    self.cancel_selected_task();
                    true
                }
                _ => false,
            }
        }
    }

    fn refresh_tasks(&mut self) {
        // TODO: Read from SQLite database
        // For now, use mock data
        self.tasks = vec![
            "⏳ run tests".to_string(),
            "→ fix auth bug [45%]".to_string(),
            "✓ list files".to_string(),
        ];
    }

    fn submit_command(&mut self) {
        if !self.current_input.is_empty() {
            // TODO: Queue command to SQLite
            eprintln!("Queuing command: {}", self.current_input);
            self.refresh_tasks();
        }
    }

    fn cancel_selected_task(&mut self) {
        if self.selected_index < self.tasks.len() {
            // TODO: Cancel task in SQLite
            eprintln!("Canceling task at index {}", self.selected_index);
            self.refresh_tasks();
        }
    }
}
