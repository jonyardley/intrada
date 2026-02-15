use intrada_core::{LibraryItemView, SessionView, ViewModel};

pub fn print_item_list(vm: &ViewModel) {
    if let Some(ref err) = vm.error {
        print_error(err);
        return;
    }

    if vm.items.is_empty() {
        println!("No items in library.");
        return;
    }

    print_table_header();

    for item in &vm.items {
        print_table_row(item);
    }

    let count = vm.items.len();
    if count == 1 {
        println!("\n1 item");
    } else {
        println!("\n{count} items");
    }
}

pub fn print_item_detail(item: &LibraryItemView) {
    println!("ID:       {}", item.id);
    println!("Type:     {}", item.item_type);
    println!("Title:    {}", item.title);
    if !item.subtitle.is_empty() {
        let label = if item.category.is_some() {
            "Category"
        } else {
            "Composer"
        };
        println!("{label}:  {}", item.subtitle);
    }
    if let Some(ref key) = item.key {
        println!("Key:      {key}");
    }
    if let Some(ref tempo) = item.tempo {
        println!("Tempo:    {tempo}");
    }
    if let Some(ref notes) = item.notes {
        println!("Notes:    {notes}");
    }
    if !item.tags.is_empty() {
        println!("Tags:     {}", item.tags.join(", "));
    }
    println!("Created:  {}", item.created_at);
    println!("Updated:  {}", item.updated_at);

    if let Some(ref practice) = item.practice {
        println!();
        println!(
            "Practice: {} session{}, {} min total",
            practice.session_count,
            if practice.session_count == 1 { "" } else { "s" },
            practice.total_minutes
        );
    }
}

pub fn print_session_logged(duration: u32, item_id: &str) {
    println!("Logged {duration} min practice session for item {item_id}");
}

pub fn print_session_detail(session: &SessionView) {
    println!("Session:  {}", session.id);
    println!("Item:     {} ({})", session.item_title, session.item_type);
    println!("Duration: {} min", session.duration_minutes);
    println!("Started:  {}", session.started_at);
    println!("Logged:   {}", session.logged_at);
    if let Some(ref notes) = session.notes {
        println!("Notes:    {notes}");
    }
}

pub fn print_session_list(sessions: &[&SessionView]) {
    if sessions.is_empty() {
        println!("No practice sessions found.");
        return;
    }

    println!(
        "{:<28} {:<24} {:<8} {:<24} NOTES",
        "SESSION ID", "ITEM", "MINS", "DATE"
    );
    println!("{}", "-".repeat(100));

    for session in sessions {
        let id_short = truncate(&session.id, 26);
        let item_display = truncate(&session.item_title, 22);
        let date_display = truncate(&session.logged_at, 22);
        let notes_preview = session
            .notes
            .as_deref()
            .map(|n| truncate(n, 20))
            .unwrap_or_default();

        println!(
            "{:<28} {:<24} {:<8} {:<24} {}",
            id_short, item_display, session.duration_minutes, date_display, notes_preview
        );
    }

    let count = sessions.len();
    if count == 1 {
        println!("\n1 session");
    } else {
        println!("\n{count} sessions");
    }
}

pub fn print_error(msg: &str) {
    eprintln!("Error: {msg}");
}

pub fn print_success(msg: &str) {
    println!("{msg}");
}

fn print_table_header() {
    println!(
        "{:<28} {:<8} {:<30} {:<20} KEY",
        "ID", "TYPE", "TITLE", "COMPOSER/CATEGORY"
    );
    println!("{}", "-".repeat(100));
}

fn print_table_row(item: &LibraryItemView) {
    let id_short = truncate(&item.id, 26);
    let title_display = truncate(&item.title, 28);
    let subtitle_display = truncate(&item.subtitle, 18);
    let key_display = item.key.as_deref().unwrap_or("");

    println!(
        "{:<28} {:<8} {:<30} {:<20} {}",
        id_short, item.item_type, title_display, subtitle_display, key_display
    );
}

fn truncate(s: &str, max: usize) -> String {
    let char_count = s.chars().count();
    if char_count > max {
        let truncated: String = s.chars().take(max.saturating_sub(3)).collect();
        format!("{truncated}...")
    } else {
        s.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_truncate_ascii() {
        assert_eq!(truncate("hello", 10), "hello");
        assert_eq!(truncate("hello world!", 8), "hello...");
    }

    #[test]
    fn test_truncate_unicode() {
        // Dvořák — the 'ř' is multi-byte in UTF-8
        assert_eq!(truncate("Dvořák", 10), "Dvořák");
        // Japanese characters are multi-byte
        assert_eq!(truncate("日本語タグテスト", 5), "日本...");
    }

    #[test]
    fn test_truncate_exact_boundary() {
        assert_eq!(truncate("12345", 5), "12345");
        assert_eq!(truncate("123456", 5), "12...");
    }
}
