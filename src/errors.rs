// Errores de Cliente (Recuperables)
pub const EXTRA_ARGUMENT: &str = "EXTRA ARGUMENT";
pub const MISSING_ARGUMENT: &str = "MISSING ARGUMENT";
pub const NOT_FOUND: &str = "NOT FOUND";
pub const UNKNOWN_COMMAND: &str = "UNKNOWN COMMAND";

// Errores de Comunicación
pub const CLIENT_SOCKET_BINDING: &str = "CLIENT SOCKET BINDING";
pub const CONNECTION_CLOSED: &str = "CONNECTION CLOSED";
pub const POISONED_LOCK: &str = "POISONED LOCK";
pub const TIMEOUT: &str = "TIMEOUT";

// Errores del Servidor (Irrecuperables)
pub const INVALID_ARGS: &str = "INVALID ARGS";
pub const INVALID_DATA_FILE: &str = "INVALID DATA FILE";
pub const INVALID_LOG_FILE: &str = "INVALID LOG FILE";
pub const SERVER_SOCKET_BINDING: &str = "SERVER SOCKET BINDING";
