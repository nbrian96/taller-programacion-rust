//! Lógica de dominio y persistencia para el sistema MiniKV.

use crate::command_type::CommandType;
use crate::errors;

use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};

const LOG_FILE: &str = ".minikv.log";
const DATA_FILE: &str = ".minikv.data";

/// Representa el almacén de datos clave-valor en memoria.
pub struct Item {
    /// Mapa que contiene la asociación clave-valor.
    items: HashMap<String, String>,
}

impl Item {
    /// Crea una nueva instancia de `Item` cargando los datos de los archivos persistentes.
    /// Retorna un error si alguno de los archivos persistentes tiene un formato inválido.
    pub fn new() -> Result<Self, String> {
        let mut items = HashMap::new();
        Self::read_data(&mut items)?;
        Self::read_log(&mut items)?;
        Ok(Self { items })
    }

    /// Retorna la cantidad de elementos almacenados en el almacén.
    pub fn length(&self) -> usize {
        self.items.len()
    }

    /// Obtiene una referencia al valor asociado a una clave, si existe.
    pub fn get(&self, key: &str) -> Option<&String> {
        self.items.get(key)
    }

    /// Inserta o actualiza un par clave-valor, registrando la operación en el log persistente.
    /// Retorna un error si no se pudo escribir en el archivo de log.
    pub fn set(&mut self, key: String, value: String) -> Result<(), String> {
        self.save_log(&["set", &key, &value])?;
        self.items.insert(key, value);
        Ok(())
    }

    /// Elimina una clave del almacén, registrando la operación en el log persistente.
    /// Retorna un error si no se pudo escribir en el archivo de log.
    pub fn unset(&mut self, key: String) -> Result<(), String> {
        self.save_log(&["set", &key])?;
        self.items.remove(&key);
        Ok(())
    }

    /// Guarda todos los datos actuales en el archivo `.data` (snapshot) y vacía el log.
    /// Retorna un error si falla la escritura en disco.
    pub fn save_data(&self) -> Result<(), String> {
        let mut file = File::create(DATA_FILE).map_err(|e| e.to_string())?;
        for (key, value) in &self.items {
            let k_esc = key.replace('\\', "\\\\").replace('"', "\\\"");
            let v_esc = value.replace('\\', "\\\\").replace('"', "\\\"");
            writeln!(file, "\"{}\" \"{}\"", k_esc, v_esc).map_err(|e| e.to_string())?;
        }

        File::create(LOG_FILE).map_err(|e| e.to_string())?;
        Ok(())
    }

    /// Registra una operación en el archivo de log (modo append).
    /// * `args` - Lista de strings que componen la línea de comando a registrar.
    pub fn save_log(&self, args: &[&str]) -> Result<(), String> {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(LOG_FILE)
            .map_err(|e| e.to_string())?;

        if args.is_empty() {
            return Ok(());
        }

        let mut line = args.first().ok_or("ERROR")?.to_string();
        for s in args.iter().skip(1) {
            line.push(' ');
            line.push('"');
            line.push_str(&s.replace('\\', "\\\\").replace('"', "\\\""));
            line.push('"');
        }

        writeln!(file, "{}", line).map_err(|e| e.to_string())?;
        Ok(())
    }

    /// Lee y aplica todas las operaciones registradas en el archivo de log.
    /// * `items` - El mapa donde se aplicarán las operaciones leídas.
    pub fn read_log(items: &mut HashMap<String, String>) -> Result<(), String> {
        if let Ok(file) = File::open(LOG_FILE) {
            let reader = BufReader::new(file);
            for l in reader.lines().map_while(Result::ok) {
                let line_args = Self::split_line(&l);
                Self::apply_log_line(items, &line_args)?;
            }
        }
        Ok(())
    }

    /// Procesa y aplica una única línea del log al mapa de ítems.
    fn apply_log_line(items: &mut HashMap<String, String>, args: &[String]) -> Result<(), String> {
        let cmd_res: Result<CommandType, String> = args
            .first()
            .ok_or(errors::INVALID_LOG_FILE.to_string())?
            .parse();

        let Ok(cmd_type) = cmd_res else {
            return Err(errors::INVALID_LOG_FILE.to_string());
        };

        match (cmd_type, args) {
            (CommandType::Set, [_, k, v]) => {
                items.insert(k.to_string(), v.to_string());
                Ok(())
            }
            (CommandType::Set, [_, k]) => {
                items.remove(k);
                Ok(())
            }
            _ => Err(errors::INVALID_LOG_FILE.to_string()),
        }
    }

    /// Carga el estado inicial desde el archivo `.data` (snapshot).
    pub fn read_data(items: &mut HashMap<String, String>) -> Result<(), String> {
        if let Ok(file) = File::open(DATA_FILE) {
            let reader = BufReader::new(file);
            for l in reader.lines().map_while(Result::ok) {
                let line_args = Self::split_line(&l);
                match line_args.as_slice() {
                    [k, v] => {
                        items.insert(k.to_string(), v.to_string());
                    }
                    _ => return Err(errors::INVALID_DATA_FILE.to_string()),
                }
            }
        }
        Ok(())
    }

    /// Separa una línea de entrada en argumentos, respetando comillas y caracteres de escape.
    /// * `l` - La línea de texto a procesar.
    pub fn split_line(l: &str) -> Vec<String> {
        let mut line_args = Vec::new();
        let mut current = String::new();
        let mut in_quotes = false;
        let mut escaped = false;
        for c in l.chars() {
            if escaped {
                current.push(c);
                escaped = false;
            } else if c == '\\' {
                escaped = true;
            } else if c == '"' {
                in_quotes = !in_quotes;
            } else if c == ' ' && !in_quotes {
                if !current.is_empty() {
                    line_args.push(current.clone());
                    current.clear();
                }
            } else {
                current.push(c);
            }
        }
        if !current.is_empty() {
            line_args.push(current);
        }
        line_args
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_line_simple() {
        let line = "set key value";
        let split = Item::split_line(line);
        assert_eq!(split, vec!["set", "key", "value"]);
    }

    #[test]
    fn test_split_line_with_quotes() {
        let line = "set \"clave con espacios\" \"valor con espacios\"";
        let split = Item::split_line(line);
        assert_eq!(
            split,
            vec!["set", "clave con espacios", "valor con espacios"]
        );
    }

    #[test]
    fn test_split_line_with_escaped_quotes() {
        let line = "set key \"valor con \\\"comillas\\\"\"";
        let split = Item::split_line(line);
        assert_eq!(split, vec!["set", "key", "valor con \"comillas\""]);
    }

    #[test]
    fn test_split_line_with_backslash() {
        let line = "set key \"valor con \\\\ backslash\"";
        let split = Item::split_line(line);
        assert_eq!(split, vec!["set", "key", "valor con \\ backslash"]);
    }
}
