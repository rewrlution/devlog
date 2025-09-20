use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use pulldown_cmark::{Event as MdEvent, Parser, Tag, TagEnd};

// Very simple Markdown renderer suitable for Preview mode. It renders block-level
// structures (headings, lists, paragraphs, code blocks) with basic styling, and
// pre-wraps at the given width so scrolling and the scrollbar remain correct.
pub fn render_markdown_simple(src: &str, width: usize) -> Vec<Line<'static>> {
    let mut lines: Vec<(String, Style)> = Vec::new();
    let mut cur = String::new();
    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    enum MdBlock { Paragraph, Heading(u32), Code, ListItem }
    let mut block = MdBlock::Paragraph;

    // Local style mapper to avoid name/type conflicts
    fn style_for(block: &MdBlock) -> Style {
        match block {
            MdBlock::Heading(_) => Style::default().add_modifier(Modifier::BOLD).fg(Color::Cyan),
            MdBlock::Code => Style::default().fg(Color::Gray),
            MdBlock::ListItem => Style::default(),
            MdBlock::Paragraph => Style::default(),
        }
    }

    let parser = Parser::new(src);
    for ev in parser {
        match ev {
            MdEvent::Start(Tag::Heading { level, .. }) => {
                if !cur.is_empty() { lines.push((cur.clone(), style_for(&block))); cur.clear(); }
                block = MdBlock::Heading(level as u32);
            }
            MdEvent::End(TagEnd::Heading(_)) => {
                if !cur.is_empty() { lines.push((cur.clone(), style_for(&block))); cur.clear(); }
                block = MdBlock::Paragraph;
                lines.push((String::new(), Style::default()));
            }
            MdEvent::Start(Tag::CodeBlock(_)) => {
                if !cur.is_empty() { lines.push((cur.clone(), style_for(&block))); cur.clear(); }
                block = MdBlock::Code;
            }
            MdEvent::End(TagEnd::CodeBlock) => {
                if !cur.is_empty() { lines.push((cur.clone(), style_for(&block))); cur.clear(); }
                block = MdBlock::Paragraph;
                lines.push((String::new(), Style::default()));
            }
            MdEvent::Start(Tag::List(_)) => {}
            MdEvent::End(TagEnd::List(_)) => {}
            MdEvent::Start(Tag::Item) => {
                if !cur.is_empty() { lines.push((cur.clone(), style_for(&block))); cur.clear(); }
                block = MdBlock::ListItem;
                cur.push_str("• ");
            }
            MdEvent::End(TagEnd::Item) => {
                if !cur.is_empty() { lines.push((cur.clone(), style_for(&block))); cur.clear(); }
                block = MdBlock::Paragraph;
            }
            MdEvent::Text(t) | MdEvent::Code(t) => {
                cur.push_str(&t);
            }
            MdEvent::SoftBreak | MdEvent::HardBreak => {
                lines.push((cur.clone(), style_for(&block)));
                cur.clear();
            }
            _ => {}
        }
    }
    if !cur.is_empty() { lines.push((cur, style_for(&block))); }

    // Wrap to width, applying the same style to each wrapped segment
    let mut out: Vec<Line> = Vec::new();
    for (mut s, st) in lines {
        if width == 0 {
            out.push(Line::from(Span::styled(s, st)));
            continue;
        }
        while !s.is_empty() {
            let take = s.chars().take(width).collect::<String>();
            let rem_len = s.chars().count();
            out.push(Line::from(Span::styled(take.clone(), st)));
            if rem_len <= width { break; }
            s = s.chars().skip(width).collect();
        }
    }
    out
}
