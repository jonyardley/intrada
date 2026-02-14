mod display;
mod shell;
mod storage;

use anyhow::Result;
use clap::{Parser, Subcommand};
use intrada_core::domain::exercise::ExerciseEvent;
use intrada_core::domain::piece::PieceEvent;
use intrada_core::domain::types::{
    CreateExercise, CreatePiece, ListQuery, Tempo, UpdateExercise, UpdatePiece,
};
use intrada_core::Event;

use crate::display::{print_error, print_item_detail, print_item_list, print_success};
use crate::shell::Shell;
use crate::storage::SqliteStore;

#[derive(Parser)]
#[command(name = "intrada", about = "A music practice library manager")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Add a piece or exercise to the library
    Add {
        #[command(subcommand)]
        item: AddCommands,
    },
    /// List items in the library
    List {
        /// Filter by type (piece or exercise)
        #[arg(long, value_name = "TYPE")]
        r#type: Option<String>,
        /// Filter by key
        #[arg(long)]
        key: Option<String>,
        /// Filter by category (exercises only)
        #[arg(long)]
        category: Option<String>,
        /// Filter by tag (can be repeated)
        #[arg(long)]
        tag: Vec<String>,
    },
    /// Show details of a specific item
    Show {
        /// Item ID
        id: String,
    },
    /// Edit an existing item
    Edit {
        /// Item ID
        id: String,
        /// New title
        #[arg(long)]
        title: Option<String>,
        /// New composer
        #[arg(long)]
        composer: Option<String>,
        /// New key
        #[arg(long)]
        key: Option<String>,
        /// New tempo marking
        #[arg(long)]
        tempo_marking: Option<String>,
        /// New tempo BPM
        #[arg(long)]
        tempo_bpm: Option<u16>,
        /// New notes
        #[arg(long)]
        notes: Option<String>,
    },
    /// Delete an item from the library
    Delete {
        /// Item ID
        id: String,
        /// Skip confirmation
        #[arg(long, short)]
        yes: bool,
    },
    /// Add tags to an item
    Tag {
        /// Item ID
        id: String,
        /// Tags to add
        tags: Vec<String>,
    },
    /// Remove tags from an item
    Untag {
        /// Item ID
        id: String,
        /// Tags to remove
        tags: Vec<String>,
    },
    /// Search items by text
    Search {
        /// Search query
        query: String,
        /// Filter by type (piece or exercise)
        #[arg(long, value_name = "TYPE")]
        r#type: Option<String>,
    },
}

