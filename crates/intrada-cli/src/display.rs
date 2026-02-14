use intrada_core::{LibraryItemView, ViewModel};

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

    println!("\n{} item(s)", vm.item_count);
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
