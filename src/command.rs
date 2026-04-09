//! Gestión de la estructura y ejecución de comandos en MiniKV.

use crate::command_type::CommandType;
use crate::errors;
use crate::item::Item;

/// Representa los comandos que el sistema puede ejecutar.
#[derive(Debug)]
pub enum Command {
    /// Inserta o actualiza una clave: `set <clave> <valor>`.
    Set {
        /// La clave a insertar o modificar.
        key: String,
        /// El valor a asociar (opcional para unsets).
        value: Option<String>,
    },
    /// Obtiene el valor de una clave: `get <clave>`.
    Get {
        /// La clave a consultar.
        key: String,
    },
    /// Muestra la cantidad de elementos en la DB: `length`.
    Length,
    /// Crea un snapshot (save_data) del estado actual: `snapshot`.
    Snapshot,
}

impl Command {
    /// Analiza una lista de argumentos y retorna el comando correspondiente.
    /// Retorna `UNKNOWN COMMAND` si el tipo no existe, o `MISSING/EXTRA ARGUMENT`
    /// si la cantidad de parámetros es incorrecta.
    pub fn analyze_command(args: &[String]) -> Result<Command, String> {
        let cmd_str = args.first().ok_or(errors::UNKNOWN_COMMAND)?;
        let cmd_type: CommandType = cmd_str.parse()?;

        match cmd_type {
            CommandType::Length => Self::parse_length(args),
            CommandType::Snapshot => Self::parse_snapshot(args),
            CommandType::Get => Self::parse_get(args),
            CommandType::Set => Self::parse_set(args),
        }
    }

    /// Parsea el comando length validando que no tenga argumentos extra.
    fn parse_length(args: &[String]) -> Result<Command, String> {
        if args.len() == 1 {
            Ok(Command::Length)
        } else {
            Err(errors::EXTRA_ARGUMENT.to_string())
        }
    }

    /// Parsea el comando snapshot validando que no tenga argumentos extra.
    fn parse_snapshot(args: &[String]) -> Result<Command, String> {
        if args.len() == 1 {
            Ok(Command::Snapshot)
        } else {
            Err(errors::EXTRA_ARGUMENT.to_string())
        }
    }

    /// Parsea el comando get validando la presencia de la clave obligatoria.
    fn parse_get(args: &[String]) -> Result<Command, String> {
        match args.len() {
            1 => Err(errors::MISSING_ARGUMENT.to_string()),
            2 => Ok(Command::Get {
                key: args.get(1).ok_or(errors::MISSING_ARGUMENT)?.to_string(),
            }),
            _ => Err(errors::EXTRA_ARGUMENT.to_string()),
        }
    }

    /// Parsea el comando set validando los argumentos mínimos y máximos.
    fn parse_set(args: &[String]) -> Result<Command, String> {
        match args.len() {
            1 => Err(errors::MISSING_ARGUMENT.to_string()),
            2 => Ok(Command::Set {
                key: args.get(1).ok_or(errors::MISSING_ARGUMENT)?.to_string(),
                value: None,
            }),
            3 => Ok(Command::Set {
                key: args.get(1).ok_or(errors::MISSING_ARGUMENT)?.to_string(),
                value: Some(args.get(2).ok_or(errors::MISSING_ARGUMENT)?.to_string()),
            }),
            _ => Err(errors::EXTRA_ARGUMENT.to_string()),
        }
    }

    /// Ejecuta el comando sobre el almacén de ítems provisto.
    /// Retorna `Ok(Some(String))` con el resultado del comando, u `Ok(None)`
    /// si el comando solo confirma éxito (como `set` o `snapshot`).
    pub fn execute(self, items: &mut Item) -> Result<Option<String>, String> {
        match self {
            Command::Set { key, value } => {
                if let Some(v) = value {
                    items.set(key, v)?;
                } else {
                    items.unset(key)?;
                }
                Ok(None)
            }
            Command::Get { key } => {
                let val = items.get(&key).ok_or(errors::NOT_FOUND)?;
                Ok(Some(val.clone()))
            }
            Command::Length => Ok(Some(items.length().to_string())),
            Command::Snapshot => {
                items.save_data()?;
                Ok(None)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analyze_command_get_valid() {
        let args = vec!["get".to_string(), "mi_clave".to_string()];
        let cmd = Command::analyze_command(&args).unwrap();
        if let Command::Get { key } = cmd {
            assert_eq!(key, "mi_clave");
        } else {
            panic!("Debería ser Command::Get");
        }
    }

    #[test]
    fn test_analyze_command_set_valid() {
        let args = vec!["set".to_string(), "k".to_string(), "v".to_string()];
        let cmd = Command::analyze_command(&args).unwrap();
        if let Command::Set { key, value } = cmd {
            assert_eq!(key, "k");
            assert_eq!(value, Some("v".to_string()));
        } else {
            panic!("Debería ser Command::Set");
        }
    }

    #[test]
    fn test_analyze_command_missing_arg() {
        let args = vec!["get".to_string()];
        let res = Command::analyze_command(&args);
        assert!(res.is_err());
        assert_eq!(res.unwrap_err(), errors::MISSING_ARGUMENT);
    }

    #[test]
    fn test_analyze_command_extra_arg() {
        let args = vec!["length".to_string(), "algo_mas".to_string()];
        let res = Command::analyze_command(&args);
        assert!(res.is_err());
        assert_eq!(res.unwrap_err(), errors::EXTRA_ARGUMENT);
    }
}
