use std::io;
use std::io::Error;
use std::net::SocketAddr;
use std::fmt::{Display, Formatter};

use futures::stream::{SplitSink, SplitStream};
use futures::Stream;

use tokio::reactor::Handle;
use tokio::codec::{Decoder, Encoder};
use tokio::net::{UdpSocket, UdpFramed};

use bytes::{Bytes, BytesMut};


#[derive(Debug)]
pub struct DnsCodec;

pub enum DnsParserError {
    TooLittleData,
    TooMuchData,
}

impl Display for DnsParserError {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        use self::DnsParserError::*;
        match self {
            TooLittleData => write!(f, "TooLittleData"),
            TooMuchData => write!(f, "TooMuchData"),
        }
    }
}

#[derive(Copy, Clone)]
pub enum UdpListenSocket {
    Addr(SocketAddr),
    Activation
}

impl Display for UdpListenSocket {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        use self::UdpListenSocket::*;
        match self {
            Addr(socket_addr) => write!(f, "{}", socket_addr),
            Activation => write!(f, "file descriptor 3"),
        }
    }
}

pub struct DnsPacket {
    data: Bytes,
    tid: [u8;2],
    response: bool,
    questions: u16,
    answer: u16,
    authority: u16,
    additional_records: u16,
}

impl DnsPacket {
    pub fn from(buffer: Bytes) -> Result<DnsPacket, DnsParserError> {
        DnsPacket::parser(buffer)
    }

    pub fn from_tid(buffer: Bytes, tid: [u8;2]) -> Result<DnsPacket, DnsParserError> {
        let mut buffer = BytesMut::from(buffer);
        buffer[0] = tid[0];
        buffer[1] = tid[1];

        DnsPacket::parser(buffer.freeze())
    }

    fn parser(buffer: Bytes) -> Result<DnsPacket, DnsParserError> {
        let len = buffer.len();

        if len < 12 {
            return Err(DnsParserError::TooLittleData);
        } else if 512 < len {
            return Err(DnsParserError::TooMuchData);
        }

        let response = (buffer[2] & 0x80) == 0x80;

        let mut tid: [u8;2] = [0;2];
        tid.copy_from_slice(&buffer[0..2]);

        let questions: u16 = ((buffer[4] as u16) << 8) | (buffer[5] as u16);
        let answer: u16 = ((buffer[6] as u16) << 8) | (buffer[7] as u16);
        let authority: u16 = ((buffer[8] as u16) << 8) | (buffer[9] as u16);
        let additional_records: u16 = ((buffer[10] as u16) << 8) | (buffer[11] as u16);

        Ok(DnsPacket{data: buffer, tid, response, questions, answer, authority, additional_records})
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn get_without_tid(&self) -> Bytes {
        let mut data: BytesMut = BytesMut::with_capacity(self.data.len());
        data.extend(self.data.iter());
        data[0] = b'\0';
        data[1] = b'\0';

        data.freeze()
    }

    pub fn get(&self) -> Bytes {
        self.data.clone()
    }

    pub fn get_tid(&self) -> [u8;2] {
        self.tid.clone()
    }

    pub fn is_response(&self) -> bool {
        self.response
    }

    pub fn get_questions(&self) -> u16 {
        self.questions
    }

    pub fn get_answer(&self) -> u16 {
        self.answer
    }

    pub fn get_authority(&self) -> u16 {
        self.authority
    }

    pub fn get_additional_records(&self) -> u16 {
        self.additional_records
    }
}

impl DnsCodec {
    pub fn new(listen: UdpListenSocket) -> Result<(SplitSink<UdpFramed<DnsCodec>>, SplitStream<UdpFramed<DnsCodec>>), Error> {
        use self::UdpListenSocket::*;
        let socket = match listen {
            Addr(socket_addr) => match UdpSocket::bind(&socket_addr) {
                Ok(socket) => socket,
                Err(e) => return Err(e)
            },
            Activation => {
                use std::net;
                use std::os::unix::io::FromRawFd;
                unsafe {
                    match UdpSocket::from_std(net::UdpSocket::from_raw_fd(3), &Handle::current()) {
                        Ok(socket) => socket,
                        Err(e) => return Err(e)
                    }
                }
            }
        };
        Ok(UdpFramed::new(socket, DnsCodec).split())
    }
}

impl Decoder for DnsCodec {
    type Item = DnsPacket;
    type Error = io::Error;

    fn decode(&mut self, buf: &mut BytesMut) -> Result<Option<DnsPacket>, io::Error> {
        if let Ok(dns) = DnsPacket::from(buf.clone().freeze()) {
            if dns.is_response() == false && dns.get_questions() > 0 {
                return Ok(Some(dns))
            }
        }

        buf.clear();
        Ok(None)
    }
}

impl Encoder for DnsCodec {
    type Item = DnsPacket;
    type Error = io::Error;

    fn encode(&mut self, data: DnsPacket, buf: &mut BytesMut) -> Result<(), io::Error> {
        buf.clear();
        buf.reserve(data.len());
        let b: Bytes = data.get();
        buf.extend(b);
        Ok(())
    }
}