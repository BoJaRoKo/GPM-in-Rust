// main.rs — GPM (Strachey) VM skeleton in Rust (variant A: faithful VM model)
//
// Status: compiles, runs, implements:
// - fixed store ST[0..self.mem_size)
// - init MST[0..38] with six machine macros (DEF/VAL/UPDATE/BIN/DEC/BAR)
// - core I/O + Load + NextCh
// - main scan cycle: Start / Copy / Scan / Q2
//
// The rest of the labels are present as states but currently stubbed to Finish/Monitor.
//
// NOTE about encoding:
// We read input as a stream of Rust `char` so the warning character '§' works correctly
// even if the input is UTF-8. Store cells are i32, matching Appendix 2 "index" usage.

mod pc;
mod control_chars;

use std::io::{self, Write};
use crate::pc::Pc;
use crate::control_chars::ControlChars;

type Cell = i32;
type Idx = i32;

/* 
// Control characters (GPM default set)
const CH_OPEN: Cell = '<' as Cell; // begin quote
const CH_CLOSE: Cell = '>' as Cell; // end quote
const CH_DEF: Cell = '&' as Cell; // definition introducer (original '§')
const CH_ARGSEP: Cell = ',' as Cell; // argument separator
const CH_APPLY: Cell = ';' as Cell; // apply / call
const CH_LOADARG: Cell = '~' as Cell; // argument reference
*/
                                      // Appendix 2: Marker = -2**20 (Titan-style). We use the same sentinel.
const MARKER: Cell = -(1 << 20);


struct Vm {
    cc:ControlChars,
    mem_size: usize,
    input:String,
    output:String,

    // fixed store
    st: Vec<Cell>,

    // registers
    a: Cell,
    w: Cell,

    h: Idx,
    p: Idx,
    f: Idx,
    c: Idx,

    s: Idx,
    e: Idx,
    q: Idx,

    pc: Pc,
}

impl Vm {
    fn new(control_chars:ControlChars, mem_size:usize) -> Self {
        let mut vm = Vm {
            cc: control_chars,
            mem_size: mem_size,

            input: "".to_string(),
            output: "".to_string(),

            st: vec![0;mem_size],

            a: 0,
            w: 0,

            h: 0,
            p: 0,
            f: 0,
            c: 0,

            // Appendix 2 initial values:
            // S=39, E=33, q=1, Marker = -2 ↑ 20 (we keep an equivalent stable negative sentinel)
            s: 39,
            e: 33,
            q: 1,

            pc: Pc::Start,
        };

        vm.init_mst();
        vm
    }

    // Appendix 2 uses "-2 ↑ 20" (Titan-style). For our faithful VM we just need
    // a stable negative sentinel value unlikely to collide with machine macro tags.

    #[inline]
    fn u(i: Idx) -> usize {
        debug_assert!(i >= 0);
        i as usize
    }

    fn init_mst(&mut self) {
        // MST from Appendix 2 (39 cells, copied to base of ST)
        // Name-value pairs for machine macros: DEF, VAL, UPDATE, BIN, DEC, BAR
        //
        // Layout per Appendix 2:
        // link, len, name chars..., value (negative tag)
        let mst: [Cell; 39] = [
            -1,
            4,
            'D' as Cell,
            'E' as Cell,
            'F' as Cell,
            -1,
            0,
            4,
            'V' as Cell,
            'A' as Cell,
            'L' as Cell,
            -2,
            6,
            7,
            'U' as Cell,
            'P' as Cell,
            'D' as Cell,
            'A' as Cell,
            'T' as Cell,
            'E' as Cell,
            -3,
            12,
            4,
            'B' as Cell,
            'I' as Cell,
            'N' as Cell,
            -4,
            21,
            4,
            'D' as Cell,
            'E' as Cell,
            'C' as Cell,
            -5,
            27,
            4,
            'B' as Cell,
            'A' as Cell,
            'R' as Cell,
            -6,
        ];

        self.st[..39].copy_from_slice(&mst);

        // start registers (already set in new(), but keep explicit)
        self.h = 0;
        self.p = 0;
        self.f = 0;
        self.c = 0;

        self.s = 39;
        self.e = 33;
        self.q = 1;
    }

    // WriteSymbol[A]
    fn write_symbol(&mut self, x: Cell) {
        let ch = char::from_u32(x as u32).unwrap_or('\u{FFFD}');
        self.output.push(ch);
    }

    // ReadSymbol[A]
    fn read_symbol(&mut self) -> Option<Cell> {
        let x = self.input.pop();
        match x {
            Some(x) => { /* eprint!("{}", x); */ Some(x as Cell) },
            None => None,
        }
    }

    // routine Load
    fn load(&mut self) {
        if self.h == 0 {
            self.write_symbol(self.a);
        } else {
            let s = Self::u(self.s);
            if s >= self.mem_size {
                self.pc = Pc::Monitor(11);
                return;
            }
            self.st[s] = self.a;
            self.s += 1;
        }
    }

