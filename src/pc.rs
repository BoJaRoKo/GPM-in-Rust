#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pc {
    // Main cycle
    Start,
    Copy,
    Scan,
    Q2,

    // Warning character actions
    Fn,
    NextItem,
    Apply,
    LoadArg,
    EndFn,
    Exit,

    // Machine code macros
    DEF,
    VAL,
    UPDATE,
    BIN,
    DEC,
    BAR,

    // Monitors
    Monitor(u8),

    // End
    Finish,
    NoInput,
}
