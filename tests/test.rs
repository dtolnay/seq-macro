use seq_macro::seq;

seq!(N in 0..8 {
    // nothing
});

#[test]
fn test_nothing() {
    macro_rules! expand_to_nothing {
        ($arg:literal) => {
            // nothing
        };
    }

    seq!(N in 0..4 {
        expand_to_nothing!(N);
    });
}

#[test]
fn test_fn() {
    seq!(N in 1..4 {
        fn f#N () -> u64 {
            N * 2
        }
    });

    // This f0 is written separately to detect whether seq correctly starts with
    // the first iteration at N=1 as specified in the invocation. If the macro
    // incorrectly started at N=0, the first generated function would conflict
    // with this one and the program would not compile.
    fn f0() -> u64 {
        100
    }

    let sum = f0() + f1() + f2() + f3();
    assert_eq!(sum, 100 + 2 + 4 + 6);
}

#[test]
fn test_enum() {
    seq!(N in 0..16 {
        #[derive(Copy, Clone, PartialEq, Debug)]
        enum Interrupt {
            #(
                Irq#N,
            )*
        }
    });

    let interrupt = Interrupt::Irq8;
    assert_eq!(interrupt as u8, 8);
    assert_eq!(interrupt, Interrupt::Irq8);
}

#[test]
fn test_inclusive() {
    seq!(N in 16..=20 {
        enum E {
            #(
                Variant#N,
            )*
        }
    });

    let e = E::Variant16;

    let desc = match e {
        E::Variant16 => "min",
        E::Variant17 | E::Variant18 | E::Variant19 => "in between",
        E::Variant20 => "max",
    };

    assert_eq!(desc, "min");
}

#[test]
fn test_array() {
    const PROCS: [Proc; 256] = {
        seq!(N in 0..256 {
            [
                #(
                    Proc::new(N),
                )*
            ]
        })
    };

    struct Proc {
        id: usize,
    }

    impl Proc {
        const fn new(id: usize) -> Self {
            Proc { id }
        }
    }

    assert_eq!(PROCS[32].id, 32);
}

#[test]
fn test_group() {
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
}
