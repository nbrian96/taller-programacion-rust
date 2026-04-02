use crate::command::CommandType;
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};

const LOG_FILE: &str = ".minikv.log";
const DATA_FILE: &str = ".minikv.data";

/// Representa el almacén de datos clave-valor.
pub struct Item {
    items: HashMap<String, String>,
}

impl Item {
    /// Crea una nueva instancia de Item cargando los datos de los archivos persistentes.
    pub fn new() -> Result<Self, String> {
        let mut items = HashMap::new();

        Self::read_data(&mut items)?;
        Self::read_log(&mut items)?;

        Ok(Self { items })
    }

    /// Retorna la cantidad de elementos almacenados.
    pub fn length(&self) -> usize {
        self.items.len()
    }

    /// Obtiene el valor asociado a una clave, si existe.
    pub fn get(&self, key: &str) -> Option<&String> {
        self.items.get(key)
    }

    /// Inserta o actualiza un par clave-valor, registrando la operación en el log.
    pub fn set(&mut self, key: String, value: String) -> Result<(), String> {
        self.save_log(&["set", &key, &value])?;
        self.items.insert(key, value);
        Ok(())
    }

    /// Elimina una clave del almacén, registrando la operación en el log.
    pub fn unset(&mut self, key: String) -> Result<(), String> {
        self.save_log(&["set", &key])?;
        self.items.remove(&key);
        Ok(())
    }

    /// Guarda todos los datos actuales en el archivo .data (snapshot) y vacía el log.
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

    /// Registra una operación en el archivo de log (append mode).
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

    /// Lee y aplica todas las operaciones registradas en el log.
    pub fn read_log(items: &mut HashMap<String, String>) -> Result<(), String> {
        if let Ok(file) = File::open(LOG_FILE) {
            let reader = BufReader::new(file);

            for l in reader.lines().map_while(Result::ok) {
                let line_args = Self::split_line(&l);

                let cmd_res: Result<CommandType, String> = line_args
                    .first()
                    .ok_or("ERROR: INVALID LOG FILE".to_string())?
                    .parse();
                let Ok(cmd_type) = cmd_res else {
                    return Err("ERROR: INVALID LOG FILE".to_string());
                };

                match (cmd_type, line_args.as_slice()) {
                    (CommandType::Set, [_, k, v]) => {
                        items.insert(k.to_string(), v.to_string());
                    }
                    (CommandType::Set, [_, k]) => {
                        items.remove(k);
                    }
                    _ => return Err("ERROR: INVALID LOG FILE".to_string()),
                }
            }
        }

        Ok(())
    }

    /// Carga el estado inicial desde el archivo .data (snapshot).
    pub fn read_data(items: &mut HashMap<String, String>) -> Result<(), String> {
        if let Ok(file) = File::open(DATA_FILE) {
            let reader = BufReader::new(file);

            for l in reader.lines().map_while(Result::ok) {
                let line_args = Self::split_line(&l);

                match line_args.as_slice() {
                    [k, v] => {
                        items.insert(k.to_string(), v.to_string());
                    }
                    _ => return Err("ERROR: INVALID DATA FILE".to_string()),
                }
            }
        }

        Ok(())
    }

    /// Separa una línea de entrada en argumentos, respetando comillas y caracteres de escape.
    fn split_line(l: &str) -> Vec<String> {
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
        // GIVEN: Una línea de log simple
        let line = "set clave valor";

        // WHEN: Se separa la línea
        let result = Item::split_line(line);

        // THEN: Los argumentos son los esperados
        assert_eq!(result, vec!["set", "clave", "valor"]);
    }

    #[test]
    fn test_split_line_with_quotes() {
        // GIVEN: Una línea con comillas y espacios internos
        let line = "set \"clave con espacio\" \"valor con espacio\"";

        // WHEN: Se separa la línea
        let result = Item::split_line(line);

        // THEN: Los argumentos mantienen los espacios internos
        assert_eq!(
            result,
            vec!["set", "clave con espacio", "valor con espacio"]
        );
    }

    #[test]
    fn test_split_line_with_escapes() {
        // GIVEN: Una línea con caracteres escapados
        let line = "set \"clave\\\" con comillas\" \"valor\\\\con backslash\"";

        // WHEN: Se separa la línea
        let result = Item::split_line(line);

        // THEN: Los caracteres se des-escapan correctamente
        assert_eq!(
            result,
            vec!["set", "clave\" con comillas", "valor\\con backslash"]
        );
    }

    #[test]
    fn test_item_set_and_get() {
        // GIVEN: Un modulo Item vacío
        let mut item = Item {
            items: HashMap::new(),
        };

        // WHEN: Se setea una clave manualmente (sin persistencia para el test unitario)
        item.items.insert("k1".to_string(), "v1".to_string());

        // THEN: Se puede recuperar el valor
        assert_eq!(item.get("k1"), Some(&"v1".to_string()));
        assert_eq!(item.length(), 1);
    }
}
