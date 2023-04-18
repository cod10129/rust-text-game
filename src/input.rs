/// Flush stdout.
#[macro_export]
macro_rules! fout {
    () => {
        #[allow(unused_imports)]
        use std::io::Write as _;
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
    ($s:expr) => {{
        print!("{}", $s);
        fout!();
        input!()
    }};
}
