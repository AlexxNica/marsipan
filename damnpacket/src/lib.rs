#[macro_use]
extern crate nom;

use nom::*;

type Bytes = Vec<u8>;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Message {
    pub name: Bytes,
    pub argument: Option<Bytes>,
    pub attrs: Vec<(Bytes, String)>,
    pub body: Option<MessageBody>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MessageBody(Bytes);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SubMessage {
    pub name: Option<Bytes>,
    pub argument: Option<Bytes>,
    pub attrs: Vec<(Bytes, String)>,
    pub body: Option<MessageBody>,
}

impl MessageBody {
    pub fn submessage(&self) -> Result<SubMessage, nom::ErrorKind> {
        parse_submessage(self.0.as_slice()).to_result()
    }
}

fn attr(input: &[u8]) -> IResult<&[u8], (Bytes, String)> {
    let (i1, key) = try_parse!(input, alphanumeric);
    let (i2, _) = try_parse!(i1, tag!("="));
    let (i3, val) = try_parse!(i2, take_until!("\n"));
    let (i4, _) = try_parse!(i3, tag!("\n"));
    IResult::Done(i4,
                  (key.to_vec(), String::from_utf8(val.to_vec()).expect("not UTF8")))
}

fn pbody(input: &[u8]) -> IResult<&[u8], Option<MessageBody>> {
    let (i1, next) = try_parse!(input, be_u8);
    fn is0(x: u8) -> bool {
        x == 0
    }
    match next {
        b'\n' => {
            let (i2, body) = try_parse!(i1, take_till!(is0));
            let (i3, b0) = try_parse!(i2, tag!("\0"));
            let mut bvec = body.to_vec();
            bvec.extend(b0);
            return IResult::Done(i3, Some(MessageBody(bvec)));
        }
        _ => IResult::Done(i1, None),
    }
}

fn parse_message(input: &[u8]) -> IResult<&[u8], Message> {
    let (i1, nm) = try_parse!(input, alpha);
    let (i2, arg) = match i1[0] {
        b' ' => {
            let (a, b) = try_parse!(i1, take_until1!("\n"));
            (a, Some(&b[1..]))
        }
        _ => (i1, None),
    };
    let (i3, _) = try_parse!(i2, tag!("\n"));
    let (i4, attrs) = try_parse!(i3, many0!(attr));
    let (i5, body) = try_parse!(i4, pbody);
    IResult::Done(i5,
                  Message {
                      name: nm.to_vec(),
                      argument: arg.map(|x| x.to_vec()),
                      attrs: attrs,
                      body: body,
                  })
}

fn parse_submessage(input: &[u8]) -> IResult<&[u8], SubMessage> {
    let (i1, ar) = try_parse!(input, opt!(attr));
    match ar {
        Some(pair) => {
            let (i2, more) = try_parse!(i1, many0!(attr));
            let (i3, body) = try_parse!(i2, pbody);
            let mut my_attrs = vec![];
            my_attrs.push(pair);
            my_attrs.extend(more);
            IResult::Done(i3,
                          SubMessage {
                              name: None,
                              argument: None,
                              attrs: my_attrs,
                              body: body,
                          })
        }
        None => {
            let (i2, msg) = try_parse!(i1, parse_message);
            IResult::Done(i2,
                          SubMessage {
                              name: Some(msg.name),
                              argument: msg.argument,
                              attrs: msg.attrs,
                              body: msg.body,
                          })
        }
    }
}

pub fn parse<'a>(bs: Vec<u8>) -> Result<Message, nom::ErrorKind> {
    let res = parse_message(&bs[..]);
    println!("{:?}", res);
    res.to_result()
}

#[test]
fn parse_basic() {
    assert_eq!(parse(b"foo bar\nbaz=qux\n\nthis is the body\0".to_vec()),
               Ok(Message {
                      name: b"foo".to_vec(),
                      argument: Some(b"bar".to_vec()),
                      attrs: vec![(b"baz".to_vec(), String::from("qux"))],
                      body: Some(MessageBody(b"this is the body\0".to_vec())),
                  }));
}

#[test]
fn parse_no_body() {
    assert_eq!(parse(b"foo bar\nbaz=qux\n\0".to_vec()),
               Ok(Message {
                      name: b"foo".to_vec(),
                      argument: Some(b"bar".to_vec()),
                      attrs: vec![(b"baz".to_vec(), String::from("qux"))],
                      body: None,
                  }));
}

#[test]
fn parse_no_attrs() {
    assert_eq!(parse(b"foo bar\n\0".to_vec()),
               Ok(Message {
                      name: b"foo".to_vec(),
                      argument: Some(b"bar".to_vec()),
                      attrs: vec![],
                      body: None,
                  }));
}

#[test]
fn parse_no_arg() {
    assert_eq!(parse(b"foo\n\0".to_vec()),
               Ok(Message {
                      name: b"foo".to_vec(),
                      argument: None,
                      attrs: vec![],
                      body: None,
                  }));
}

#[test]
fn parse_sub() {
    let msg = parse(b"foo\n\na=b\nc=d\n\0".to_vec())
        .expect("oh no")
        .body
        .expect("no body");
    assert_eq!(msg.submessage(),
               Ok(SubMessage {
                      name: None,
                      argument: None,
                      attrs: vec![(b"a".to_vec(), String::from("b")),
                                  (b"c".to_vec(), String::from("d"))],
                      body: None,
                  }))
}