    // routine NextCh
    fn next_ch(&mut self) -> bool {
        if self.c == 0 {
            let a = self.read_symbol();
            match a {
                Some(a) => { self.a = a; true },
                None => false,
            }
        } else {
            let c = Self::u(self.c);
            self.a = self.st[c];
            self.c += 1;
            true
        }
    }

    // ===== Main cycle states (Appendix 2) =====

    // Start: NextCh ... goto ...
    fn op_start(&mut self) -> Pc {
        if !self.next_ch() { return Pc::NoInput; }
        match self.a {
            x if x == self.cc.open => {
                self.q += 1;
                Pc::Q2
            }
            x if x == self.cc.def => Pc::Fn,
            x if x == self.cc.arg_sep => {
                if self.h == 0 {
                    Pc::Copy
                } else {
                    Pc::NextItem
                }
            }
            x if x == self.cc.apply => {
                if self.h == 0 {
                    Pc::Copy
                } else {
                    Pc::Apply
                }
            }
            x if x == self.cc.load_arg => {
                if self.p == 0 {
                    Pc::Copy
                } else {
                    Pc::LoadArg
                }
            }
            MARKER => {
                if self.h == 0 && self.c == 0 {
                    Pc::Finish
                } else {
                    Pc::EndFn
                }
            }
            x if x == self.cc.close => {
                if self.h == 0 && self.c == 0 {
                    Pc::Finish
                } else {
                    Pc::Exit
                }
            }
            _ => Pc::Copy,
        }
    }

    // Copy: Load
    fn op_copy(&mut self) -> Pc {
        self.load();
        Pc::Scan
    }

    // Scan: if q = 1 goto Start ; Q2 otherwise
    fn op_scan(&mut self) -> Pc {
        if self.q == 1 {
            Pc::Start
        } else {
            Pc::Q2
        }
    }

    // Q2: NextCh; ... quoting depth handling
    fn op_q2(&mut self) -> Pc {
        self.next_ch();

        match self.a {
            x if x == self.cc.open => {
                self.q += 1;
                Pc::Copy
            }
            x if x == self.cc.close => {
                self.q -= 1;
                if self.q == 1 {
                    Pc::Start
                } else {
                    Pc::Copy
                }
            }
            _ => Pc::Copy,
        }
    }

    // ===== Stubs for the rest (to be implemented 1:1) =====
    fn find(&mut self, x: Idx) {
        // Appendix 2:
        // Find[x] be
        //   { A, W := E, x
        //     { for r = 0 to ST[W]-1 do
        //         if ST[W+r] ≠ ST[A+r+1] go to Next
        //       W := A+1+ST[W]
        //       return
        //     Next:
        //       A := ST[A]
        //     } repeat until A < 0
        //     go to Monitor7
        //   }
        //
        // :contentReference[oaicite:0]{index=0}

        let mut a: Idx = self.e;
        let w: Idx = x;

        loop {
            // bounds (minimal sanity)
            if a < 0 || w < 0 {
                self.pc = Pc::Monitor(7);
                return;
            }
            let a_u = Self::u(a);
            let w_u = Self::u(w);
            if a_u >= self.mem_size || w_u >= self.mem_size {
                self.pc = Pc::Monitor(11); // internal/overflow (temporary)
                return;
            }

            // len = ST[W]
            let len: Idx = self.st[w_u] as Idx;
            if len < 0 {
                self.pc = Pc::Monitor(11);
                return;
            }

            // compare name item at W[0..len-1] with entry name at A[1..len]
            let mut matched = true;
            for r in 0..len {
                let lw = w + r;
                let ra = a + r + 1;
                if lw < 0 || ra < 0 {
                    matched = false;
                    break;
                }
                let lw_u = Self::u(lw);
                let ra_u = Self::u(ra);
                if lw_u >= self.mem_size || ra_u >= self.mem_size {
                    self.pc = Pc::Monitor(11);
                    return;
                }
                if self.st[lw_u] != self.st[ra_u] {
                    matched = false;
                    break;
                }
            }

            if matched {
                // W := A + 1 + ST[W]
                let new_w: Idx = a + 1 + len;
                self.w = new_w as Cell;
                return;
            }

            // Next: A := ST[A]
            let next_a: Idx = self.st[a_u] as Idx;
            a = next_a;

            // repeat until A < 0
            if a < 0 {
                self.pc = Pc::Monitor(7);
                return;
            }
        }
    }

    fn jump_if_marked(&mut self, x: Cell) -> Option<Pc> {
        // Appendix 2:
        // JumpIfMarked[x] be
        //   { if x < 0 go to MachineMacro[-x]
        //     return
        //   }
        //
        // MachineMacro = [DEF, VAL, UPDATE, BIN, DEC, BAR]
        //
        // :contentReference[oaicite:0]{index=0}

        if x >= 0 {
            return None;
        }

        // x is a negative tag: -1..-6 (per MST)
        let idx = (-x) as i32;
        match idx {
            1 => Some(Pc::DEF),
            2 => Some(Pc::VAL),
            3 => Some(Pc::UPDATE),
            4 => Some(Pc::BIN),
            5 => Some(Pc::DEC),
            6 => Some(Pc::BAR),
            _ => {
                // Unknown negative tag: treat as fatal internal error for now
                self.pc = Pc::Monitor(11);
                Some(Pc::Monitor(11))
            }
        }
    }

