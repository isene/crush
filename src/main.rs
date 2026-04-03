use crust::{Crust, Pane, Input, Popup};
use crust::style;
use serde_json::Value;
use std::collections::HashMap;
use std::io::Write;

const THEMES: &[&str] = &["default", "solarized", "dracula", "gruvbox", "nord", "monokai"];

struct App {
    top: Pane,
    left: Pane,
    right: Pane,
    status: Pane,
    cols: u16,
    rows: u16,
    config: Value,
    config_path: String,
    categories: Vec<Category>,
    cat_index: usize,
    item_index: usize,
    dirty: bool,
}

struct Category {
    name: String,
    items: Vec<Item>,
}

#[derive(Clone)]
struct Item {
    key: String,
    label: String,
    kind: ItemKind,
}

#[derive(Clone)]
enum ItemKind {
    Color(u8),
    Bool(bool),
    Number(u64),
    Text(String),
    Theme(String),
    Choice(Vec<String>, String),
}

fn main() {
    let home = std::env::var("HOME").unwrap_or_default();
    let config_path = format!("{}/.rushrc.json", home);
    let config: Value = std::fs::read_to_string(&config_path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_else(|| serde_json::json!({}));

    Crust::init();
    let (cols, rows) = Crust::terminal_size();

    let categories = build_categories(&config);
    let split = 25u16;
    // Gap between panes: left border at split, right border at split+2
    let lw = split - 1;
    let rx = split + 3;
    let rw = cols.saturating_sub(rx).saturating_sub(1);

    let mut app = App {
        top: Pane::new(1, 1, cols, 1, 0, 236),
        left: Pane::new(2, 3, lw, rows - 4, 255, 0),
        right: Pane::new(rx, 3, rw, rows - 4, 252, 0),
        status: Pane::new(1, rows, cols, 1, 252, 236),
        cols,
        rows,
        config,
        config_path,
        categories,
        cat_index: 0,
        item_index: 0,
        dirty: false,
    };

    app.left.border = true;
    app.right.border = true;
    app.left.border_refresh();
    app.right.border_refresh();
    app.render();

    loop {
        let Some(key) = Input::getchr(None) else { continue };
        match key.as_str() {
            "q" | "ESC" => {
                if app.dirty {
                    app.status.say(&style::fg(" Save changes? (y/n)", 220));
                    if let Some(k) = Input::getchr(None) {
                        if k == "y" || k == "Y" {
                            app.save();
                        }
                    }
                }
                break;
            }
            "j" | "DOWN" => { app.move_down(); app.render(); }
            "k" | "UP" => { app.move_up(); app.render(); }
            "J" | "PgDOWN" => { app.next_category(); app.render(); }
            "K" | "PgUP" => { app.prev_category(); app.render(); }
            "l" | "RIGHT" | "TAB" => { app.next_value(); app.render(); }
            "h" | "LEFT" | "S-TAB" => { app.prev_value(); app.render(); }
            "ENTER" => { app.edit_value(); app.render(); }
            "W" => { app.save(); }
            "RESIZE" => {
                let (cols, rows) = Crust::terminal_size();
                app.cols = cols;
                app.rows = rows;
                let split = 25u16;
                let lw = split - 1;
                let rx = split + 3;
                let rw = cols.saturating_sub(rx).saturating_sub(1);
                app.top = Pane::new(1, 1, cols, 1, 0, 236);
                app.left = Pane::new(2, 3, lw, rows - 4, 255, 0);
                app.right = Pane::new(rx, 3, rw, rows - 4, 252, 0);
                app.status = Pane::new(1, rows, cols, 1, 252, 236);
                app.left.border = true;
                app.right.border = true;
                Crust::clear_screen();
                app.left.border_refresh();
                app.right.border_refresh();
                app.render();
            }
            _ => {}
        }
    }

    Crust::cleanup();
}

impl App {
    fn render(&mut self) {
        // Top bar
        let dirty_mark = if self.dirty { " [modified]" } else { "" };
        self.top.say(&format!(" crush - Rush Configuration{}", dirty_mark));

        // Left: category list
        let mut lines = Vec::new();
        for (i, cat) in self.categories.iter().enumerate() {
            if i == self.cat_index {
                lines.push(style::reverse(&format!(" {} ", cat.name)));
            } else {
                lines.push(format!(" {} ", cat.name));
            }
        }
        self.left.set_text(&lines.join("\n"));
        self.left.ix = 0;
        self.left.full_refresh();

        // Right: items for selected category
        self.render_items();

        // Status
        self.status.say(&format!(" {}/{} | j/k:item  J/K:category  h/l:change  Enter:edit  W:save  q:quit",
            self.item_index + 1,
            self.categories.get(self.cat_index).map(|c| c.items.len()).unwrap_or(0)));
    }

    fn render_items(&mut self) {
        let Some(cat) = self.categories.get(self.cat_index) else { return };
        let mut lines = Vec::new();

        lines.push(style::fg(&style::bold(&cat.name), 81));
        lines.push(style::fg(&"\u{2500}".repeat(40), 245));
        lines.push(String::new());

        for (i, item) in cat.items.iter().enumerate() {
            let selected = i == self.item_index;
            let label = format!("{:<18}", item.label);
            let value_str = match &item.kind {
                ItemKind::Color(c) => {
                    let swatch = style::fg("\u{2588}\u{2588}\u{2588}", *c);
                    format!("{} {:>3}", swatch, c)
                }
                ItemKind::Bool(b) => {
                    if *b { style::fg("YES", 82) } else { style::fg("NO", 196) }
                }
                ItemKind::Number(n) => format!("{}", n),
                ItemKind::Text(t) => {
                    if t.len() > 30 { format!("{}...", &t[..27]) } else { t.clone() }
                }
                ItemKind::Theme(t) => style::fg(t, 220),
                ItemKind::Choice(_, current) => style::fg(current, 81),
            };

            let arrow_l = if selected { "\u{25C0} " } else { "  " };
            let arrow_r = if selected { " \u{25B6}" } else { "  " };
            let line = format!("  {}{}{}{}",
                if selected { style::underline(&label) } else { label },
                arrow_l, value_str, arrow_r);
            lines.push(line);
        }

        // Color palette preview for color items
        if let Some(item) = cat.items.get(self.item_index) {
            if let ItemKind::Color(_) = &item.kind {
                lines.push(String::new());
                lines.push(style::fg("Color palette:", 245));
                // Show 16 rows of 16 colors
                for row in 0..16u8 {
                    let mut palette_line = String::from("  ");
                    for col in 0..16u8 {
                        let c = row * 16 + col;
                        palette_line.push_str(&style::fg("\u{2588}", c));
                    }
                    lines.push(palette_line);
                }
            }
        }

        // Theme preview
        if let Some(item) = cat.items.get(self.item_index) {
            if let ItemKind::Theme(t) = &item.kind {
                lines.push(String::new());
                lines.push(style::fg(&format!("Theme: {}", t), 220));
                lines.push(String::new());
                lines.push(format!("  {} {} {}",
                    style::fg("user", get_theme_color(t, "c_user")),
                    style::fg("@", 245),
                    style::fg("host", get_theme_color(t, "c_host"))));
                lines.push(format!("  {} {}",
                    style::fg("~/projects", get_theme_color(t, "c_cwd")),
                    style::fg("(main)", get_theme_color(t, "c_git"))));
                lines.push(format!("  {} {}",
                    style::fg(">", get_theme_color(t, "c_prompt")),
                    style::fg("ls -la | grep foo", get_theme_color(t, "c_cmd"))));
            }
        }

        self.right.set_text(&lines.join("\n"));
        self.right.ix = 0;
        self.right.full_refresh();
    }

    fn move_down(&mut self) {
        let Some(cat) = self.categories.get(self.cat_index) else { return };
        if self.item_index < cat.items.len().saturating_sub(1) {
            self.item_index += 1;
        }
    }

    fn move_up(&mut self) {
        if self.item_index > 0 {
            self.item_index -= 1;
        }
    }

    fn next_category(&mut self) {
        if self.cat_index < self.categories.len() - 1 {
            self.cat_index += 1;
            self.item_index = 0;
        }
    }

    fn prev_category(&mut self) {
        if self.cat_index > 0 {
            self.cat_index -= 1;
            self.item_index = 0;
        }
    }

    fn next_value(&mut self) {
        let Some(cat) = self.categories.get_mut(self.cat_index) else { return };
        let Some(item) = cat.items.get_mut(self.item_index) else { return };
        match &mut item.kind {
            ItemKind::Color(c) => { *c = c.wrapping_add(1); self.dirty = true; }
            ItemKind::Bool(b) => { *b = !*b; self.dirty = true; }
            ItemKind::Number(n) => { *n += 1; self.dirty = true; }
            ItemKind::Theme(t) => {
                let idx = THEMES.iter().position(|&th| th == t.as_str()).unwrap_or(0);
                *t = THEMES[(idx + 1) % THEMES.len()].to_string();
                self.dirty = true;
            }
            ItemKind::Choice(opts, current) => {
                let idx = opts.iter().position(|o| o == current).unwrap_or(0);
                *current = opts[(idx + 1) % opts.len()].clone();
                self.dirty = true;
            }
            _ => {}
        }
        self.apply_to_config();
    }

    fn prev_value(&mut self) {
        let Some(cat) = self.categories.get_mut(self.cat_index) else { return };
        let Some(item) = cat.items.get_mut(self.item_index) else { return };
        match &mut item.kind {
            ItemKind::Color(c) => { *c = c.wrapping_sub(1); self.dirty = true; }
            ItemKind::Bool(b) => { *b = !*b; self.dirty = true; }
            ItemKind::Number(n) => { *n = n.saturating_sub(1); self.dirty = true; }
            ItemKind::Theme(t) => {
                let idx = THEMES.iter().position(|&th| th == t.as_str()).unwrap_or(0);
                *t = THEMES[(idx + THEMES.len() - 1) % THEMES.len()].to_string();
                self.dirty = true;
            }
            ItemKind::Choice(opts, current) => {
                let idx = opts.iter().position(|o| o == current).unwrap_or(0);
                *current = opts[(idx + opts.len() - 1) % opts.len()].clone();
                self.dirty = true;
            }
            _ => {}
        }
        self.apply_to_config();
    }

    fn edit_value(&mut self) {
        let Some(cat) = self.categories.get_mut(self.cat_index) else { return };
        let Some(item) = cat.items.get_mut(self.item_index) else { return };
        match &mut item.kind {
            ItemKind::Text(t) => {
                let orig_bg = self.status.bg;
                self.status.bg = 18;
                let new_val = self.status.ask(&format!("{}: ", item.label), t);
                self.status.bg = orig_bg;
                if !new_val.is_empty() {
                    *t = new_val;
                    self.dirty = true;
                }
            }
            ItemKind::Number(n) => {
                let orig_bg = self.status.bg;
                self.status.bg = 18;
                let new_val = self.status.ask(&format!("{}: ", item.label), &n.to_string());
                self.status.bg = orig_bg;
                if let Ok(v) = new_val.parse() {
                    *n = v;
                    self.dirty = true;
                }
            }
            ItemKind::Color(c) => {
                let orig_bg = self.status.bg;
                self.status.bg = 18;
                let new_val = self.status.ask(&format!("{} (0-255): ", item.label), &c.to_string());
                self.status.bg = orig_bg;
                if let Ok(v) = new_val.parse::<u8>() {
                    *c = v;
                    self.dirty = true;
                }
            }
            _ => { self.next_value(); return; }
        }
        self.apply_to_config();
    }

    fn apply_to_config(&mut self) {
        for cat in &self.categories {
            for item in &cat.items {
                let val: Value = match &item.kind {
                    ItemKind::Color(c) => Value::from(*c),
                    ItemKind::Bool(b) => Value::from(*b),
                    ItemKind::Number(n) => Value::from(*n),
                    ItemKind::Text(t) => Value::from(t.clone()),
                    ItemKind::Theme(t) => Value::from(t.clone()),
                    ItemKind::Choice(_, current) => Value::from(current.clone()),
                };
                self.config[&item.key] = val;
            }
        }
    }

    fn save(&mut self) {
        self.apply_to_config();
        if let Ok(json) = serde_json::to_string_pretty(&self.config) {
            if std::fs::write(&self.config_path, &json).is_ok() {
                self.dirty = false;
                // Send SIGUSR1 to parent rush process to trigger hot-reload
                signal_parent_rush();
                self.status.say(&style::fg(" Config saved (rush reloaded)", 82));
            } else {
                self.status.say(&style::fg(" Failed to save config!", 196));
            }
        }
    }
}

/// Send SIGUSR1 to the parent process (rush) to trigger config reload
fn signal_parent_rush() {
    unsafe {
        let ppid = libc::getppid();
        if ppid > 1 {
            libc::kill(ppid, libc::SIGUSR1);
        }
    }
}

fn build_categories(config: &Value) -> Vec<Category> {
    vec![
        Category {
            name: "Theme".into(),
            items: vec![
                item("theme", "Theme", ItemKind::Theme(
                    config["theme"].as_str().unwrap_or("default").into())),
            ],
        },
        Category {
            name: "Prompt Colors".into(),
            items: vec![
                color_item(config, "c_user", "Username", 2),
                color_item(config, "c_host", "Hostname", 2),
                color_item(config, "c_cwd", "Directory", 81),
                color_item(config, "c_git", "Git branch", 243),
                color_item(config, "c_stamp", "Timestamp", 240),
                color_item(config, "c_prompt", "Prompt >", 208),
                color_item(config, "c_user_root", "Root user", 196),
                color_item(config, "c_host_root", "Root host", 196),
            ],
        },
        Category {
            name: "Syntax Colors".into(),
            items: vec![
                color_item(config, "c_cmd", "Commands", 48),
                color_item(config, "c_nick", "Nicks", 87),
                color_item(config, "c_gnick", "Global nicks", 87),
                color_item(config, "c_path", "Paths", 7),
                color_item(config, "c_switch", "Switches", 220),
                color_item(config, "c_bookmark", "Bookmarks", 51),
                color_item(config, "c_colon", "Colon cmds", 33),
                color_item(config, "c_suggestion", "Suggestions", 240),
            ],
        },
        Category {
            name: "Tab Colors".into(),
            items: vec![
                color_item(config, "c_tabselect", "Selected tab", 214),
                color_item(config, "c_taboption", "Tab options", 244),
                color_item(config, "c_dir", "Directories", 12),
                color_item(config, "c_exec", "Executables", 9),
                color_item(config, "c_file", "Files", 7),
            ],
        },
        Category {
            name: "Completion".into(),
            items: vec![
                bool_item(config, "completion_fuzzy", "Fuzzy match", false),
                bool_item(config, "completion_case_sensitive", "Case sensitive", false),
                num_item(config, "completion_limit", "Max results", 10),
                bool_item(config, "completion_show_metadata", "Show metadata", false),
            ],
        },
        Category {
            name: "Behavior".into(),
            items: vec![
                bool_item(config, "auto_correct", "Auto correct", false),
                bool_item(config, "auto_pair", "Auto pair", true),
                bool_item(config, "show_tips", "Show tips", true),
                bool_item(config, "show_cmd", "Show command", true),
                bool_item(config, "rprompt", "Right prompt", true),
                num_item(config, "slow_command_threshold", "Slow cmd warn (s)", 0),
                num_item(config, "session_autosave", "Autosave (s)", 0),
                item("history_dedup", "History dedup", ItemKind::Choice(
                    vec!["off".into(), "full".into(), "smart".into()],
                    config["history_dedup"].as_str().unwrap_or("smart").into())),
            ],
        },
        Category {
            name: "Paths".into(),
            items: vec![
                text_item(config, "file_manager", "File manager", "rtfm"),
            ],
        },
    ]
}

fn item(key: &str, label: &str, kind: ItemKind) -> Item {
    Item { key: key.into(), label: label.into(), kind }
}

fn color_item(config: &Value, key: &str, label: &str, default: u8) -> Item {
    let val = config[key].as_u64().unwrap_or(default as u64) as u8;
    Item { key: key.into(), label: label.into(), kind: ItemKind::Color(val) }
}

fn bool_item(config: &Value, key: &str, label: &str, default: bool) -> Item {
    let val = config[key].as_bool().unwrap_or(default);
    Item { key: key.into(), label: label.into(), kind: ItemKind::Bool(val) }
}

fn num_item(config: &Value, key: &str, label: &str, default: u64) -> Item {
    let val = config[key].as_u64().unwrap_or(default);
    Item { key: key.into(), label: label.into(), kind: ItemKind::Number(val) }
}

fn text_item(config: &Value, key: &str, label: &str, default: &str) -> Item {
    let val = config[key].as_str().unwrap_or(default).to_string();
    Item { key: key.into(), label: label.into(), kind: ItemKind::Text(val) }
}

fn get_theme_color(theme: &str, key: &str) -> u8 {
    match theme {
        "solarized" => match key {
            "c_user" => 136, "c_host" => 166, "c_cwd" => 37, "c_git" => 125, "c_prompt" => 136,
            "c_cmd" => 64, "c_nick" => 33, _ => 245,
        },
        "dracula" => match key {
            "c_user" => 117, "c_host" => 212, "c_cwd" => 84, "c_git" => 141, "c_prompt" => 141,
            "c_cmd" => 84, "c_nick" => 117, _ => 248,
        },
        "gruvbox" => match key {
            "c_user" => 214, "c_host" => 142, "c_cwd" => 109, "c_git" => 175, "c_prompt" => 214,
            "c_cmd" => 142, "c_nick" => 108, _ => 245,
        },
        "nord" => match key {
            "c_user" => 110, "c_host" => 150, "c_cwd" => 110, "c_git" => 139, "c_prompt" => 110,
            "c_cmd" => 150, "c_nick" => 146, _ => 246,
        },
        "monokai" => match key {
            "c_user" => 148, "c_host" => 81, "c_cwd" => 186, "c_git" => 141, "c_prompt" => 197,
            "c_cmd" => 148, "c_nick" => 81, _ => 246,
        },
        _ => match key { // default
            "c_user" => 2, "c_host" => 2, "c_cwd" => 81, "c_git" => 243, "c_prompt" => 208,
            "c_cmd" => 48, "c_nick" => 87, _ => 240,
        },
    }
}
