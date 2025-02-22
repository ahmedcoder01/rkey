use std::collections::LinkedList;
use std::error::Error;
use std::io::{BufRead, Cursor, Read, Seek};
use std::str::FromStr;

#[derive(Debug, PartialEq)]
pub enum RespType {
    BString(String),
    String(String),
    Err(String),
    Int(isize),
    Array(Vec<RespType>),
    Null
}

impl RespType {
    pub fn serialize(&self) -> String {
        match self {
            RespType::BString(s) => {
                let len = s.len();
                format!("${}\r\n{}\r\n", len, s)
            },
            RespType::String(s) => {
                format!("+{}\r\n", s)
            },
            RespType::Err(e) => {
                format!("-{}\r\n", e)
            },
            RespType::Int(i) => {
                format!(":{}\r\n", i)
            },
            RespType::Array(vec) => {
                let serialized_simple: String = vec.iter().map(|s| {
                    s.serialize()
                }).collect();
                let agg_len = vec.len();
                format!("*{}\r\n{}", agg_len, serialized_simple)
            },
            RespType::Null => {
                format!("$-1\r\n")
            },
        }
    }
} 

pub struct Resp {}

enum ProcessMode {
    Collect,
    Return,
}

impl Resp {
    pub fn new() -> Self {
        Self {}
    }

    pub fn parse_line(&self, s: &str) -> Result<RespType, Box<dyn Error>> {
        let mut mode = ProcessMode::Return;
        let mut c = Cursor::new(s.as_bytes());
        struct AggStack {
            i: u8,
            len: u8,
            agg: RespType,
        }
        let mut agg_stack: LinkedList<AggStack> = LinkedList::new();

        loop {
            if (c.position() as usize) >= c.get_ref().len() {
                break;
            };
            let mut next_byte = [0u8; 1]; // Buffer to hold a single byte
            c.read_exact(&mut next_byte)?; // Read the first byte
            let mut curr_type: Option<RespType> = None;
            println!("next byte: {:?}", char::from_u32(next_byte[0] as u32));
            
            match next_byte[0] {
                b'*' => {
                    println!("processing arr");
                    let len: u8 = self.read_line_and_delim(&mut c)?;
                    agg_stack.push_front(AggStack {
                        i: 0,
                        len,
                        agg: RespType::Array(Vec::new()),
                    });
                    println!("added new agg collector");
                    mode = ProcessMode::Collect;
                }

                b'+' => {
                    println!("processing simple string");
                    curr_type = Some(RespType::String(self.read_line_and_delim(&mut c)?));
                }

                b'$' => {
                    let length: isize = self.read_line_and_delim(&mut c)?;
                    println!("length of bulk string: {}", length);
                    if length == -1 { // indicates a null type
                        println!("processing null");
                        curr_type = Some(RespType::Null);
                    } else {
                        println!("processing bulk string");
                        let mut s = vec![0u8; length as usize];
                        c.read_exact(&mut s)?;

                        {
                            let mut t = String::new();
                            c.read_line(&mut t)?; // skips the delimiter after string
                        }

                        curr_type = Some(RespType::BString(String::from_utf8(s).unwrap()));
                    }
                }

                b':' => {
                    println!("processing integer");
                    curr_type = Some(RespType::Int(self.read_line_and_delim(&mut c)?))
                }

                // b'-' => {
                //
                // }
                _ => {
                    // Err(Box::from("Failed to parse"))
                    println!("failed to parse type.... aborting parser...");
                    break;
                }
            }

            if let ProcessMode::Return = mode {
                return if let Some(t) = curr_type {
                    Ok(t)
                } else {
                    Err(Box::new(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        "Failed to parse type",
                    )))
                };
            } else {
                let curr_agg = agg_stack.front_mut().unwrap();

                if let Some(t) = curr_type {
                    if let RespType::Array(v) = &mut curr_agg.agg {
                        curr_agg.i += 1;
                        println!("added value to agg collector {:?}", t);
                        v.push(t);
                    }
                }

                // check if the current array loop is finished
                if curr_agg.i == curr_agg.len {
                    if agg_stack.len() == 1 {
                        // we finished the main agg
                        break;
                    } else {
                        // we finished processing a nested agg
                        let curr_agg = agg_stack.pop_front();
                        let prev_agg = agg_stack.front_mut().unwrap();

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

    fn read_line_and_delim<T: FromStr>(&self, c: &mut Cursor<&[u8]>) -> Result<T, Box< dyn Error>>
    where
        <T as FromStr>::Err: std::fmt::Debug,
    {
        let mut line = String::new();
        c.read_line(&mut line)
        .map_err(|e| Box::new(e) as Box<dyn Error>)?;

        println!("line: {}", line.escape_debug());

        // Trim and parse, converting parse error into a string
        line.trim_end_matches("\r\n")
            .parse::<T>()
            .map_err(|e| Box::<dyn Error>::from(format!("Parse error: {:?}", e))) // Convert parse error to a string
    }
}