    fn op_fn(&mut self) -> Pc {
        // Appendix 2 (Warning Character Actions):
        // Fn: H, S, F, ST[S], ST[S+1], ST[S+2], ST[S+3] := S+3, S+4, S+1, H, F, 0, 0
        //     goto Start
        //
        //

        let s0 = self.s;
        let h0 = self.h;
        let f0 = self.f;

        // bounds check: we will write ST[s0..s0+3] and then set S = s0+4
        let need_top = s0 + 4;
        if need_top < 0 || Self::u(need_top) > self.mem_size {
            return Pc::Monitor(11); // stack overflow (temporary; later map to the right monitor)
        }

        // ST[S] := H
        self.st[Self::u(s0)] = h0 as Cell;
        // ST[S+1] := F
        self.st[Self::u(s0 + 1)] = f0 as Cell;
        // ST[S+2] := 0
        self.st[Self::u(s0 + 2)] = 0;
        // ST[S+3] := 0
        self.st[Self::u(s0 + 3)] = 0;

        // H := S+3
        self.h = s0 + 3;
        // F := S+1
        self.f = s0 + 1;
        // S := S+4
        self.s = s0 + 4;

        Pc::Start
    }

    fn op_next_item(&mut self) -> Pc {
        // Appendix 2 (Warning Character Actions):
        // NextItem: if H = 0 goto Copy
        //          H, S, ST[H], ST[S] := S, S+1, S-H-ST[H], 0
        //          goto Start
        //
        //

        if self.h == 0 {
            return Pc::Copy;
        }

        let s0 = self.s;
        let h0 = self.h;

        // bounds: we will write ST[h0] and ST[s0], then set H=s0, S=s0+1
        if s0 < 0 || h0 < 0 || Self::u(s0) >= self.mem_size || Self::u(h0) >= self.mem_size {
            return Pc::Monitor(11); // temporary hard-stop; later map to proper monitor
        }
        if Self::u(s0 + 1) > self.mem_size {
            return Pc::Monitor(11); // overflow
        }

        // old "length so far" (header at ST[H])
        let len_so_far = self.st[Self::u(h0)] as Idx;

        // ST[H] := S - H - ST[H]
        // (close current item by writing its final length)
        let new_len = s0 - h0 - len_so_far;
        self.st[Self::u(h0)] = new_len as Cell;

        // H := S
        self.h = s0;

        // ST[S] := 0   (new item's length header)
        self.st[Self::u(s0)] = 0;

        // S := S+1
        self.s = s0 + 1;

        Pc::Start
    }

    fn op_apply(&mut self) -> Pc {
        // Appendix 2 (Warning Character Actions):
        // Apply:  if P > F goto Monitor1
        //         if H = 0 goto Copy
        //         F, P, H, S, ST[H], ST[S], ST[F-1], ST[F], ST[F+1]
        //           := ST[F], F, ST[F-1], S+1, S-H, Marker, S-F+2, P, C
        //         unless H = 0 do ST[H] := ST[H] + ST[P-1]
        //         Find[P+2]
        //         JumpIfMarked[ST[W]]
        //         C := W+1
        //         goto Start
        //
        //

        if self.p > self.f {
            return Pc::Monitor(1);
        }
        if self.h == 0 {
            return Pc::Copy;
        }

        let p0 = self.p;
        let f0 = self.f;
        let h0 = self.h;
        let s0 = self.s;
        let c0 = self.c;

        // bounds sanity (we touch: H, S, F-1, F, F+1, P-1, P+2 later)
        if f0 <= 0 || h0 < 0 || s0 < 0 || p0 < 0 {
            return Pc::Monitor(11); // temporary hard-stop
        }
        let need_max = (s0 + 1).max(f0 + 1).max(h0).max(f0);
        if Self::u(need_max) >= self.mem_size {
            return Pc::Monitor(11); // overflow / invalid
        }

        // Evaluate RHS using OLD values (simultaneous assignment).
        let new_f: Idx = self.st[Self::u(f0)] as Idx; // ST[F]
        let new_p: Idx = f0; // F
        let new_h: Idx = self.st[Self::u(f0 - 1)] as Idx; // ST[F-1]
        let new_s: Idx = s0 + 1; // S+1

        let st_h_val: Cell = (s0 - h0) as Cell; // S-H
        let st_s_val: Cell = MARKER; // Marker
        let st_fm1_val: Cell = (s0 - f0 + 2) as Cell; // S-F+2
        let st_f_val: Cell = p0 as Cell; // P
        let st_fp1_val: Cell = c0 as Cell; // C

        // Perform LHS writes (still effectively “simultaneous”, but we already captured RHS).
        self.st[Self::u(h0)] = st_h_val;
        self.st[Self::u(s0)] = st_s_val;
        self.st[Self::u(f0 - 1)] = st_fm1_val;
        self.st[Self::u(f0)] = st_f_val;
        self.st[Self::u(f0 + 1)] = st_fp1_val;

        // Update registers
        self.f = new_f;
        self.p = new_p;
        self.h = new_h;
        self.s = new_s;

        // unless H = 0 do ST[H] := ST[H] + ST[P-1]
        if self.h != 0 {
            // NOTE: P is now new_p (= old f0), so P-1 refers to (old f0-1),
            // which we already set to (s0-f0+2) above.
            let h = Self::u(self.h);
            let pm1 = Self::u(self.p - 1);
            self.st[h] = self.st[h] + self.st[pm1];
        }

        // Find[P+2]
        self.find(self.p + 2);

        // JumpIfMarked[ST[W]]
        let tag = self.st[Self::u(self.w as Idx)];
        if let Some(pc) = self.jump_if_marked(tag) {
            return pc; // go to machine macro label
        }

        // C := W+1 ; goto Start
        self.c = (self.w as Idx) + 1;
        Pc::Start
    }

