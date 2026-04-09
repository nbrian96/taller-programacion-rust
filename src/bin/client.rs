//! Cliente para el sistema de almacenamiento clave-valor MiniKV.
//! Este binario permite interactuar con un servidor MiniKV a través de comandos
//! ingresados por la entrada estándar (STDIN).

use minikv::errors;
use std::env::args;
use std::io::{BufRead, BufReader, Write, stdin, stdout};
use std::net::TcpStream;
use std::time::Duration;

const TIMEOUT_SECONDS: u64 = 5;

/// Punto de entrada del cliente.
/// Lee los argumentos de línea de comandos, valida la dirección y ejecuta el bucle principal.
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

    if let Err(e) = client_run(address) {
        println!("ERROR \"{}\"", e);
    }
}

/// Inicia la conexión con el servidor y gestiona el bucle de entrada/salida.
/// # Errores
/// Retorna un string con el motivo del error si falla la conexión o la comunicación.
fn client_run(address: &str) -> Result<(), String> {
    let mut socket =
        TcpStream::connect(address).map_err(|_| errors::CLIENT_SOCKET_BINDING.to_string())?;

    socket
        .set_read_timeout(Some(Duration::from_secs(TIMEOUT_SECONDS)))
        .map_err(|_| errors::CONNECTION_CLOSED.to_string())?;
    socket
        .set_write_timeout(Some(Duration::from_secs(TIMEOUT_SECONDS)))
        .map_err(|_| errors::CONNECTION_CLOSED.to_string())?;

    let mut input_reader = BufReader::new(stdin());
    let mut socket_reader = BufReader::new(
        socket
            .try_clone()
            .map_err(|_| errors::CONNECTION_CLOSED.to_string())?,
    );

    let mut buffer = String::new();
    while input_reader
        .read_line(&mut buffer)
        .map_err(|_| errors::CONNECTION_CLOSED.to_string())?
        > 0
    {
        process_command(&mut socket, &mut socket_reader, &buffer)?;
        buffer.clear();
    }
    Ok(())
}

/// Envía un comando al servidor y muestra su respuesta.
///
/// # Argumentos
/// * `socket` - El stream de red para enviar datos.
/// * `reader` - El BufReader para leer la respuesta del servidor de forma eficiente.
/// * `cmd` - La cadena de comando a enviar.
fn process_command(
    socket: &mut TcpStream,
    reader: &mut BufReader<TcpStream>,
    cmd: &str,
) -> Result<(), String> {
    socket
        .write_all(cmd.as_bytes())
        .map_err(|e| match e.kind() {
            std::io::ErrorKind::TimedOut | std::io::ErrorKind::WouldBlock => {
                errors::TIMEOUT.to_string()
            }
            _ => errors::CONNECTION_CLOSED.to_string(),
        })?;

    let mut response = String::new();
    match reader.read_line(&mut response) {
        Ok(0) => Err(errors::CONNECTION_CLOSED.to_string()),
        Ok(_) => {
            print!("{}", response);
            let _ = stdout().flush();
            Ok(())
        }
        Err(e) => match e.kind() {
            std::io::ErrorKind::TimedOut | std::io::ErrorKind::WouldBlock => {
                Err(errors::TIMEOUT.to_string())
            }
            _ => Err(errors::CONNECTION_CLOSED.to_string()),
        },
    }
}
