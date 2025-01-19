use std::collections::LinkedList;
use std::error::Error;
use std::io::{BufRead, BufReader, Cursor, Read};
use std::str::FromStr;

#[derive(Debug, PartialEq)]
 enum RespType {
    BString(String),
    String(String),
    Err(String),
    Int(isize),
    Array(Vec<RespType>)
}

pub struct Resp {
}

enum ProcessMode {
    COLLECT,
    RETURN
}


impl Resp {

    pub fn new() -> Self {
        Self {}
    }

    pub fn parse_line(&self, s: &str) -> Result<RespType, Box<dyn Error>> {
        let mut mode = ProcessMode::RETURN;
        let mut c = Cursor::new(s.as_bytes());
        struct AggStack { i: u8, len: u8, agg: RespType }
        let mut agg_stack: LinkedList<AggStack> = LinkedList::new();




        loop {
            if (c.position() as usize) >= c.get_ref().len() {break};
            let mut next_byte = [0u8; 1]; // Buffer to hold a single byte
            c.read_exact(&mut next_byte)?; // Read the first byte
            let mut curr_type: Option<RespType> = None;
            println!("next byte: {:?}", char::from_u32(next_byte[0] as u32));
            match next_byte[0] {
                b'*' => {
                    println!("processing arr");
                    let len: u8 = self.read_line_and_trim(&mut c);
                    agg_stack.push_front(AggStack {  i: 0, len, agg: RespType::Array(Vec::new()) });
                    println!("added new agg collector");
                    mode = ProcessMode::COLLECT;
                }

                b'+' => {
                    println!("processing simple string");
                    curr_type = Some(RespType::String(self.read_line_and_trim(&mut c)));

                }

                b'$' => {
                    println!("processing bulk string");
                    let length = self.read_line_and_trim(&mut c);
                    let mut s = vec![0u8; length];
                    c.read_exact(&mut s)?;

                    {
                        let mut t = String::new();
                        c.read_line(&mut t)?; // skips the delimiter after string
                    }



                    curr_type = Some(RespType::BString(String::from_utf8(s).unwrap()));


                }

                b':' => {
                    println!("processing integer");
                    curr_type = Some(RespType::Int(self.read_line_and_trim(&mut c)))

                }

                // b'-' => {
                //
                // }
                _ => {
                    // Err(Box::from("Failed to parse"))
                    println!("failed to parse type.... terminating...");
                    break;
                }

            }

            if let ProcessMode::RETURN = mode {

                return if let Some(t) = curr_type {
                    Ok(t)
                } else {
                    Err(Box::new(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        "Failed to parse type",
                    )))
                }
            } else {
                let mut curr_agg = agg_stack.front_mut().unwrap();


                if let Some(t) = curr_type {
                    if let RespType::Array(v) = &mut curr_agg.agg {
                        curr_agg.i += 1;
                        println!("added value to agg collector {:?}", t);
                        v.push(t);
                    }
                }

                // check if the current array loop is finished
                if (curr_agg.i == curr_agg.len) {

                    if (agg_stack.len() == 1) { // we finished the main agg
                        break;
                    } else  { // we finished processing a nested agg
                        let curr_agg =  agg_stack.pop_front();
                        let prev_agg =  agg_stack.front_mut().unwrap();

                        if let RespType::Array(vec) = &mut prev_agg.agg {
                            vec.push(curr_agg.unwrap().agg);
                            prev_agg.i += 1;
                        }


                    }
                    // if le
                }
            }
        }

        Ok(agg_stack.pop_front().unwrap().agg)
    }

    fn read_line_and_trim<T: FromStr>(&self, c: &mut Cursor<&[u8]>) -> T
    where
        <T as FromStr>::Err: std::fmt::Debug,
    {
        let mut line = String::new();
        c.read_line(&mut line).expect("failed to read line");
        line.trim_end_matches("\r\n").parse().unwrap()
    }
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_string() {
        let resp_parser = Resp::new();
        let resp_str = "+OK\r\n";
        let parsed = resp_parser.parse_line(&resp_str).unwrap();
        assert_eq!(parsed, RespType::String("OK".to_string()));
    }

    #[test]
    fn test_bulk_string() {
        let resp_parser = Resp::new();
        let resp_str = "$5\r\nhello\r\n";
        let parsed = resp_parser.parse_line(&resp_str).unwrap();
        assert_eq!(parsed, RespType::BString("hello".to_string()));
    }

    #[test]
    fn test_integer() {
        let resp_parser = Resp::new();
        let resp_str_no_sign = ":5\r\n";
        let resp_str_signed_pos = ":+5\r\n";
        let resp_str_signed_neg = ":-5\r\n";
        let parsed = resp_parser.parse_line(&resp_str_no_sign).unwrap();
        assert_eq!(parsed, RespType::Int(5));

        let parsed = resp_parser.parse_line(&resp_str_signed_pos).unwrap();
        assert_eq!(parsed, RespType::Int(5));

        let parsed = resp_parser.parse_line(&resp_str_signed_neg).unwrap();
        assert_eq!(parsed, RespType::Int(-5));
    }

    #[test]
    fn test_array() {
        let resp_parser = Resp::new();


        let resp_str_2item = "*2\r\n$5\r\nhello\r\n:223\r\n";
        let parsed = resp_parser.parse_line(&resp_str_2item).unwrap();
        assert_eq!(parsed, RespType::Array(vec![RespType::BString("hello".to_string()), RespType::Int(223)]));

    }

    #[test]
    fn test_nested_arrays() {
        let resp_parser = Resp::new();
        let resp_str = "*3\r\n+Hello\r\n*3\r\n:1\r\n:2\r\n:3\r\n*2\r\n+Hello\r\n+World\r\n";
        let parsed = resp_parser.parse_line(&resp_str).unwrap();
        assert_eq!(parsed, RespType::Array(vec![
            RespType::String("Hello".to_string()),
            RespType::Array(vec![RespType::Int(1), RespType::Int(2), RespType::Int(3)]),
            RespType::Array(vec![RespType::String("Hello".to_string()), RespType::String("World".to_string())])
        ]))

    }

}