#[derive(Subcommand)]
enum AddCommands {
    /// Add a piece
    Piece {
        /// Title of the piece
        title: String,
        /// Composer (required)
        #[arg(long)]
        composer: String,
        /// Musical key
        #[arg(long)]
        key: Option<String>,
        /// Tempo marking (e.g. "Allegro")
        #[arg(long)]
        tempo_marking: Option<String>,
        /// Tempo in BPM
        #[arg(long)]
        tempo_bpm: Option<u16>,
        /// Notes
        #[arg(long)]
        notes: Option<String>,
        /// Tags (can be repeated)
        #[arg(long)]
        tag: Vec<String>,
    },
    /// Add an exercise
    Exercise {
        /// Title of the exercise
        title: String,
        /// Composer
        #[arg(long)]
        composer: Option<String>,
        /// Category
        #[arg(long)]
        category: Option<String>,
        /// Musical key
        #[arg(long)]
        key: Option<String>,
        /// Tempo marking
        #[arg(long)]
        tempo_marking: Option<String>,
        /// Tempo in BPM
        #[arg(long)]
        tempo_bpm: Option<u16>,
        /// Notes
        #[arg(long)]
        notes: Option<String>,
        /// Tags (can be repeated)
        #[arg(long)]
        tag: Vec<String>,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let store = SqliteStore::new()?;
    let shell = Shell::new(store);
    let vm = shell.load_data()?;

    match cli.command {
        Commands::Add { item } => match item {
            AddCommands::Piece {
                title,
                composer,
                key,
                tempo_marking,
                tempo_bpm,
                notes,
                tag,
            } => {
                let title_for_msg = title.clone();
                let tempo = build_tempo(tempo_marking, tempo_bpm);
                let vm = shell.run(Event::Piece(PieceEvent::Add(CreatePiece {
                    title,
                    composer,
                    key,
                    tempo,
                    notes,
                    tags: tag,
                })))?;

                if let Some(ref err) = vm.error {
                    print_error(err);
                    std::process::exit(1);
                }

                if let Some(item) = vm.items.last() {
                    print_success(&format!("Added piece: {} ({})", title_for_msg, item.id));
                }
            }
            AddCommands::Exercise {
                title,
                composer,
                category,
                key,
                tempo_marking,
                tempo_bpm,
                notes,
                tag,
            } => {
                let title_for_msg = title.clone();
                let tempo = build_tempo(tempo_marking, tempo_bpm);
                let vm = shell.run(Event::Exercise(ExerciseEvent::Add(CreateExercise {
                    title,
                    composer,
                    category,
                    key,
                    tempo,
                    notes,
                    tags: tag,
                })))?;

                if let Some(ref err) = vm.error {
                    print_error(err);
                    std::process::exit(1);
                }

                if let Some(item) = vm.items.last() {
                    print_success(&format!("Added exercise: {} ({})", title_for_msg, item.id));
                }
            }
        },

        Commands::List {
            r#type,
            key,
            category,
            tag,
        } => {
            let has_filters =
                r#type.is_some() || key.is_some() || category.is_some() || !tag.is_empty();

            if has_filters {
                let tags = if tag.is_empty() { None } else { Some(tag) };
                let vm = shell.run(Event::SetQuery(Some(ListQuery {
                    text: None,
                    item_type: r#type,
                    key,
                    category,
                    tags,
                })))?;
                print_item_list(&vm);
            } else {
                print_item_list(&vm);
            }
        }

        Commands::Show { id } => match vm.items.iter().find(|item| item.id == id) {
            Some(item) => print_item_detail(item),
            None => {
                print_error(&format!("Item not found: {id}"));
                std::process::exit(1);
            }
        },

        Commands::Edit {
            id,
            title,
            composer,
            key,
            tempo_marking,
            tempo_bpm,
            notes,
        } => {
            // Determine item type from current ViewModel
            let item_type = vm
                .items
                .iter()
                .find(|i| i.id == id)
                .map(|i| i.item_type.as_str());

            let tempo = if tempo_marking.is_some() || tempo_bpm.is_some() {
                Some(Some(Tempo {
                    marking: tempo_marking,
                    bpm: tempo_bpm,
                }))
            } else {
                None
            };

            let event = match item_type {
                Some("piece") => Event::Piece(PieceEvent::Update {
                    id: id.clone(),
                    input: UpdatePiece {
                        title,
                        composer,
                        key: key.map(Some),
                        tempo,
                        notes: notes.map(Some),
                        tags: None,
                    },
                }),
                Some("exercise") => Event::Exercise(ExerciseEvent::Update {
                    id: id.clone(),
                    input: UpdateExercise {
                        title,
                        composer: composer.map(Some),
                        category: None,
                        key: key.map(Some),
                        tempo,
                        notes: notes.map(Some),
                        tags: None,
                    },
                }),
                _ => {
                    print_error(&format!("Item not found: {id}"));
                    std::process::exit(1);
                }
            };

            let vm = shell.run(event)?;
            if let Some(ref err) = vm.error {
                print_error(err);
                std::process::exit(1);
            }
            print_success("Item updated.");
        }

        Commands::Delete { id, yes } => {
            if !yes {
                eprint!("Delete item {id}? [y/N] ");
                let mut input = String::new();
                std::io::stdin().read_line(&mut input)?;
                if !input.trim().eq_ignore_ascii_case("y") {
                    println!("Cancelled.");
                    return Ok(());
                }
            }

            // Determine item type
            let item_type = vm
                .items
                .iter()
                .find(|i| i.id == id)
                .map(|i| i.item_type.as_str());

            let event = match item_type {
                Some("piece") => Event::Piece(PieceEvent::Delete { id: id.clone() }),
                Some("exercise") => Event::Exercise(ExerciseEvent::Delete { id: id.clone() }),
                _ => {
                    print_error(&format!("Item not found: {id}"));
                    std::process::exit(1);
                }
            };

            let vm = shell.run(event)?;
            if let Some(ref err) = vm.error {
                print_error(err);
                std::process::exit(1);
            }
            print_success("Item deleted.");
        }

        Commands::Tag { id, tags } => {
            let item_type = vm
                .items
                .iter()
                .find(|i| i.id == id)
                .map(|i| i.item_type.as_str());

            let event = match item_type {
                Some("piece") => Event::Piece(PieceEvent::AddTags {
                    id: id.clone(),
                    tags: tags.clone(),
                }),
                Some("exercise") => Event::Exercise(ExerciseEvent::AddTags {
                    id: id.clone(),
                    tags: tags.clone(),
                }),
                _ => {
                    print_error(&format!("Item not found: {id}"));
                    std::process::exit(1);
                }
            };

            let vm = shell.run(event)?;
            if let Some(ref err) = vm.error {
                print_error(err);
                std::process::exit(1);
            }
            print_success(&format!("Tags added: {}", tags.join(", ")));
        }

        Commands::Untag { id, tags } => {
            let item_type = vm
                .items
                .iter()
                .find(|i| i.id == id)
                .map(|i| i.item_type.as_str());

            let event = match item_type {
                Some("piece") => Event::Piece(PieceEvent::RemoveTags {
                    id: id.clone(),
                    tags: tags.clone(),
                }),
                Some("exercise") => Event::Exercise(ExerciseEvent::RemoveTags {
                    id: id.clone(),
                    tags: tags.clone(),
                }),
                _ => {
                    print_error(&format!("Item not found: {id}"));
                    std::process::exit(1);
                }
            };

            let vm = shell.run(event)?;
            if let Some(ref err) = vm.error {
                print_error(err);
                std::process::exit(1);
            }
            print_success(&format!("Tags removed: {}", tags.join(", ")));
        }

        Commands::Search { query, r#type } => {
            let vm = shell.run(Event::SetQuery(Some(ListQuery {
                text: Some(query),
                item_type: r#type,
                ..Default::default()
            })))?;
            print_item_list(&vm);
        }
    }

    Ok(())
}

fn build_tempo(marking: Option<String>, bpm: Option<u16>) -> Option<Tempo> {
    if marking.is_some() || bpm.is_some() {
        Some(Tempo { marking, bpm })
    } else {
        None
    }
}
