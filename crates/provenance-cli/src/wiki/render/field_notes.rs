use crate::wiki::model::{EvidenceThread, PageId};
use provenance_core::MessageRole;
use std::fmt::Write as _;

use super::html::{escape_html, icon_svg, linkify_body, snippet_html};
use super::labels::{
    counted, format_date_iso_ms, format_date_ms, node_type_word, thread_status_word,
};

pub(in crate::wiki::render) fn field_notes(threads: &[EvidenceThread], page_id: &PageId) -> String {
    if threads.is_empty() {
        return String::new();
    }
    let mut html = String::new();
    html.push_str("<div class=\"field-notes\">\n<div class=\"container\">\n");
    for thread in threads {
        let borrowed = if thread.parent_id == page_id.record_id {
            String::new()
        } else {
            format!(
                " · on {} {}",
                node_type_word(thread.parent_type),
                escape_html(&thread.parent_id)
            )
        };
        write!(
            html,
            "<div class=\"fn-head\">\n{}<h2>Discussion</h2>\n<span class=\"count\">{} · {}</span>\n<span class=\"thread-id\">{}{borrowed}</span>\n</div>\n",
            icon_svg("i-message-square"),
            counted(thread.messages.len(), "message", "messages"),
            thread_status_word(&thread.status),
            escape_html(&thread.thread_id)
        )
        .expect("writing to a String should not fail");
        html.push_str("<div class=\"fn-list\">\n");
        for note in &thread.messages {
            let (role_icon, role_label) = match note.role {
                MessageRole::User => ("i-user", "User"),
                MessageRole::Assistant => ("i-bot", "Assistant"),
                MessageRole::System => ("i-bot", "System"),
            };
            // The graph records a message role, not an actor type or author
            // name. Repeat the recorded role rather than guessing that a
            // User was human or that an Assistant was an agent persona.
            write!(
                html,
                "<div class=\"field-note\">\n<span class=\"role-badge\">{}{role_label}</span>\n<div class=\"fn-body\">\n<div class=\"fn-meta\"><span class=\"who\">{role_label}</span><time datetime=\"{}\">{}</time></div>\n<p class=\"fn-content\">{}</p>\n",
                icon_svg(role_icon),
                format_date_iso_ms(note.created_at),
                format_date_ms(note.created_at),
                linkify_body(&note.body, &note.refs)
            )
            .expect("writing to a String should not fail");
            for snippet in note
                .refs
                .iter()
                .filter_map(|reference| reference.snippet.as_ref())
            {
                html.push_str(&snippet_html(snippet));
            }
            html.push_str("</div>\n</div>\n");
        }
        html.push_str("</div>\n");
    }
    html.push_str("</div>\n</div>\n");
    html
}
