#![feature(proc_macro_hygiene)]

use seq_macro::seq;

// Source of truth. Call a given macro passing nproc as argument.
macro_rules! pass_nproc {
    ($mac:ident) => {
        $mac! { 256 }
    };
}

macro_rules! literal_identity_macro {
    ($nproc:literal) => {
        $nproc
    };
}

const NPROC: usize = pass_nproc!(literal_identity_macro);

pub struct Proc;

impl Proc {
    const fn new() -> Self {
        Proc
    }
}

pub struct Mutex<T: ?Sized>(T);

impl<T> Mutex<T> {
    const fn new(_name: &'static str, value: T) -> Self {
        Mutex(value)
    }
}

macro_rules! make_procs_array {
    ($nproc:literal) => {
        seq!(N in 0..$nproc { [#(Proc::new(),)*] })
    }
}

pub static PROCS: Mutex<[Proc; NPROC]> = Mutex::new("procs", pass_nproc!(make_procs_array));
