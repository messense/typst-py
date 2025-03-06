use comemo::Track;
use ecow::{eco_format, EcoString};
use serde::Serialize;
use typst::diag::{bail, StrResult, Warned};
use typst::foundations::{Content, IntoValue, LocatableSelector, Scope};
use typst::layout::PagedDocument;
use typst::syntax::Span;
use typst::World;
use typst_eval::{eval_string, EvalMode};

use crate::world::SystemWorld;

/// Processes an input file to extract provided metadata
#[derive(Debug, Clone)]
pub struct QueryCommand {
    /// Defines which elements to retrieve
    pub selector: String,

    /// Extracts just one field from all retrieved elements
    pub field: Option<String>,

    /// Expects and retrieves exactly one element
    pub one: bool,

    /// The format to serialize in
    pub format: SerializationFormat,
}

// Output file format for query command
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum SerializationFormat {
    Json,
    Yaml,
}

/// Execute a query command.
pub fn query(world: &mut SystemWorld, command: &QueryCommand) -> StrResult<String> {
    let Warned { output, warnings } = typst::compile(world);

    match output {
        // Retrieve and print query results.
        Ok(document) => {
            let data = retrieve(world, command, &document)?;
            let serialized = format(data, command)?;
            Ok(serialized)
        }
        // Print errors and warnings.
        Err(errors) => {
            let mut message = EcoString::from("failed to compile document");
            for (i, error) in errors.into_iter().enumerate() {
                message.push_str(if i == 0 { ": " } else { ", " });
                message.push_str(&error.message);
            }
            for warning in warnings {
                message.push_str(": ");
                message.push_str(&warning.message);
            }
            Err(message)
        }
    }
}

/// Retrieve the matches for the selector.
fn retrieve(
    world: &dyn World,
    command: &QueryCommand,
    document: &PagedDocument,
) -> StrResult<Vec<Content>> {
    let selector = eval_string(
        &typst::ROUTINES,
        world.track(),
        &command.selector,
        Span::detached(),
        EvalMode::Code,
        Scope::default(),
    )
    .map_err(|errors| {
        let mut message = EcoString::from("failed to evaluate selector");
        for (i, error) in errors.into_iter().enumerate() {
            message.push_str(if i == 0 { ": " } else { ", " });
            message.push_str(&error.message);
        }
        message
    })?
    .cast::<LocatableSelector>()
    .map_err(|e| e.message().clone())?;

    Ok(document
        .introspector
        .query(&selector.0)
        .into_iter()
        .collect::<Vec<_>>())
}

/// Format the query result in the output format.
fn format(elements: Vec<Content>, command: &QueryCommand) -> StrResult<String> {
    if command.one && elements.len() != 1 {
        bail!("expected exactly one element, found {}", elements.len());
    }

    let mapped: Vec<_> = elements
        .into_iter()
        .filter_map(|c| match &command.field {
            Some(field) => c.get_by_name(field).ok(),
            _ => Some(c.into_value()),
        })
        .collect();

    if command.one {
        let Some(value) = mapped.first() else {
            bail!("no such field found for element");
        };
        serialize(value, command.format)
    } else {
        serialize(&mapped, command.format)
    }
}

/// Serialize data to the output format.
fn serialize(data: &impl Serialize, format: SerializationFormat) -> StrResult<String> {
    match format {
        SerializationFormat::Json => {
            serde_json::to_string_pretty(data).map_err(|e| eco_format!("{e}"))
        }
        SerializationFormat::Yaml => serde_yaml::to_string(&data).map_err(|e| eco_format!("{e}")),
    }
}
