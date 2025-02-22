use rkey::{Resp, RespType};

/*
    Each test should include the type serialization & deserialization
*/


#[test]
fn test_simple_string() {
    let resp_parser = Resp::new();
    let resp_str = "+OK\r\n";
    let des: RespType = RespType::String("OK".to_string());
    assert_eq!(des.serialize(), resp_str);

    let parsed = resp_parser.parse_line(resp_str).unwrap();
    assert_eq!(parsed, des);
}

#[test]
fn test_bulk_string() {
    let resp_parser = Resp::new();
    let resp_str = "$5\r\nhello\r\n";
    let des = RespType::BString("hello".to_string());
    assert_eq!(des.serialize(), resp_str);
    let parsed = resp_parser.parse_line(resp_str).unwrap();
    assert_eq!(parsed, des);
}

#[test]
fn test_integer() {
    let resp_parser = Resp::new();

    let resp = ":5\r\n";
    let parsed = resp_parser.parse_line(resp).unwrap();
    let des = RespType::Int(5);
    assert_eq!(des.serialize(), resp);
    assert_eq!(parsed, des);

    let resp = ":+5\r\n";
    let des = RespType::Int(5);
    let parsed = resp_parser.parse_line(resp).unwrap();
    assert_eq!(parsed, des);


    let resp =":-5\r\n";
    let parsed = resp_parser.parse_line(resp).unwrap();
    let des = RespType::Int(-5);
    assert_eq!(parsed, des);
    assert_eq!(des.serialize(), resp);
    assert_eq!(parsed, des);
}

#[test]
fn test_array() {
    let resp_parser = Resp::new();

    let resp_str = "*2\r\n$5\r\nhello\r\n:223\r\n";
    let parsed = resp_parser.parse_line(&resp_str).unwrap();
    let des = RespType::Array(vec![
        RespType::BString("hello".to_string()),
        RespType::Int(223)
    ]);
    assert_eq!(des.serialize(), resp_str);

    assert_eq!(
        parsed,
        des
    );
}

#[test]
fn test_nested_arrays() {
    let resp_parser = Resp::new();
    let resp_str = "*3\r\n+Hello\r\n*3\r\n:1\r\n:2\r\n:3\r\n*2\r\n+Hello\r\n+World\r\n";
    let parsed = resp_parser.parse_line(&resp_str).unwrap();
    let des: RespType = RespType::Array(vec![
        RespType::String("Hello".to_string()),
        RespType::Array(vec![RespType::Int(1), RespType::Int(2), RespType::Int(3)]),
        RespType::Array(vec![
            RespType::String("Hello".to_string()),
            RespType::String("World".to_string())
        ])
    ]);
    assert_eq!(des.serialize(), resp_str);
    assert_eq!(
        parsed,
        des
    )
}


#[test] 
fn test_null() {
    let resp_parser = Resp::new();
    let resp_str = "$-1\r\n";
    let parsed = resp_parser.parse_line(&resp_str).unwrap();
    let des: RespType = RespType::Null;
    assert_eq!(des.serialize(), resp_str);
    assert_eq!(
        parsed,
        des
    )
}