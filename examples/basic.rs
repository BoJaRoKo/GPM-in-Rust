use gpm_in_rust::{Cell, ControlChars, GpmVm};

fn main() {
    let def = '&' as Cell;
    let control_chars = ControlChars { def, ..ControlChars::default() };

    let mut vm = GpmVm::new(control_chars, 50_000);

    // Definicja makra i natychmiastowe użycie (wszystko w jednym chunku)
    let _ = vm.run("&DEF,Suc,<&1,2,3,4,5,6,7,8,9,10,&DEF,1,<~>~1;;>;");
    let out = vm.run("&Suc,7;");

    println!("{}", out);

    let _ = vm.end();
}
