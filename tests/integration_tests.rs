use minikv::command::Command;
use minikv::item::Item;

#[test]
fn test_full_cycle_set_get() -> Result<(), String> {
    // GIVEN: Un modulo Item nuevo y argumentos para set
    let mut items = Item::new()?;
    let set_args = vec!["set".to_string(), "inter".to_string(), "test".to_string()];

    // WHEN: Se analiza y ejecuta el comando SET
    let cmd = Command::analyze_command(&set_args)?;
    cmd.execute(&mut items)?;

    // THEN: Al buscar la clave, el valor es el correcto
    let get_args = vec!["get".to_string(), "inter".to_string()];
    let _cmd_get = Command::analyze_command(&get_args)?;
    // En lugar de execute que printea, usamos el item directamente para validar
    assert_eq!(items.get("inter"), Some(&"test".to_string()));

    Ok(())
}
