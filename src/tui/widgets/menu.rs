//! Reusable menu widget with selectable items

use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::tui::theme::Theme;

/// A menu item with label and description
#[derive(Debug, Clone)]
pub struct MenuItem {
    pub label: String,
    pub description: String,
    pub enabled: bool,
}

impl MenuItem {
    pub fn new(label: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            description: description.into(),
            enabled: true,
        }
    }

    pub fn disabled(mut self) -> Self {
        self.enabled = false;
        self
    }
}

/// A selectable menu with items
pub struct Menu<'a> {
    items: &'a [MenuItem],
    selected: usize,
    title: Option<&'a str>,
    theme: &'a Theme,
}

impl<'a> Menu<'a> {
    pub fn new(items: &'a [MenuItem], theme: &'a Theme) -> Self {
        Self {
            items,
            selected: 0,
            title: None,
            theme,
        }
    }

    pub fn selected(mut self, index: usize) -> Self {
        self.selected = index.min(self.items.len().saturating_sub(1));
        self
    }

    pub fn title(mut self, title: &'a str) -> Self {
        self.title = Some(title);
        self
    }

    /// Move selection up, skipping disabled items
    pub fn select_previous(items: &[MenuItem], current: usize) -> usize {
        if current == 0 {
            return current;
        }
        let mut new_idx = current - 1;
        while new_idx > 0 && !items[new_idx].enabled {
            new_idx -= 1;
        }
        if items[new_idx].enabled {
            new_idx
        } else {
            current
        }
    }

    /// Move selection down, skipping disabled items
    pub fn select_next(items: &[MenuItem], current: usize) -> usize {
        if current >= items.len().saturating_sub(1) {
            return current;
        }
        let mut new_idx = current + 1;
        while new_idx < items.len() - 1 && !items[new_idx].enabled {
            new_idx += 1;
        }
        if items[new_idx].enabled {
            new_idx
        } else {
            current
        }
    }
}

impl Widget for Menu<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = if let Some(title) = self.title {
            Block::default()
                .title(format!(" {} ", title))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(self.theme.border))
                .style(Style::default().bg(self.theme.bg_alt))
        } else {
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(self.theme.border))
                .style(Style::default().bg(self.theme.bg_alt))
        };

        let inner = block.inner(area);
        block.render(area, buf);

        // Render each menu item
        for (i, item) in self.items.iter().enumerate() {
            if i >= inner.height as usize {
                break;
            }

            let is_selected = i == self.selected;
            let prefix = if is_selected { "> " } else { "  " };

            let (label_style, desc_style) = if !item.enabled {
                (
                    Style::default().fg(self.theme.fg_dim),
                    Style::default().fg(self.theme.fg_dim),
                )
            } else if is_selected {
                (
                    Style::default().fg(self.theme.primary).bold(),
                    Style::default().fg(self.theme.fg),
                )
            } else {
                (
                    Style::default().fg(self.theme.fg),
                    Style::default().fg(self.theme.fg_dim),
                )
            };

            let line = Line::from(vec![
                Span::styled(prefix, label_style),
                Span::styled(format!("{:<18}", item.label), label_style),
                Span::styled(&item.description, desc_style),
            ]);

            let item_area = Rect {
                x: inner.x,
                y: inner.y + i as u16,
                width: inner.width,
                height: 1,
            };

            Paragraph::new(line).render(item_area, buf);
        }
    }
}
