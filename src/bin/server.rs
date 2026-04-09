//! Servidor multihilo para el sistema de almacenamiento clave-valor MiniKV.
//! Este binario gestiona conexiones TCP entrantes, permitiendo que múltiples clientes
//! manipulen una base de datos en memoria de forma concurrente y segura.

use minikv::command::Command;
use minikv::errors;
use minikv::item::Item;

use std::env::args;
use std::io::{BufRead, BufReader, Write, stdin};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;

/// Punto de entrada del servidor.
/// Valida los argumentos y lanza el bucle de escucha del servidor.
fn main() {
    let argv: Vec<String> = args().collect();
    let Some(address) = argv.get(1) else {
        println!("ERROR \"{}\"", errors::INVALID_ARGS);
        return;
    };

    if argv.len() != 2 {
        println!("ERROR \"{}\"", errors::INVALID_ARGS);
        return;
    }

    if let Err(e) = server_run(address) {
        println!("ERROR \"{}\"", e);
    }
}

/// Configura el listener TCP y gestiona la aceptación de nuevos clientes.
/// # Errores
/// Retorna el motivo del error si no puede realizar el binding al socket o cargar la DB.
fn server_run(address: &str) -> Result<(), String> {
    let listener =
        TcpListener::bind(address).map_err(|_| errors::SERVER_SOCKET_BINDING.to_string())?;

    let items = Item::new()?;
    let shared_db = Arc::new(Mutex::new(items));

    let shared_db_clone = Arc::clone(&shared_db);
    let listener_clone = listener
        .try_clone()
        .map_err(|_| errors::SERVER_SOCKET_BINDING.to_string())?;

    thread::spawn(move || {
        for stream in listener_clone.incoming() {
            match stream {
                Ok(client_stream) => {
                    spawn_client_handler(client_stream, Arc::clone(&shared_db_clone))
                }
                Err(_) => println!("ERROR \"{}\"", errors::CONNECTION_CLOSED),
            }
        }
    });

    let mut buffer = String::new();
    let mut stdin_reader = BufReader::new(stdin());

    while let Ok(n) = stdin_reader.read_line(&mut buffer) {
        if n == 0 {
            break;
        }
        buffer.clear();
    }

    Ok(())
}

/// Lanza un nuevo hilo para gestionar la comunicación con un cliente específico.
fn spawn_client_handler(stream: TcpStream, db: Arc<Mutex<Item>>) {
    thread::spawn(move || {
        if let Err(e) = handle_client(stream, db) {
            println!("ERROR \"{}\"", e);
        }
    });
}

/// Bucle principal de atención al cliente. Lee líneas, ejecuta comandos y envía respuestas.
fn handle_client(mut stream: TcpStream, db: Arc<Mutex<Item>>) -> Result<(), String> {
    let reader = BufReader::new(
        stream
            .try_clone()
            .map_err(|_| errors::CONNECTION_CLOSED.to_string())?,
    );

    for line in reader.lines() {
        let line = line.map_err(|_| errors::CONNECTION_CLOSED.to_string())?;
        let response = process_line(&line, &db)?;

        writeln!(stream, "{}", response).map_err(|_| errors::CONNECTION_CLOSED.to_string())?;
        stream
            .flush()
            .map_err(|_| errors::CONNECTION_CLOSED.to_string())?;
    }
    Ok(())
}

/// Analiza una línea recibida y ejecuta el comando resultante sobre la DB protegida.
fn process_line(line: &str, db: &Arc<Mutex<Item>>) -> Result<String, String> {
    let args: Vec<String> = Item::split_line(line);
    let mut db_lock = db.lock().map_err(|_| errors::POISONED_LOCK.to_string())?;

    let result = match Command::analyze_command(&args) {
        Ok(cmd) => match cmd.execute(&mut db_lock) {
            Ok(res) => res.unwrap_or_else(|| "OK".to_string()),
            Err(e) => format!("ERROR \"{}\"", e),
        },
        Err(e) => format!("ERROR \"{}\"", e),
    };
    Ok(result)
}
