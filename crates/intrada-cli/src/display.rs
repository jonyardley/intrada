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

pub fn print_filtered_list(vm: &ViewModel, type_filter: Option<&str>, key_filter: Option<&str>, category_filter: Option<&str>, tag_filters: &[String]) {
    if let Some(ref err) = vm.error {
        print_error(err);
        return;
    }

    let filtered: Vec<&LibraryItemView> = vm.items.iter().filter(|item| {
        if let Some(t) = type_filter {
            if item.item_type != t {
                return false;
            }
        }
        if let Some(k) = key_filter {
            if item.key.as_deref() != Some(k) {
                return false;
            }
        }
        if let Some(cat) = category_filter {
            if item.item_type != "exercise" || item.subtitle != cat {
                return false;
            }
        }
        for tag in tag_filters {
            let tag_lower = tag.to_lowercase();
            if !item.tags.iter().any(|t| t.to_lowercase() == tag_lower) {
                return false;
            }
        }
        true
    }).collect();

    if filtered.is_empty() {
        println!("No matching items.");
        return;
    }

    print_table_header();

    for item in &filtered {
        print_table_row(item);
    }

    println!("\n{} item(s)", filtered.len());
}

pub fn print_item_detail(item: &LibraryItemView) {
    println!("ID:       {}", item.id);
    println!("Type:     {}", item.item_type);
    println!("Title:    {}", item.title);
    if !item.subtitle.is_empty() {
        let label = if item.item_type == "exercise" { "Category" } else { "Composer" };
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

pub fn print_search_results(vm: &ViewModel, query: &str, type_filter: Option<&str>) {
    let query_lower = query.to_lowercase();

    let filtered: Vec<&LibraryItemView> = vm.items.iter().filter(|item| {
        if let Some(t) = type_filter {
            if item.item_type != t {
                return false;
            }
        }
        item.title.to_lowercase().contains(&query_lower)
            || item.subtitle.to_lowercase().contains(&query_lower)
            || item.notes.as_ref().is_some_and(|n| n.to_lowercase().contains(&query_lower))
            || item.tags.iter().any(|t| t.to_lowercase().contains(&query_lower))
    }).collect();

    if filtered.is_empty() {
        println!("No items matching \"{query}\".");
        return;
    }

    print_table_header();

    for item in &filtered {
        print_table_row(item);
    }

    println!("\n{} result(s)", filtered.len());
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
    let id_short = if item.id.len() > 26 {
        &item.id[..26]
    } else {
        &item.id
    };
    let title_display = truncate(&item.title, 28);
    let subtitle_display = truncate(&item.subtitle, 18);
    let key_display = item.key.as_deref().unwrap_or("");

    println!(
        "{:<28} {:<8} {:<30} {:<20} {}",
        id_short, item.item_type, title_display, subtitle_display, key_display
    );
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() > max {
        format!("{}...", &s[..max.saturating_sub(3)])
    } else {
        s.to_string()
    }
}
