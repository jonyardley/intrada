use chrono::Utc;
use intrada_core::domain::exercise::Exercise;
use intrada_core::domain::piece::Piece;
use intrada_core::domain::types::Tempo;

/// Create the stub data per data-model.md
pub fn create_stub_data() -> (Vec<Piece>, Vec<Exercise>) {
    let now = Utc::now();
    let pieces = vec![Piece {
        id: ulid::Ulid::new().to_string(),
        title: "Clair de Lune".to_string(),
        composer: "Claude Debussy".to_string(),
        key: Some("Db Major".to_string()),
        tempo: Some(Tempo {
            marking: Some("Andante tr\u{00e8}s expressif".to_string()),
            bpm: Some(66),
        }),
        notes: Some("Third movement of Suite bergamasque".to_string()),
        tags: vec!["impressionist".to_string(), "piano".to_string()],
        created_at: now,
        updated_at: now,
    }];
    let exercises = vec![Exercise {
        id: ulid::Ulid::new().to_string(),
        title: "Hanon No. 1".to_string(),
        composer: Some("Charles-Louis Hanon".to_string()),
        category: Some("Technique".to_string()),
        key: Some("C Major".to_string()),
        tempo: Some(Tempo {
            marking: Some("Moderato".to_string()),
            bpm: Some(108),
        }),
        notes: Some("The Virtuoso Pianist \u{2014} Exercise 1".to_string()),
        tags: vec!["technique".to_string(), "warm-up".to_string()],
        created_at: now,
        updated_at: now,
    }];
    (pieces, exercises)
}

/// Sample piece names for the "Add Sample Item" button
pub const SAMPLE_PIECES: &[(&str, &str)] = &[
    ("Moonlight Sonata", "Ludwig van Beethoven"),
    ("Nocturne Op. 9 No. 2", "Fr\u{00e9}d\u{00e9}ric Chopin"),
    ("Gymnop\u{00e9}die No. 1", "Erik Satie"),
    ("Prelude in C Major", "Johann Sebastian Bach"),
    ("Liebestr\u{00e4}ume No. 3", "Franz Liszt"),
];
