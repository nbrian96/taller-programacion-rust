use crate::item::Item;

/// Representa los comandos que el sistema puede ejecutar.
#[derive(Debug)]
pub enum Command {
    /// Inserta o actualiza una clave.
    Set { key: String, value: Option<String> },
    /// Obtiene el valor de una clave.
    Get { key: String },
    /// Muestra la cantidad de elementos.
    Length,
    /// Crea un snapshot (save_data) del estado actual.
    Snapshot,
}

impl Command {
    /// Analiza una lista de argumentos y retorna el comando correspondiente.
    pub fn analyze_command(args: &[String]) -> Result<Command, String> {
        if args.is_empty() {
            return Err("ERROR: UNKNOWN COMMAND".to_string());
        }

        let cmd = &args[0];
        match cmd.as_str() {
            "length" => {
                if args.len() == 1 {
                    Ok(Command::Length)
                } else {
                    Err("ERROR: EXTRA ARGUMENT".to_string())
                }
            }
            "snapshot" => {
                if args.len() == 1 {
                    Ok(Command::Snapshot)
                } else {
                    Err("ERROR: EXTRA ARGUMENT".to_string())
                }
            }
            "get" => match args.len() {
                1 => Err("ERROR: MISSING ARGUMENT".to_string()),
                2 => Ok(Command::Get {
                    key: args[1].to_string(),
                }),
                _ => Err("ERROR: EXTRA ARGUMENT".to_string()),
            },
            "set" => match args.len() {
                1 => Err("ERROR: MISSING ARGUMENT".to_string()),
                2 => Ok(Command::Set {
                    key: args[1].to_string(),
                    value: None,
                }),
                3 => Ok(Command::Set {
                    key: args[1].to_string(),
                    value: Some(args[2].to_string()),
                }),
                _ => Err("ERROR: EXTRA ARGUMENT".to_string()),
            },
            _ => Err("ERROR: UNKNOWN COMMAND".to_string()),
        }
    }

    /// Ejecuta el comando sobre el almacén de ítems provisto.
    pub fn execute(self, items: &mut Item) -> Result<(), String> {
        match self {
            Command::Set { key, value } => Self::execute_set(items, key, value),
            Command::Get { key } => Self::execute_get(items, key),
            Command::Length => Self::execute_length(items),
            Command::Snapshot => Self::execute_snapshot(items),
        }
    }

    fn execute_set(items: &mut Item, key: String, value: Option<String>) -> Result<(), String> {
        if let Some(v) = value {
            items.set(key, v)?;
        } else {
            items.unset(key)?;
        }
        println!("OK");
        Ok(())
    }

    fn execute_get(items: &Item, key: String) -> Result<(), String> {
        let result = items.get(&key);
        let value = result.ok_or("ERROR: NOT FOUND")?.to_string();
        println!("{}", value);
        Ok(())
    }

    fn execute_length(items: &Item) -> Result<(), String> {
        println!("{}", items.length());
        Ok(())
    }

    fn execute_snapshot(items: &Item) -> Result<(), String> {
        items.save_data()?;
        println!("OK");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analyze_length() -> Result<(), String> {
        let args = vec!["length".to_string()];
        let cmd = Command::analyze_command(&args)?;
        assert!(matches!(cmd, Command::Length));
        Ok(())
    }

    #[test]
    fn test_analyze_set_with_value() -> Result<(), String> {
        let args = vec!["set".to_string(), "k1".to_string(), "v1".to_string()];
        let cmd = Command::analyze_command(&args)?;
        if let Command::Set { key, value } = cmd {
            assert_eq!(key, "k1");
            assert_eq!(value, Some("v1".to_string()));
        } else {
            panic!("Expected Set command");
        }
        Ok(())
    }

    #[test]
    fn test_analyze_extra_argument() {
        let args = vec!["length".to_string(), "extra".to_string()];
        let result = Command::analyze_command(&args);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "ERROR: EXTRA ARGUMENT");
    }

    #[test]
    fn test_analyze_missing_argument() {
        let args = vec!["get".to_string()];
        let result = Command::analyze_command(&args);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "ERROR: MISSING ARGUMENT");
    }
}
