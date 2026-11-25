use serde::{Deserialize, Serialize};

use crate::domain::profile::ProfileField;

/// Where on the create form the failure the banner names actually is (#1595).
/// Set only by the handler that reported the error, and only where the form has
/// somewhere to point; every other path leaves it `None`, which reads as the
/// banner alone.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
#[cfg_attr(feature = "facet_typegen", repr(C))]
pub enum FormErrorTarget {
    Piece {
        field: FormErrorField,
    },
    /// The chord chart as a whole, which is what a chart holding no bars at all
    /// produces: there is nothing inside it to point at.
    Chart,
    /// One bar of the chord chart, numbered from 1, and the token the parser
    /// stumbled on.
    ChartBar {
        bar_number: usize,
        token: String,
    },
    Profile {
        field: ProfileField,
    },
    /// A staged exercise by position, from 0, in the exercises the event
    /// carried. `field` is `None` for a chosen exercise, which has no field of
    /// its own. The shell reorders that list without telling the core, so the
    /// position holds only until it does (#1390 decision 12).
    Exercise {
        index: usize,
        field: Option<FormErrorField>,
    },
}

/// A field the add form shows. A validation failure on anything else maps to no
/// target rather than a field nobody can reach. Named apart from the shell's own
/// `FormField` view, which it would otherwise shadow inside the app module.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
#[cfg_attr(feature = "facet_typegen", repr(C))]
pub enum FormErrorField {
    Title,
    Composer,
    Tempo,
    Notes,
    Tags,
    /// The exercise's inline variation rows (#1831). Appended last: the
    /// bincode wire is positional.
    Variations,
}
