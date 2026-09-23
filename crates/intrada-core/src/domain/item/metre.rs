use super::*;

/// A session-local override of the metre lives in the shell's click; this is
/// the piece's own, and the chart's beat split follows it.
pub(super) fn set_metre(
    model: &mut Model,
    id: String,
    next: Option<Metre>,
) -> Command<Effect, Event> {
    if let Some(ref metre) = next {
        if let Err(e) = validation::validate_metre(metre) {
            model.raise_error(e.to_string());
            return crux_core::render::render();
        }
    }

    let Some(item) = model.items.iter_mut().find(|i| i.id == id) else {
        model.raise_error(LibraryError::NotFound { id }.to_string());
        return crux_core::render::render();
    };

    model.last_error = None;
    if item.metre == next {
        return crux_core::render::render();
    }

    item.metre = next;
    let derived = item.metre.clone().unwrap_or_default();
    if let Some(chart) = item.chord_chart.as_mut() {
        chart.reassign_beats(&derived);
    }
    item.updated_at = chrono::Utc::now();
    let item = item.clone();

    model.clear_error();
    Command::all([
        crate::persistence::save_item(item),
        crux_core::render::render(),
    ])
}
