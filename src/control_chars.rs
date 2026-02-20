pub type Cell = i32;
// Control characters (GPM default set)
pub struct ControlChars {
    pub open: Cell, // begin quote
    pub close: Cell, // end quote
    pub def:Cell, // definition introducer
    pub arg_sep: Cell, // argument separator
    pub apply: Cell, // apply / call
    pub load_arg: Cell, // argument reference
}
impl ControlChars {
    pub fn default()->Self {
        ControlChars{
            open: '<' as Cell, // begin quote
            close: '>' as Cell, // end quote
            def:'§' as Cell, // definition introducer
            arg_sep: ',' as Cell, // argument separator
            apply: ';' as Cell, // apply / call
            load_arg: '~' as Cell, // argument reference
        }
    }
}