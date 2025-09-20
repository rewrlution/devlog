use std::cmp::min;

use super::state::App;

impl App {
    pub fn move_cursor_to_end(&mut self) {
        let lines: Vec<&str> = self.content.split('\n').collect();
        if lines.is_empty() {
            self.cursor_row = 0;
            self.cursor_col = 0;
        } else {
            self.cursor_row = lines.len() - 1;
            // Use character count, not byte length
            self.cursor_col = lines.last().unwrap().chars().count();
        }
    }

    pub fn insert_char(&mut self, ch: char) {
        let mut lines: Vec<String> = self.content.split('\n').map(|s| s.to_string()).collect();
        if lines.is_empty() {
            lines.push(String::new());
        }
        let row = self.cursor_row.min(lines.len() - 1);
        let line = &mut lines[row];
        // Convert to character-based indexing
        let line_chars: Vec<char> = line.chars().collect();
        let col = self.cursor_col.min(line_chars.len());
        
        // Insert character at the correct character position
        let mut new_chars = line_chars;
        new_chars.insert(col, ch);
        *line = new_chars.into_iter().collect();
        
        // Advance cursor by 1 character (not bytes)
        self.cursor_col = col + 1;
        self.content = lines.join("\n");
        self.dirty = true;
    }

    pub fn backspace(&mut self) {
        if self.cursor_row == 0 && self.cursor_col == 0 {
            return;
        }
        let mut lines: Vec<String> = self.content.split('\n').map(|s| s.to_string()).collect();
        if lines.is_empty() {
            return;
        }
        let row = self.cursor_row;
        let col = self.cursor_col;
        if col > 0 {
            let line = &mut lines[row];
            // Convert to character-based indexing
            let mut line_chars: Vec<char> = line.chars().collect();
            if col <= line_chars.len() {
                let char_idx = col - 1;
                line_chars.remove(char_idx);
                *line = line_chars.into_iter().collect();
                self.cursor_col = char_idx;
            }
        } else if row > 0 {
            // Moving to previous line - use character count for cursor position
            let prev_line_chars = lines[row - 1].chars().count();
            let current = lines.remove(row);
            self.cursor_row -= 1;
            self.cursor_col = prev_line_chars;
            lines[self.cursor_row].push_str(&current);
        }
        self.content = lines.join("\n");
        self.dirty = true;
    }

    pub fn delete(&mut self) {
        let mut lines: Vec<String> = self.content.split('\n').map(|s| s.to_string()).collect();
        if lines.is_empty() {
            return;
        }
        let row = self.cursor_row.min(lines.len() - 1);
        let line = &mut lines[row];
        // Convert to character-based indexing
        let mut line_chars: Vec<char> = line.chars().collect();
        let line_char_len = line_chars.len();
        
        if self.cursor_col < line_char_len {
            // Delete character at cursor position
            line_chars.remove(self.cursor_col);
            *line = line_chars.into_iter().collect();
        } else if row + 1 < lines.len() {
            // Delete newline - merge with next line
            let next = lines.remove(row + 1);
            lines[row].push_str(&next);
        }
        self.content = lines.join("\n");
        self.dirty = true;
    }

    pub fn insert_newline(&mut self) {
        let mut lines: Vec<String> = self.content.split('\n').map(|s| s.to_string()).collect();
        if lines.is_empty() {
            lines.push(String::new());
        }
        let row = self.cursor_row.min(lines.len() - 1);
        let line = &mut lines[row];
        
        // Convert to character-based indexing
        let line_chars: Vec<char> = line.chars().collect();
        let col = self.cursor_col.min(line_chars.len());
        
        // Split line at character position
        let (left_chars, right_chars) = line_chars.split_at(col);
        *line = left_chars.iter().collect();
        let rest: String = right_chars.iter().collect();
        
        self.cursor_row = row + 1;
        self.cursor_col = 0;
        lines.insert(self.cursor_row, rest);
        self.content = lines.join("\n");
        self.dirty = true;
    }

    pub fn move_left(&mut self) {
        if self.cursor_col > 0 {
            self.cursor_col -= 1;
        } else if self.cursor_row > 0 {
            self.cursor_row -= 1;
            // Use character count, not byte length
            self.cursor_col = self
                .content
                .split('\n')
                .nth(self.cursor_row)
                .map(|s| s.chars().count())
                .unwrap_or(0);
        }
    }

    pub fn move_right(&mut self) {
        let lines: Vec<&str> = self.content.split('\n').collect();
        if lines.is_empty() {
            return;
        }
        // Use character count, not byte length
        let line_char_len = lines[self.cursor_row.min(lines.len() - 1)].chars().count();
        if self.cursor_col < line_char_len {
            self.cursor_col += 1;
        } else if self.cursor_row + 1 < lines.len() {
            self.cursor_row += 1;
            self.cursor_col = 0;
        }
    }

    pub fn move_up(&mut self) {
        if self.cursor_row > 0 {
            self.cursor_row -= 1;
            // Use character count, not byte length
            let line_char_len = self
                .content
                .split('\n')
                .nth(self.cursor_row)
                .map(|s| s.chars().count())
                .unwrap_or(0);
            self.cursor_col = min(self.cursor_col, line_char_len);
        }
    }

    pub fn move_down(&mut self) {
        let lines: Vec<&str> = self.content.split('\n').collect();
        if self.cursor_row + 1 < lines.len() {
            self.cursor_row += 1;
            // Use character count, not byte length
            let line_char_len = lines[self.cursor_row].chars().count();
            self.cursor_col = min(self.cursor_col, line_char_len);
        }
    }
}
