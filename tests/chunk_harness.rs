use gpm_in_rust::{Cell, ControlChars, GpmVm};

#[test]
fn chunk_harness_z_dawnego_main() {
    let def = '&' as Cell;
    let control_chars = ControlChars { def, ..ControlChars::default() };

    let data = [
        "&DE",
        "F,Suc,<&1,2,3,4,5,6,7,8,9,1", // świadome przełamanie definicji wewnątrz parametru
        "0,&DEF,1,<~>~1;;>;",
        "&Suc,9;",
        "&Suc,7;",
        "&Suc,10;",
        "&Suc,3,",
        ";",
        "Ala ma kota Mruczka.;",
        "> Ola ma psa.",
    ];

    let expected = [
        "",
        "",
        "",
        "10",
        "8",
        "20",
        "",
        "4",
        "Ala ma kota Mruczka.;",
        "",
    ];

    let mut vm = GpmVm::new(control_chars, 50_000);

    for (input, expected_output) in data.iter().zip(expected.iter()) {
        let out = vm.run(input);
        assert_eq!(out, *expected_output, "input={:?}", input);
    }

    let end = vm.end();
    assert_eq!(end, "");
}
