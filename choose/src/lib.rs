extern crate irc;
extern crate rand;

use std::io::{BufReader, BufWriter, Result};
use irc::client::conn::NetStream;
use irc::client::data::Command::PRIVMSG;
use irc::client::prelude::*;
use rand::{thread_rng, sample};

#[no_mangle]
pub fn process<'a>(server: &'a ServerExt<'a, BufReader<NetStream>, BufWriter<NetStream>>,
                   message: Message) -> Result<()> {
    process_internal(server, &message)
}

pub fn process_internal<'a, T, U>(server: &'a ServerExt<'a, T, U>, msg: &Message) -> Result<()>
    where T: IrcRead, U: IrcWrite {
    let user = msg.get_source_nickname().unwrap_or("");
    if let Ok(PRIVMSG(chan, msg)) = Command::from_message(msg) {
        if msg.starts_with("@choose ") {
            let res = sample(&mut thread_rng(), msg[8..].split(" or "), 1);
            try!(server.send_privmsg(&chan, &format!("{}: {}", user, res[0])));
        }
    }
    Ok(())
}

#[cfg(test)]
mod test {
    use std::default::Default;
    use std::io::Cursor;
    use irc::client::conn::Connection;
    use irc::client::prelude::*;

    fn test_helper(input: &str) -> String {
        let server = IrcServer::from_connection(Default::default(), Connection::new(
            Cursor::new(input.as_bytes().to_vec()), Vec::new()
        ));
        for message in server.iter() {
            let message = message.unwrap();
            println!("{:?}", message);
            super::process_internal(&server, &message).unwrap();
        }
        let vec = server.conn().writer().to_vec();
        String::from_utf8(vec).unwrap()
    }

    #[test]
    fn choose_from_two() {
        let data = test_helper(":test!test@test PRIVMSG #test :@choose this or that\r\n");
        assert!(["PRIVMSG #test :test: this\r\n", "PRIVMSG #test :test: that\r\n"]
                .contains(&&data[..]));
    }

    #[test]
    fn choose_from_three() {
        let data = test_helper(":test!test@test PRIVMSG #test :@choose this or that or the other \
                                thing\r\n");
        assert!(["PRIVMSG #test :test: this\r\n", "PRIVMSG #test :test: that\r\n",
                "PRIVMSG #test :test: the other thing\r\n"].contains(&&data[..]));
    }
}