    fn op_load_arg(&mut self) -> Pc {
        // Appendix 2 (Warning Character Actions):
        // LoadArg: if P=0 goto H=0 → Copy, Monitor2
        //          NextCh
        //          W := P+2
        //          if Number[A] < 0 goto Monitor3
        //          for f = 0 to Number[A]-1 do
        //            { W := W + ST[W]
        //              if ST[W] = Marker goto Monitor4
        //            }
        //          for r = 1 to ST[W]-1 do
        //            { A := ST[W+r]
        //              Load
        //            }
        //          goto Start
        //
        //

        if self.p == 0 {
            // goto H=0 -> Copy, Monitor2
            if self.h == 0 {
                return Pc::Copy;
            }
            return Pc::Monitor(2);
        }

        // NextCh
        self.next_ch();
        // If NextCh hit EOF and set pc, respect it.
        if self.pc != Pc::LoadArg {
            return self.pc;
        }

        // W := P+2
        let mut w: Idx = self.p + 2;

        // if Number[A] < 0 goto Monitor3
        let x = Self::number(self.a);
        if x < 0 {
            return Pc::Monitor(3);
        }

        // for f = 0 to Number[A]-1 do ...
        for _ in 0..x {
            // W := W + ST[W]
            let w_u = Self::u(w);
            if w_u >= self.mem_size {
                return Pc::Monitor(11);
            }
            let step = self.st[w_u] as Idx;
            w = w + step;

            // if ST[W] = Marker goto Monitor4
            let w_u2 = Self::u(w);
            if w_u2 >= self.mem_size {
                return Pc::Monitor(11);
            }
            if self.st[w_u2] == MARKER {
                return Pc::Monitor(4);
            }
        }

        // for r = 1 to ST[W]-1 do { A := ST[W+r]; Load }
        let w_u = Self::u(w);
        if w_u >= self.mem_size {
            return Pc::Monitor(11);
        }
        let len = self.st[w_u] as Idx;
        if len < 0 {
            return Pc::Monitor(11);
        }

        for r in 1..len {
            let idx = w + r;
            let idx_u = Self::u(idx);
            if idx_u >= self.mem_size {
                return Pc::Monitor(11);
            }
            self.a = self.st[idx_u];
            self.load();
            if self.pc == Pc::Monitor(11) {
                return Pc::Monitor(11);
            }
        }

        Pc::Start
    }

    // Appendix 2:
    // Number[x] = x - 16
    #[inline]
    fn number(x: Cell) -> Idx {
        (x - '0' as i32) as Idx
    }

