use std::sync::{Arc, Mutex};

use rkey::{CommandHandler, RespType, Storage};


#[test]
fn test_ping_cmd() {
    let d = RespType::Array(vec![
        RespType::BString("PING".to_string()),
    ]);
    let result = CommandHandler::new( Arc::new(Mutex::new(Storage::new()))).handle_cmd(d);
    assert_eq!(result.unwrap(), RespType::String("PONG".to_string()));
}

#[test]
fn test_set_cmd() {
    let d = RespType::Array(vec![
        RespType::BString("SET".to_string()),
        RespType::BString("key1".to_string()),
        RespType::BString("val".to_string()),
    ]);

    let result = CommandHandler::new( Arc::new(Mutex::new(Storage::new()))).handle_cmd(d);
    assert_eq!(result.unwrap(), RespType::String("OK".to_string()));
}

#[test]
fn test_get_cmd() {
    let storage = Arc::new(Mutex::new(Storage::new()));
    insert("key1", "val", Arc::clone(&storage));
    
    let d = RespType::Array(vec![
        RespType::BString("GET".to_string()),
        RespType::BString("key1".to_string()),
    ]);

    let result = CommandHandler::new(Arc::clone(&storage)).handle_cmd(d);
    assert_eq!(result.unwrap(), RespType::BString("val".to_string()));

}


#[test]
fn test_del() {
    let storage = Arc::new(Mutex::new(Storage::new()));
    let v = [("k1", "v1"), ("k2", "v2"), ("k3", "v3")];

    for (k, v) in v {
        insert(k, v, Arc::clone(&storage));
    }

    let keys: [RespType; 3] = v.map(|v| RespType::BString(v.0.to_string()));

    let mut resp_cmd= vec![
        RespType::BString("DEL".to_string()),
    ];
    resp_cmd.extend(keys);
    let resp_cmd =  RespType::Array(resp_cmd);
    let result = CommandHandler::new(Arc::clone(&storage)).handle_cmd(resp_cmd);

    assert_eq!(result.unwrap(), RespType::Int(v.len() as isize))

}


fn insert(k: &str, v: &str, storage: Arc<Mutex<Storage>>) -> bool {
    let d = RespType::Array(vec![
        RespType::BString("SET".to_string()),
        RespType::BString(k.to_string()),
        RespType::BString(v.to_string()),
    ]);

    let result = CommandHandler::new(storage).handle_cmd(d);
    match result {
        Ok(_) => {true},
        Err(_) => {false}
    }
}