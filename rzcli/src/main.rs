use std::env;
use std::io::*;
use mio_uds::UnixStream;

fn main() {
    let mut path = env::temp_dir();
    path.push("rzrtd.cli");

    let mut stream = match UnixStream::connect(path) {
        Ok(mut stream) => stream,
        Err(_) => panic!("Error: cannot connect to Rzrtd")
    };

    loop {
        stdout().write(b"> ");
        stdout().flush();

        let mut buffer = String::new();
        stdin().read_line(&mut buffer);

        stream.write(buffer.as_ref());
        stream.flush();
    }
}
