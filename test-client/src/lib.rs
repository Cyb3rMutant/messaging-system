use std::{
    fmt::Debug,
    io::{BufRead, BufReader, Write},
    net::TcpStream,
    str::FromStr,
};

struct Client {
    id: i32,
    reader: BufReader<TcpStream>,
    writer: TcpStream,
}

impl Client {
    pub fn init() -> Client {
        let stream = TcpStream::connect("127.0.0.1:7878").unwrap();
        let writer = stream.try_clone().unwrap();
        let reader = BufReader::new(stream);

        Client {
            id: 0,
            reader,
            writer,
        }
    }

    #[allow(dead_code)]
    fn reg_and_login(&mut self, i: usize) {
        self.send(&format!("REG;USR{i};PASS{i}\n"));
        let reply = self.receive();
        check_command(&reply, "REG");

        self.send(&format!("LGN;USR{i};PASS{i}\n"));
        let reply = self.receive();
        check_command(&reply, "LGN");
        check_number_of_arguments(&reply, 3);

        self.id = extract_arg(&reply, 0);
    }

    #[allow(dead_code)]
    fn connect(&mut self, other: &mut Client) -> i32 {
        self.send(&format!("CNT;{}\n", other.id));

        let r1 = self.receive();
        let r2 = other.receive();

        let id = extract_arg(&r1, 0);
        assert_eq!(id, extract_arg(&r2, 0));
        assert_eq!(other.id, extract_arg(&r1, 1));
        assert_eq!(self.id, extract_arg(&r2, 1));

        id
    }
    #[allow(dead_code)]
    fn message(&mut self, other: &mut Client, id: i32) -> i32 {
        self.send(&format!("SND;{};hello\n", id));
        let r1 = self.receive();
        let r2 = other.receive();

        check_command(&r1, "MID");
        check_number_of_arguments(&r1, 1);
        check_command(&r2, "MSG");
        check_number_of_arguments(&r2, 3);

        let mid = extract_arg(&r1, 0);
        assert_eq!(id, extract_arg(&r2, 0));
        assert_eq!(mid, extract_arg(&r2, 1));
        assert_eq!("hello", extract_arg::<String>(&r2, 2));

        mid
    }
    #[allow(dead_code)]
    fn block(&mut self, other: &mut Client) {
        self.send(&format!("BLK;{}\n", other.id));
        let r = other.receive();
        check_command(&r, "BLK");
        assert_eq!(self.id, extract_arg(&r, 0));
    }
    #[allow(dead_code)]
    fn unblock(&mut self, other: &mut Client) {
        self.send(&format!("UBK;{}\n", other.id));
        let r = other.receive();
        check_command(&r, "ALL");
        assert_eq!(self.id, extract_arg(&r, 0));
    }

    fn send(&mut self, command: &str) {
        self.writer
            .write_all(command.as_bytes())
            .expect("Failed to send message to the server");
    }

    fn receive(&mut self) -> String {
        let mut buf = String::new();
        if self.reader.read_line(&mut buf).unwrap() == 0 {
            panic!("server died");
        }
        buf
    }

    #[allow(dead_code)]
    fn clean() {
        let mut client = Self::init();
        client.send(&format!("TESTINGCLEAR;\n"));
    }
}
#[allow(dead_code)]
fn connect() -> (Client, Client, i32) {
    let mut c1 = init();
    let mut c2 = init();

    c1.reg_and_login(1);
    c2.reg_and_login(2);

    let c = c1.connect(&mut c2);
    (c1, c2, c)
}

#[allow(dead_code)]
fn check_command(payload: &str, command: &str) {
    let (c, _) = payload.trim().split_once(';').unwrap();
    assert_eq!(command, c);
}
#[allow(dead_code)]
fn check_number_of_arguments(payload: &str, noa: usize) {
    let (_, payload) = payload.trim().split_once(';').unwrap();
    assert_eq!(noa, payload.split(';').count());
}
#[allow(dead_code)]
fn extract_arg<T: FromStr>(payload: &str, idx: usize) -> T
where
    <T as FromStr>::Err: Debug,
{
    let (_, payload) = payload.trim().split_once(';').unwrap();
    let payload: Vec<&str> = payload.split(';').collect();
    payload.get(idx).unwrap().parse::<T>().unwrap()
}

#[allow(dead_code)]
fn clean() {
    Client::clean();
}
#[allow(dead_code)]
fn init() -> Client {
    Client::init()
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn connect_once() {
        clean();
        let mut c = init();
        c.reg_and_login(1);
        clean();
    }

    #[test]
    fn connect_10_times() {
        clean();
        let mut v = Vec::new();

        for n in 0..10 {
            let mut c = init();
            c.reg_and_login(n);

            c.send(&format!("GET;ALL\n"));
            let reply = c.receive();
            check_command(&reply, "ALL");
            check_number_of_arguments(
                &reply,
                match n {
                    0 | 1 => n + 1,
                    _ => (n + 1) * 2 - 2,
                },
            );

            v.push(c);
        }
        clean();
    }

    #[test]
    fn connecting_2_users() {
        clean();
        let mut c1 = init();
        let mut c2 = init();

        c1.reg_and_login(1);
        c2.reg_and_login(2);

        c1.connect(&mut c2);

        clean();
    }
    #[test]
    fn send_message() {
        clean();
        let (mut c1, mut c2, id) = connect();

        c1.message(&mut c2, id);

        clean();
    }
    #[test]
    fn set_read() {
        clean();
        let (mut c1, mut c2, id) = connect();

        c1.message(&mut c2, id);
        c2.send(&format!("STS;{id}\n"));
        let r = c1.receive();
        check_command(&r, "STS");
        assert_eq!(id, extract_arg(&r, 0));

        clean();
    }

    #[test]
    fn delete_message() {
        clean();
        let (mut c1, mut c2, id) = connect();

        let mid = c1.message(&mut c2, id);
        let command = format!("DEL;{id};{mid}\n");
        c2.send(&command);
        let r = c1.receive();
        assert_eq!(r, command);

        clean();
    }
    #[test]
    fn edit_message() {
        clean();
        let (mut c1, mut c2, id) = connect();

        let mid = c1.message(&mut c2, id);
        let command = format!("UPD;{id};{mid};hii\n");
        c2.send(&command);
        let r = c1.receive();
        assert_eq!(r, command);

        clean();
    }
    #[test]
    fn block() {
        clean();
        let mut c1 = init();
        let mut c2 = init();
        c1.reg_and_login(1);
        c2.reg_and_login(2);
        c1.block(&mut c2);
        clean();
    }
    #[test]
    fn unblock() {
        clean();
        let mut c1 = init();
        let mut c2 = init();
        c1.reg_and_login(1);
        c2.reg_and_login(2);
        c1.block(&mut c2);
        c1.unblock(&mut c2);
        clean();
    }
}