    fn op_end_fn(&mut self) -> Pc {
        // Appendix 2 (Warning Character Actions):
        // EndFn: if F>P go to Monitor5
        //        ST[S], A := E, S
        //        while ST[A] >= P-1 + ST[P-1] do
        //            ST[A], A := ST[A] - ST[P-1], ST[A]
        //        W := ST[A}
        //        while W > P-1 do W := ST[W]
        //        ST[A] := W
        //        E := ST[S]
        //        unless H=0 do
        //            test H>P then H := H - ST[P-1]
        //            or do ST[H] := ST[H] - ST[P-1]
        //        P, C, S, A, W := ST[P], ST[P+1], S - ST[P-1], P-1, P-1 + ST[P-1]
        //        until A=S do ST[A], A, W := ST[W], A+1, W+1
        //        go to Start
        //
        // :contentReference[oaicite:0]{index=0}

        if self.f > self.p {
            return Pc::Monitor(5);
        }

        // Keep old values where simultaneous assignment relies on them.
        let p0 = self.p;
        let s0 = self.s;

        if p0 <= 0 || s0 < 0 {
            return Pc::Monitor(11); // temporary hard stop for invalid state
        }

        // calllen = ST[P-1]
        let calllen_u = Self::u(p0 - 1);
        if calllen_u >= self.mem_size {
            return Pc::Monitor(11);
        }
        let calllen: Idx = self.st[calllen_u] as Idx;

        // ST[S], A := E, S
        let s_u = Self::u(s0);
        if s_u >= self.mem_size {
            return Pc::Monitor(11);
        }
        self.st[s_u] = self.e as Cell;
        let mut a: Idx = s0;

        // while ST[A] >= P-1 + ST[P-1] do ST[A], A := ST[A]-ST[P-1], ST[A]
        let limit: Idx = (p0 - 1) + calllen;
        loop {
            let a_u = Self::u(a);
            if a_u >= self.mem_size {
                return Pc::Monitor(11);
            }
            let link: Idx = self.st[a_u] as Idx;
            if link < limit {
                break;
            }
            // simultaneous update:
            // ST[A] := link - calllen ; A := link
            self.st[a_u] = (link - calllen) as Cell;
            a = link;
            if a < 0 {
                return Pc::Monitor(11);
            }
        }

        // W := ST[A}
        let a_u = Self::u(a);
        if a_u >= self.mem_size {
            return Pc::Monitor(11);
        }
        let mut w: Idx = self.st[a_u] as Idx;

        // while W > P-1 do W := ST[W]
        let p_minus_1 = p0 - 1;
        while w > p_minus_1 {
            let w_u = Self::u(w);
            if w_u >= self.mem_size {
                return Pc::Monitor(11);
            }
            w = self.st[w_u] as Idx;
        }

        // ST[A] := W
        self.st[a_u] = w as Cell;

        // E := ST[S]
        self.e = self.st[s_u] as Idx;

        // unless H=0 do test H>P then H := H - ST[P-1] or do ST[H] := ST[H] - ST[P-1]
        if self.h != 0 {
            if self.h > p0 {
                self.h -= calllen;
            } else {
                let h_u = Self::u(self.h);
                if h_u >= self.mem_size {
                    return Pc::Monitor(11);
                }
                self.st[h_u] = (self.st[h_u] as Idx - calllen) as Cell;
            }
        }

        // P, C, S, A, W := ST[P], ST[P+1], S - ST[P-1], P-1, P-1 + ST[P-1]
        let p_u = Self::u(p0);
        let p1_u = Self::u(p0 + 1);
        if p_u >= self.mem_size || p1_u >= self.mem_size {
            return Pc::Monitor(11);
        }
        let new_p: Idx = self.st[p_u] as Idx;
        let new_c: Idx = self.st[p1_u] as Idx;
        let new_s: Idx = s0 - calllen;
        let mut a2: Idx = p0 - 1;
        let mut w2: Idx = (p0 - 1) + calllen;

        self.p = new_p;
        self.c = new_c;
        self.s = new_s;

        // until A=S do ST[A], A, W := ST[W], A+1, W+1
        while a2 != self.s {
            let a2_u = Self::u(a2);
            let w2_u = Self::u(w2);
            if a2_u >= self.mem_size || w2_u >= self.mem_size {
                return Pc::Monitor(11);
            }
            self.st[a2_u] = self.st[w2_u];
            a2 += 1;
            w2 += 1;
        }

        Pc::Start
    }

    fn op_exit(&mut self) -> Pc {
        // Appendix 2 (Main cycle / Exit):
        // Exit: unless C=H=0 go to Monitor8
        //       Finish
        //
        //

        if !(self.c == 0 && self.h == 0) {
            return Pc::Monitor(8);
        }
        Pc::Finish
    }

    fn op_def(&mut self) -> Pc {
        // Appendix 2 (Machine Code Macros):
        // DEF: unless H = 0 do ST[H] := ST[H] - ST[P-1] + 6
        //      ST[P-1], ST[P+5], E := 6, E, P+5
        //      goto EndFn
        //
        //

        let p0 = self.p;
        let e0 = self.e;

        // Minimal sanity/bounds (we touch P-1 and P+5; H only if H!=0)
        if p0 <= 0 {
            return Pc::Monitor(11);
        }
        let pm1 = p0 - 1;
        let pp5 = p0 + 5;

        if pm1 < 0 || pp5 < 0 {
            return Pc::Monitor(11);
        }
        if Self::u(pm1) >= self.mem_size || Self::u(pp5) >= self.mem_size {
            return Pc::Monitor(11);
        }

        // unless H = 0 do ST[H] := ST[H] - ST[P-1] + 6
        if self.h != 0 {
            let h = self.h;
            if h < 0 || Self::u(h) >= self.mem_size {
                return Pc::Monitor(11);
            }
            let h_u = Self::u(h);
            let pm1_u = Self::u(pm1);
            self.st[h_u] = self.st[h_u] - self.st[pm1_u] + 6;
        }

        // Simultaneous assignment:
        // ST[P-1] := 6
        // ST[P+5] := E(old)
        // E := P+5
        self.st[Self::u(pm1)] = 6;
        self.st[Self::u(pp5)] = e0 as Cell;
        self.e = pp5;

        Pc::EndFn
    }

