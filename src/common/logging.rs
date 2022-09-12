use std::process;

pub fn pg_fatal(line: &str) -> ! {
    println!("{}", line);
    process::exit(1)
}
