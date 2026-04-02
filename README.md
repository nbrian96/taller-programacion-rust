# MiniKV — Trabajo Práctico

Mini key-value store persistente implementado en Rust.

---

## Estructura del proyecto

```
src/
├── main.rs       ← Punto de entrada. Parsea args, llama a executor.
```

---

## Comandos de desarrollo

```bash
# Compilar
cargo build

# Compilar y ejecutar
cargo run -- set clave1 valor1
cargo run -- get clave1
cargo run -- length
cargo run -- snapshot

# Ejecutar todos los tests
cargo test

# Ejecutar tests de un módulo
cargo test store
cargo test log

# Verificar warnings y lints
# Existen diferentes niveles y grupos de lints en Clippy:

# 1. Básico: Convierte todos los warnings estándar en errores
cargo clippy -- -D warnings

# 2. Indexación Segura: Prohíbe indexar con [n] que puede causar panics
cargo clippy -- -D clippy::indexing-slicing

# 3. Estilo Moderno: Sugiere usar `let else` en lugar de `match` simple
cargo clippy -- -D clippy::manual-let-else

# 4. Pedante: Nivel muy estricto para máxima calidad de código
# (Aviso: Suele requerir muchos cambios en documentación y estilo)
cargo clippy -- -D clippy::pedantic

# Otros grupos útiles (opcionales):
# -- -D clippy::perf         -> Enfocado en rendimiento
# -- -D clippy::complexity   -> Enfocado en simplificar código
# -- -D clippy::style        -> Enfocado en legibilidad y convenciones

# Formatear código
cargo fmt

# Generar documentación
cargo doc --open

# Generar entregable
zip -r entrega.zip Cargo.toml Cargo.lock src/ tests/
```


---

## Restricciones del enunciado (recordatorio)

| ❌ Prohibido | ✅ Alternativa |
|---|---|
| `.unwrap()` / `.expect()` | `?` operator / `match` / `if let` |
| `std::process::exit()` | Retornar `Result` desde `main` o usar código de salida implícito |
| `mem::*` | Ownership + referencias naturales |
| `.clone()` / `.copy()` en datos principales | Pasar ownership o usar referencias |
| Crates externos | Solo `std` |
| `unsafe` blocks | Safe Rust únicamente |
| `panic!()` directo | `Result<T, E>` propagado |
| Funciones > 30 líneas | Particionarlas en funciones auxiliares |

---

## Requerimientos no funcionales

| # | Requerimiento |
|---|---|
| 1 | El proyecto debe desarrollarse en **Rust estable (1.94)** usando únicamente la biblioteca estándar. |
| 2 | Se deben implementar **tests unitarios**. |
| 3 | **No se permite usar crates externos.** |
| 4 | El código fuente debe **compilar en la versión estable** del compilador. |
| 5 | **No se permiten bloques `unsafe`.** |
| 6 | El código debe **funcionar en Unix / Linux**. |
| 7 | Los programas deben **ejecutarse desde la línea de comandos**. |
| 8 | La compilación **no debe arrojar warnings**, ni del compilador ni de `clippy`. |
| 9 | Funciones y tipos de datos deben estar **documentados** siguiendo el estándar de `cargo doc`. |
| 10 | El código debe formatearse con **`cargo fmt`**. |
| 11 | Las funciones **no deben superar las 30 líneas**; particionarlas si es necesario. |
| 12 | Cada tipo de dato debe colocarse en un **módulo (archivo) independiente**. |

---

## Formato de archivos de persistencia

**`.minikv.log`** (append-only):
```
set clave1 valor1
set clave2 valor2
set clave2
set clave1
```

**`.minikv.data`** (snapshot):
```
clave1 valor1
clave2 valor2
```

---

## Ejemplo de sesión completa

```bash
$ cargo run -- set clave1 valor1
OK

$ cargo run -- set clave2 valor2
OK

$ cargo run -- get clave1
valor1

$ cargo run -- length
2

$ cargo run -- set clave1
OK

$ cargo run -- get clave1
NOT FOUND

$ cargo run -- length
1

$ cargo run -- snapshot
OK
```