    fn op_val(&mut self) -> Pc {
        // VAL: Find[P+6]
        // until ST[W+1]=Marker do { A,W := ST[W+1], W+1; Load }
        // go to EndFn
        self.find(self.p + 6);
        if matches!(self.pc, Pc::Monitor(_)) {
            return self.pc;
        }

        let mut w: Idx = self.w as Idx;
        loop {
            let wp1 = w + 1;
            if wp1 < 0 || Self::u(wp1) >= self.mem_size {
                return Pc::Monitor(11);
            }
            if self.st[Self::u(wp1)] == MARKER {
                break;
            }
            // A,W := ST[W+1], W+1
            self.a = self.st[Self::u(wp1)];
            w = wp1;
            self.w = w as Cell;

            self.load();
            if matches!(self.pc, Pc::Monitor(_)) {
                return self.pc;
            }
        }

        Pc::EndFn
    }

    fn op_update(&mut self) -> Pc {
        // UPDATE: Find[P+9]
        // A := P+9 + ST[P+9]
        // if ST[A] > ST[W] go to Monitor9
        // for r=1 to ST[A] do ST[W+r] := ST[A+r]
        // go to EndFn
        self.find(self.p + 9);
        if matches!(self.pc, Pc::Monitor(_)) {
            return self.pc;
        }

        let p = self.p;
        let w = self.w as Idx;

        let p9 = p + 9;
        if p9 < 0 || Self::u(p9) >= self.mem_size {
            return Pc::Monitor(11);
        }
        let a0: Idx = p9 + (self.st[Self::u(p9)] as Idx);
        if a0 < 0 || Self::u(a0) >= self.mem_size {
            return Pc::Monitor(11);
        }

        let len_new: Idx = self.st[Self::u(a0)] as Idx;
        if w < 0 || Self::u(w) >= self.mem_size {
            return Pc::Monitor(11);
        }
        let len_old: Idx = self.st[Self::u(w)] as Idx;

        if len_new > len_old {
            return Pc::Monitor(9);
        }

        for r in 1..=len_new {
            let dst = w + r;
            let src = a0 + r;
            if dst < 0 || src < 0 || Self::u(dst) >= self.mem_size || Self::u(src) >= self.mem_size {
                return Pc::Monitor(11);
            }
            self.st[Self::u(dst)] = self.st[Self::u(src)];
        }

        Pc::EndFn
    }

    fn op_bin(&mut self) -> Pc {
        // BIN: W,A := 0, (ST[P+7]='+' -> P+8, ST[P+7]='-' -> P+8, P+7)
        // until ST[A]=Marker do { x=Number[ST[A]]; unless 0<=x<=9 go Monitor10; W,A := 10*W+x, A+1 }
        // S, ST[S] := S+1, (ST[P+7]='-' -> -W, W)
        // go to EndFn
        let p = self.p;
        let p7 = p + 7;
        if p7 < 0 || Self::u(p7) >= self.mem_size {
            return Pc::Monitor(11);
        }

        let sign_ch = self.st[Self::u(p7)];
        let mut a: Idx = if sign_ch == ('+' as Cell) || sign_ch == ('-' as Cell) {
            p + 8
        } else {
            p + 7
        };

        let mut w_acc: Idx = 0;

        loop {
            if a < 0 || Self::u(a) >= self.mem_size {
                return Pc::Monitor(11);
            }
            let ch = self.st[Self::u(a)];
            if ch == MARKER {
                break;
            }

            let x = Self::number(ch); // our encoding: '0'..'9'
            if !(0..=9).contains(&x) {
                return Pc::Monitor(10);
            }

            w_acc = 10 * w_acc + x;
            a += 1;
        }

        self.s += 1;
        let su = Self::u(self.s);
        if su >= self.mem_size {
            return Pc::Monitor(11);
        }

        self.st[su] = if sign_ch == ('-' as Cell) {
            -(w_acc as Cell)
        } else {
            w_acc as Cell
        };

        Pc::EndFn
    }

    fn op_dec(&mut self) -> Pc {
        // DEC: W := ST[P+7]
        // if W<0 do { W,A := -W,'-'; Load }
        // W1 := 1; until 10*W1 > W do W1 := 10*W1
        // { A,W,W1 := Char[Quot[W,W1]], Rem[W,W1], W1/10; Load } repeat until W1 < 1
        // go to EndFn
        let p7 = self.p + 7;
        if p7 < 0 || Self::u(p7) >= self.mem_size {
            return Pc::Monitor(11);
        }

        let mut w: Idx = self.st[Self::u(p7)] as Idx;

        if w < 0 {
            w = -w;
            self.a = '-' as Cell;
            self.load();
            if matches!(self.pc, Pc::Monitor(_)) {
                return self.pc;
            }
        }

        // W1 := 1; until 10*W1 > W do W1 := 10*W1
        let mut w1: Idx = 1;
        while 10 * w1 <= w {
            w1 *= 10;
        }

        // digits
        while w1 >= 1 {
            let q = if w1 != 0 { w / w1 } else { 0 };
            let r = if w1 != 0 { w % w1 } else { w };

            // Char[q]
            self.a = ('0' as Idx + q) as Cell;
            self.load();
            if matches!(self.pc, Pc::Monitor(_)) {
                return self.pc;
            }

            w = r;
            w1 /= 10;
        }

        Pc::EndFn
    }

