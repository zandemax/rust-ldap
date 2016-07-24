extern crate ldap_protocol as protocol;

use std::net::TcpStream;
use std::net::ToSocketAddrs;

use std::io::{Read, Write};

use protocol::ber::{self, common};
use protocol::Result;

#[derive(Debug)]
pub struct LDAP
{
    // TODO: Later abstract over io::Read / io::Write
    stream: TcpStream,

    msgid: i32,
}

impl LDAP
{
    pub fn connect<A: ToSocketAddrs>(addr: A) -> Result<LDAP>
    {
        let stream = try!(TcpStream::connect(addr));
        stream.set_read_timeout(None);

        Ok(LDAP
        {
            stream: stream,
            msgid: 0,
        })
    }

    pub fn send(&mut self, tag: common::Tag) -> Result<()>
    {
        println!("Sending tag: {:?}", tag);
        let tagbuf = try!(ber::encode(tag, self.msgid));
        try!(self.stream.write(tagbuf.as_slice()));

        Ok(())
    }

    pub fn recv(&mut self) -> Result<common::Tag>
    {
        let mut buf = [0; 500];

        let readamount = try!(self.stream.read(&mut buf));
        println!("read: {}", readamount);

        let tag = try!(ber::decode(&mut buf));
        println!("Received tag: {:?}", tag);

        Ok(tag)
    }
}