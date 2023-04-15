/// Flush stdout.
#[macro_export]
macro_rules! fout {
    () => {
        use std::io::Write;
        std::io::stdout()
            .flush()
            .expect("Should be able to flush stdout.")
    };
}

/// Prompts the user for input with the prompt given.
#[macro_export]
macro_rules! input {
    () => {{
        let mut buf = String::new();
        std::io::stdin()
            .read_line(&mut buf)
            .expect("Should be able to read line");
        buf
    }};
    ($s:ident) => {{
        print!("{}", $s);
        fout!();
        input!()
    }};
    ($s:literal) => {{
        print!($s);
        fout!();
        input!()
    }};
    ($s:expr) => {{
        print!("{}", $s);
        fout!();
        input!()
    }};
}
