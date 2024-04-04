use std::{
    io::{BufRead, BufReader, Write},
    net::TcpStream,
};

pub fn connect() -> (BufReader<TcpStream>, TcpStream) {
    let stream = TcpStream::connect("127.0.0.1:8080").unwrap();
    let writer = stream.try_clone().unwrap();

    let reader = BufReader::new(stream);

    (reader, writer)
}

pub fn send(writer: &mut TcpStream, command: String) {
    writer
        .write_all(command.as_bytes())
        .expect("Failed to send message to the server");
}

pub fn receive(reader: &mut BufReader<TcpStream>) -> String {
    let mut buf = String::new();
    if reader.read_line(&mut buf).unwrap() == 0 {
        panic!("server died");
    }

    buf
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn connect_once() {
        let (mut reader, mut writer) = connect();

        // send(&mut writer, format!("REG;USR1;PASS1\n"));
        // println!("{:?}", receive(&mut reader));
        send(&mut writer, format!("TESTINGCLEAR;\n"));
    }
    #[test]
    fn connect_10_times() {
        let mut v = Vec::new();

        for n in 0..10 {
            let (mut reader, mut writer) = connect();
            send(&mut writer, format!("REG;USR{};PASS{}\n", n, n));
            println!("{:?}", receive(&mut reader));
            send(&mut writer, format!("LGN;USR{};PASS{}\n", n, n));
            println!("{:?}", receive(&mut reader));
            send(&mut writer, format!("GET;\n"));
            println!("{:?}", receive(&mut reader));
            v.push((reader, writer));
        }
        // sleep(Duration::from_secs(1));
    }
}
