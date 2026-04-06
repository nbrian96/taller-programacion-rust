//! Definición y parseo de los tipos de comandos válidos.

use crate::errors;
use std::str::FromStr;

/// Representa el tipo de comando (la palabra clave inicial).
#[derive(Debug, PartialEq, Eq)]
pub enum CommandType {
    /// Comando `set`.
    Set,
    /// Comando `get`.
    Get,
    /// Comando `length`.
    Length,
    /// Comando `snapshot`.
    Snapshot,
}

impl FromStr for CommandType {
    type Err = String;

    /// Convierte un string en un `CommandType`.
    /// Soporta "set", "get", "length" y "snapshot" (case-sensitive).
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "set" => Ok(CommandType::Set),
            "get" => Ok(CommandType::Get),
            "length" => Ok(CommandType::Length),
            "snapshot" => Ok(CommandType::Snapshot),
            _ => Err(errors::UNKNOWN_COMMAND.to_string()),
        }
    }
}
