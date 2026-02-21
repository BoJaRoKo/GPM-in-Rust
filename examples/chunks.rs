use gpm_in_rust::{Cell, ControlChars, GpmVm};

fn main() {
    let def = '&' as Cell;
    let control_chars = ControlChars { def, ..ControlChars::default() };

    let mut vm = GpmVm::new(control_chars, 50_000);

    // Definicja makra rozbita na 2 wywołania run()
    let _ = vm.run("&DE");
    let _ = vm.run("F,Suc,<&1,2,3,4,5,6,7,8,9,10,&DEF,1,<~>~1;;>;");

    // Wywołanie makra też może być rozbite
    let _ = vm.run("&Suc,3,");
    let out = vm.run(";");

    println!("{}", out);

    let _ = vm.end();
}