    fn op_bar(&mut self) -> Pc {
        // BAR: W,A := ST[P+9], ST[P+11]
        // A := opchar selects W+A, W-A, W*A, Quot[W,A], Rem[W,A]
        // Load; go to EndFn
        let p7 = self.p + 7;
        let p9 = self.p + 9;
        let p11 = self.p + 11;

        if [p7, p9, p11]
            .iter()
            .any(|&i| i < 0 || Self::u(i) >= self.mem_size)
        {
            return Pc::Monitor(11);
        }

        let op = self.st[Self::u(p7)];
        let wv: Idx = self.st[Self::u(p9)] as Idx;
        let av: Idx = self.st[Self::u(p11)] as Idx;

        let res: Idx = match op {
            x if x == ('+' as Cell) => wv + av,
            x if x == ('-' as Cell) => wv - av,
            x if x == ('x' as Cell) => wv * av,
            x if x == ('/' as Cell) => {
                if av == 0 {
                    return Pc::Monitor(11);
                } // (tu brak monitora w Appendix; traktuję jako błąd wewn.)
                wv / av
            }
            x if x == ('R' as Cell) => {
                if av == 0 {
                    return Pc::Monitor(11);
                }
                wv % av
            }
            _ => return Pc::Monitor(11),
        };

        self.a = res as Cell;
        self.load();
        if matches!(self.pc, Pc::Monitor(_)) {
            return self.pc;
        }

        Pc::EndFn
    }

    fn finish(&mut self) -> Pc {
        todo!("fn finish")
    }

    fn step(&mut self) {
        let next = match self.pc {
            // main cycle
            Pc::Start => self.op_start(),
            Pc::Copy => self.op_copy(),
            Pc::Scan => self.op_scan(),
            Pc::Q2 => self.op_q2(),

            // warning char actions
            Pc::Fn => self.op_fn(),
            Pc::NextItem => self.op_next_item(),
            Pc::Apply => self.op_apply(),
            Pc::LoadArg => self.op_load_arg(),
            Pc::EndFn => self.op_end_fn(),
            Pc::Exit => self.op_exit(),

            // machine code macros
            Pc::DEF => self.op_def(),
            Pc::VAL => self.op_val(),
            Pc::UPDATE => self.op_update(),
            Pc::BIN => self.op_bin(),
            Pc::DEC => self.op_dec(),
            Pc::BAR => self.op_bar(),

            Pc::Monitor(n) => self.monitor(n),
            Pc::Finish => panic!("Finish"),
            Pc::NoInput => panic!("NoInput"),
        };

        self.pc = next;
    }
    // CPL Write['...'] with *n/*t/*s escapes
    fn write_text(&mut self, s: &str) {
        let mut it = s.chars().peekable();
        while let Some(ch) = it.next() {
            if ch == '*' {
                match it.peek().copied() {
                    Some('n') => {
                        it.next();
                        self.write_symbol('\n' as Cell);
                    }
                    Some('t') => {
                        it.next();
                        self.write_symbol('\t' as Cell);
                    }
                    Some('s') => {
                        it.next();
                        self.write_symbol(' ' as Cell);
                    }
                    _ => self.write_symbol('*' as Cell),
                }
            } else {
                self.write_symbol(ch as u32 as Cell);
            }
        }
    }
    // Appendix 2 §3 Item[x]
    fn item(&mut self, x: Idx) {
        // save A,H
        let a0 = self.a;
        let h0 = self.h;
        self.h = 0;

        if x < 0 || Self::u(x) >= self.mem_size {
            self.write_text("*n(Item: bad pointer)");
            self.a = a0;
            self.h = h0;
            return;
        }

        let stx = self.st[Self::u(x)] as Idx;

        // if ST[x]=0 → incomplete object
        let end_k: Idx = if stx == 0 {
            (self.s - x - 1).max(0)
        } else {
            (stx - 1).max(0)
        };

        for k in 1..=end_k {
            let idx = x + k;
            if idx < 0 || Self::u(idx) >= self.mem_size {
                break;
            }
            self.a = self.st[Self::u(idx)];
            self.load(); // with H=0 → output
        }

        if stx == 0 {
            self.write_text("...*t(Incomplete)");
        }

        // restore A,H
        self.a = a0;
        self.h = h0;
    }

