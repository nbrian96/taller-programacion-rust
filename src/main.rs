use minikv::command::Command;
use minikv::item::Item;

use std::env::args;

fn main() {
    let mut items = match Item::new() {
        Ok(it) => it,
        Err(e) => {
            println!("{}", e);
            return;
        }
    };

    let args: Vec<String> = args().skip(1).collect();

    let comando = match Command::analyze_command(&args) {
        Ok(comando) => comando,
        Err(e) => {
            println!("{}", e);
            return;
        }
    };

    match comando.execute(&mut items) {
        Ok(_) => (),
        Err(e) => println!("{}", e),
    };
}