    fn monitor(&mut self, nr: u8) -> Pc {
        match nr {
            0 => Pc::Monitor(11), // unused
            1 => {
                // Monitor1: Unmatched ; in definition string. Treated as (;}
                self.write_text("*nMONITOR: Unmatched semicolon in definition of ");
                self.item(self.p + 2);
                self.write_text("*nIf this had been quoted the result would be *n");
                Pc::Copy
            }
            2 => {
                // Monitor2: Unquoted ~ in argument list in input stream. Treated as <~>
                self.write_text("*nMONITOR: Unquoted tilde in argument list of ");
                self.item(self.f + 2);
                self.write_text("*nIf this had been quoted the result would be *n");
                Pc::Copy
            }
            3 => {
                // Monitor3: Impossible argument number (negative)
                self.write_text("*nMONITOR:*tImpossible argument number in definition of ");
                self.item(self.p + 2);
                Pc::Monitor(11)
            }
            4 => {
                // Monitor4: Not enough arguments supplied in call
                self.write_text("*nMONITOR: No argument ");
                self.h = 0;
                self.load(); // outputs current A (argument designator)
                self.write_text("*n in call for ");
                self.item(self.p + 2);
                Pc::Monitor(11)
            }
            5 => {
                // Monitor5: Terminator in impossible place
                self.write_text("*nMONITOR: Terminator in ");
                if self.c == 0 {
                    self.write_text("input stream. Probably machine error.");
                    Pc::Monitor(11)
                } else {
                    self.write_text("argument list for ");
                    self.item(self.f + 2);
                    self.write_text(
                        "*nProbably due to a semicolon missing from the definition of ",
                    );
                    self.item(self.p + 2);
                    self.write_text("*nIf a final semicolon is added the result is *n");
                    self.c -= 1;
                    Pc::Apply
                }
            }
            // Monitor6 not exists
            7 => {
                // Monitor7: Undefined macro name
                self.write_text("*nMONITOR: Undefined name ");
                self.item(self.w as Idx);
                Pc::Monitor(11)
            }
            8 => {
                // Monitor8: Wrong exit (not C=H=0)
                self.write_text("*nMONITOR: Unmatched >. Probably machine error. ");
                Pc::Monitor(11)
            }
            9 => {
                // Monitor9: Update string too long
                self.write_text("*nMONITOR: Update argument too long for ");
                self.item(self.p + 9);
                Pc::Monitor(11)
            }
            10 => {
                // Monitor10: Non-digit in BIN
                self.write_text("*nMONITOR: Non-digit in number ");
                Pc::Monitor(11)
            }
            11 => {
                // Monitor11: General monitor after irremediable errors.
                // W := 20
                let mut w_limit: Idx = 20;
                self.write_text("*nCurrent macros are ");

                // until P=F=0 do ...
                while !(self.p == 0 && self.f == 0) {
                    let mut w1: Idx;
                    if self.p > self.f {
                        // W1, P := P+2, ST[P]
                        w1 = self.p + 2;
                        let new_p = self.st[Self::u(self.p)] as Idx;
                        self.p = new_p;
                        self.write_text("*nAlready entered ");
                    } else {
                        // W1, F := F+2, ST[F]
                        w1 = self.f + 2;
                        let new_f = self.st[Self::u(self.f)] as Idx;
                        self.f = new_f;
                        self.write_text("*nNot yet entered ");
                    }

                    // for r = 1 to W do ...
                    for r in 1..=w_limit {
                        self.item(w1);
                        // if ST[W1]=0 do break
                        if w1 >= 0 && Self::u(w1) < self.mem_size && self.st[Self::u(w1)] == 0 {
                            break;
                        }
                        // W1 := W1 + ST[W1]
                        let step = if w1 >= 0 && Self::u(w1) < self.mem_size {
                            self.st[Self::u(w1)] as Idx
                        } else {
                            0
                        };
                        w1 = w1 + step;

                        // if ST[W1] = Marker do break
                        if w1 >= 0 && Self::u(w1) < self.mem_size && self.st[Self::u(w1)] == MARKER {
                            break;
                        }

                        // unless W=1 do Write['*nArg ', r, '*t']
                        if w_limit != 1 {
                            self.write_text(&format!("*nArg {},*t", r));
                        }
                    }

                    // W := 1
                    w_limit = 1;
                }

                self.write_text("*nEnd of monitor printing");
                self.a = 'Q' as Cell;
                self.load();

                // go to P>F -> EndFn, Start
                if self.p > self.f {
                    Pc::EndFn
                } else {
                    Pc::Start
                }
            }
            _ => unreachable!(" only 11 monitors exists"),
        }
    }

    fn run(&mut self, input: &str) -> String {
        let mut input = String::from(input);
        self.input = input.chars().rev().collect();
        self.pc = Pc::Start ;
        while self.pc != Pc::Finish
                && self.pc != Pc::NoInput {
            self.step();
        }
        match self.pc {
            _ => {}
        }
        let output = self.output.clone();
        self.output = "".to_string();
        output
    }
}

fn main() {
    let def = '&' as Cell;
    let control_chars = ControlChars{def, .. ControlChars::default() } ;
    
    let mut vm = Vm::new(control_chars,50000 );
    let o0 = vm.run("<&DEF,Suc,<&1,2,3,4,5,6,7,8,9,10,&DEF,1,<~>~1;;>;>");
    println!("o0: {}", o0);
    let o1 = vm.run("&DEF,Suc,<&1,2,3,4,5,6,7,8,9,10,&DEF,1,<~>~1;;>;");
    println!("o1: {}", o1);
    let o2 = vm.run("&Suc,7;");
    println!("o2: {}", o2);
    let o3 = vm.run("Ala ma kota mruczka.");
    println!("o3: {}", o3);
}
