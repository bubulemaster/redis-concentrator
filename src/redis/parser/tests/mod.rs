use crate::redis::parser::{read_array, read_bulk_string, read_integer, read_strict_string};
use crate::redis::stream::tests::TestRedisStream;
use crate::redis::stream::RedisStream;
use crate::redis::types::{
    ErrorKind, RedisError, RedisValue, REDIS_TYPE_ARRAY, REDIS_TYPE_BULK_STRING, REDIS_TYPE_ERROR,
    REDIS_TYPE_INTEGER, REDIS_TYPE_STRING,
};

#[test]
fn read_strict_string_ok() -> Result<(), RedisError> {
    let stream = TestRedisStream::new(vec![
        REDIS_TYPE_STRING,
        'H' as u8,
        'e' as u8,
        'l' as u8,
        'l' as u8,
        'o' as u8,
        '\r' as u8,
        '\n' as u8,
    ]);
    let mut box_stream: Box<dyn RedisStream> = Box::new(stream);

    assert_eq!(read_strict_string(&mut box_stream)?, "Hello");

    Ok(())
}

#[test]
fn read_strict_string_ko() {
    let stream = TestRedisStream::new(vec![]);
    let mut box_stream: Box<dyn RedisStream> = Box::new(stream);

    match read_strict_string(&mut box_stream) {
        Ok(_) => panic!("Must be return error!"),
        Err(e) => {
            assert_eq!(e.kind(), ErrorKind::IoError);
            assert_eq!(e.io_error_kind().unwrap(), std::io::ErrorKind::BrokenPipe);
        }
    }
}

#[test]
fn read_strict_string_bad_type() {
    let stream = TestRedisStream::new(vec![
        REDIS_TYPE_ARRAY,
        'H' as u8,
        'e' as u8,
        'l' as u8,
        'l' as u8,
        'o' as u8,
        '\r' as u8,
        '\n' as u8,
    ]);
    let mut box_stream: Box<dyn RedisStream> = Box::new(stream);

    match read_strict_string(&mut box_stream) {
        Ok(_) => panic!("Must be return error!"),
        Err(e) => {
            assert_eq!(e.kind(), ErrorKind::OtherError);
            assert_eq!(e.message(), String::from("Not a string but a Array"));
        }
    }
}

#[test]
fn read_strict_string_type_unknow() {
    let stream = TestRedisStream::new(vec![
        '.' as u8, 'H' as u8, 'e' as u8, 'l' as u8, 'l' as u8, 'o' as u8, '\r' as u8, '\n' as u8,
    ]);
    let mut box_stream: Box<dyn RedisStream> = Box::new(stream);

    match read_strict_string(&mut box_stream) {
        Ok(_) => panic!("Must be return error!"),
        Err(e) => {
            assert_eq!(e.kind(), ErrorKind::OtherError);
            assert_eq!(
                e.message(),
                String::from("Not a string but a Unknow '0x46'")
            );
        }
    }
}

#[test]
fn read_strict_string_error() {
    let stream = TestRedisStream::new(vec![
        REDIS_TYPE_ERROR,
        'H' as u8,
        'e' as u8,
        'l' as u8,
        'l' as u8,
        'o' as u8,
        '\r' as u8,
        '\n' as u8,
    ]);
    let mut box_stream: Box<dyn RedisStream> = Box::new(stream);

    match read_strict_string(&mut box_stream) {
        Ok(_) => panic!("Must be return error!"),
        Err(e) => {
            assert_eq!(e.kind(), ErrorKind::OtherError);
            assert_eq!(e.message(), "Hello");
        }
    }
}

#[test]
fn read_integer_ok() -> Result<(), RedisError> {
    let stream = TestRedisStream::new(vec![
        REDIS_TYPE_INTEGER,
        '1' as u8,
        '2' as u8,
        '\r' as u8,
        '\n' as u8,
    ]);
    let mut box_stream: Box<dyn RedisStream> = Box::new(stream);

    assert_eq!(read_integer(&mut box_stream)?, 12);

    Ok(())
}

#[test]
fn read_integer_ko() {
    let stream = TestRedisStream::new(vec![]);
    let mut box_stream: Box<dyn RedisStream> = Box::new(stream);

    match read_integer(&mut box_stream) {
        Ok(_) => panic!("Must be return error!"),
        Err(e) => {
            assert_eq!(e.kind(), ErrorKind::IoError);
            assert_eq!(e.io_error_kind().unwrap(), std::io::ErrorKind::BrokenPipe);
        }
    }
}

#[test]
fn read_integer_bad_type() {
    let stream = TestRedisStream::new(vec![
        REDIS_TYPE_ARRAY,
        'H' as u8,
        'e' as u8,
        'l' as u8,
        'l' as u8,
        'o' as u8,
        '\r' as u8,
        '\n' as u8,
    ]);
    let mut box_stream: Box<dyn RedisStream> = Box::new(stream);

    match read_integer(&mut box_stream) {
        Ok(_) => panic!("Must be return error!"),
        Err(e) => {
            assert_eq!(e.kind(), ErrorKind::OtherError);
            assert_eq!(e.message(), String::from("Not an integer but a Array"));
        }
    }
}

#[test]
fn read_integer_type_unknow() {
    let stream = TestRedisStream::new(vec![
        '.' as u8, 'H' as u8, 'e' as u8, 'l' as u8, 'l' as u8, 'o' as u8, '\r' as u8, '\n' as u8,
    ]);
    let mut box_stream: Box<dyn RedisStream> = Box::new(stream);

    match read_integer(&mut box_stream) {
        Ok(_) => panic!("Must be return error!"),
        Err(e) => {
            assert_eq!(e.kind(), ErrorKind::OtherError);
            assert_eq!(
                e.message(),
                String::from("Not an integer but a Unknow '0x46'")
            );
        }
    }
}

#[test]
fn read_integer_error() {
    let stream = TestRedisStream::new(vec![
        REDIS_TYPE_ERROR,
        'E' as u8,
        'R' as u8,
        'R' as u8,
        ' ' as u8,
        'H' as u8,
        'e' as u8,
        'l' as u8,
        'l' as u8,
        'o' as u8,
        '\r' as u8,
        '\n' as u8,
    ]);
    let mut box_stream: Box<dyn RedisStream> = Box::new(stream);

    match read_integer(&mut box_stream) {
        Ok(_) => panic!("Must be return error!"),
        Err(e) => {
            assert_eq!(e.kind(), ErrorKind::ResponseError);
            assert_eq!(e.message(), " Hello");
        }
    }
}

#[test]
fn read_bulk_string_ok() -> Result<(), RedisError> {
    let stream = TestRedisStream::new(vec![
        REDIS_TYPE_BULK_STRING,
        '5' as u8,
        '\r' as u8,
        '\n' as u8,
        'H' as u8,
        'e' as u8,
        'l' as u8,
        'l' as u8,
        'o' as u8,
        '\r' as u8,
        '\n' as u8,
    ]);
    let mut box_stream: Box<dyn RedisStream> = Box::new(stream);

    assert_eq!(
        read_bulk_string(&mut box_stream)?.unwrap(),
        vec!['H' as u8, 'e' as u8, 'l' as u8, 'l' as u8, 'o' as u8]
    );

    Ok(())
}

#[test]
fn read_bulk_string_ko() {
    let stream = TestRedisStream::new(vec![]);
    let mut box_stream: Box<dyn RedisStream> = Box::new(stream);

    match read_bulk_string(&mut box_stream) {
        Ok(_) => panic!("Must be return error!"),
        Err(e) => {
            assert_eq!(e.kind(), ErrorKind::IoError);
            assert_eq!(e.io_error_kind().unwrap(), std::io::ErrorKind::BrokenPipe);
        }
    }
}

#[test]
fn read_bulk_string_type() {
    let stream = TestRedisStream::new(vec![
        REDIS_TYPE_ARRAY,
        'H' as u8,
        'e' as u8,
        'l' as u8,
        'l' as u8,
        'o' as u8,
        '\r' as u8,
        '\n' as u8,
    ]);
    let mut box_stream: Box<dyn RedisStream> = Box::new(stream);

    match read_bulk_string(&mut box_stream) {
        Ok(_) => panic!("Must be return error!"),
        Err(e) => {
            assert_eq!(e.kind(), ErrorKind::OtherError);
            assert_eq!(e.message(), String::from("Not a bulk string but a Array"));
        }
    }
}

#[test]
fn read_bulk_string_type_unknow() {
    let stream = TestRedisStream::new(vec![
        '.' as u8, 'H' as u8, 'e' as u8, 'l' as u8, 'l' as u8, 'o' as u8, '\r' as u8, '\n' as u8,
    ]);
    let mut box_stream: Box<dyn RedisStream> = Box::new(stream);

    match read_bulk_string(&mut box_stream) {
        Ok(_) => panic!("Must be return error!"),
        Err(e) => {
            assert_eq!(e.kind(), ErrorKind::OtherError);
            assert_eq!(
                e.message(),
                String::from("Not a bulk string but a Unknow '0x46'")
            );
        }
    }
}

#[test]
fn read_bulk_string_error() {
    let stream = TestRedisStream::new(vec![
        REDIS_TYPE_ERROR,
        'E' as u8,
        'X' as u8,
        'E' as u8,
        'C' as u8,
        'A' as u8,
        'B' as u8,
        'O' as u8,
        'R' as u8,
        'T' as u8,
        ' ' as u8,
        'H' as u8,
        'e' as u8,
        'l' as u8,
        'l' as u8,
        'o' as u8,
        '\r' as u8,
        '\n' as u8,
    ]);
    let mut box_stream: Box<dyn RedisStream> = Box::new(stream);

    match read_bulk_string(&mut box_stream) {
        Ok(_) => panic!("Must be return error!"),
        Err(e) => {
            assert_eq!(e.kind(), ErrorKind::ExecAbortError);
            assert_eq!(e.message(), " Hello");
        }
    }
}

#[test]
fn read_array_ok() -> Result<(), RedisError> {
    let stream = TestRedisStream::new(vec![
        REDIS_TYPE_ARRAY,
        '2' as u8,
        '\r' as u8,
        '\n' as u8,
        REDIS_TYPE_BULK_STRING,
        '5' as u8,
        '\r' as u8,
        '\n' as u8,
        'H' as u8,
        'e' as u8,
        'l' as u8,
        'l' as u8,
        'o' as u8,
        '\r' as u8,
        '\n' as u8,
        REDIS_TYPE_INTEGER,
        '1' as u8,
        '2' as u8,
        '\r' as u8,
        '\n' as u8,
    ]);
    let mut box_stream: Box<dyn RedisStream> = Box::new(stream);

    let result = RedisValue::Array(vec![
        RedisValue::BulkString(vec!['H' as u8, 'e' as u8, 'l' as u8, 'l' as u8, 'o' as u8]),
        RedisValue::Integer(12),
    ]);

    assert_eq!(read_array(&mut box_stream)?, result);

    Ok(())
}

#[test]
fn read_bulk_array_ko() {
    let stream = TestRedisStream::new(vec![]);
    let mut box_stream: Box<dyn RedisStream> = Box::new(stream);

    match read_array(&mut box_stream) {
        Ok(_) => panic!("Must be return error!"),
        Err(e) => {
            assert_eq!(e.kind(), ErrorKind::IoError);
            assert_eq!(e.io_error_kind().unwrap(), std::io::ErrorKind::BrokenPipe);
        }
    }
}

#[test]
fn read_array_type() {
    let stream = TestRedisStream::new(vec![
        REDIS_TYPE_INTEGER,
        '1' as u8,
        '2' as u8,
        '\r' as u8,
        '\n' as u8,
    ]);
    let mut box_stream: Box<dyn RedisStream> = Box::new(stream);

    match read_array(&mut box_stream) {
        Ok(_) => panic!("Must be return error!"),
        Err(e) => {
            assert_eq!(e.kind(), ErrorKind::OtherError);
            assert_eq!(e.message(), String::from("Not an array but a Integer"));
        }
    }
}

#[test]
fn read_array_type_unknow() {
    let stream = TestRedisStream::new(vec![
        '.' as u8, 'H' as u8, 'e' as u8, 'l' as u8, 'l' as u8, 'o' as u8, '\r' as u8, '\n' as u8,
    ]);
    let mut box_stream: Box<dyn RedisStream> = Box::new(stream);

    match read_array(&mut box_stream) {
        Ok(_) => panic!("Must be return error!"),
        Err(e) => {
            assert_eq!(e.kind(), ErrorKind::OtherError);
            assert_eq!(
                e.message(),
                String::from("Not an array but a Unknow '0x46'")
            );
        }
    }
}

#[test]
fn read_array_error() {
    let stream = TestRedisStream::new(vec![
        REDIS_TYPE_ERROR,
        'L' as u8,
        'O' as u8,
        'A' as u8,
        'D' as u8,
        'I' as u8,
        'N' as u8,
        'G' as u8,
        ' ' as u8,
        'H' as u8,
        'e' as u8,
        'l' as u8,
        'l' as u8,
        'o' as u8,
        '\r' as u8,
        '\n' as u8,
    ]);
    let mut box_stream: Box<dyn RedisStream> = Box::new(stream);

    match read_array(&mut box_stream) {
        Ok(_) => panic!("Must be return error!"),
        Err(e) => {
            assert_eq!(e.kind(), ErrorKind::BusyLoadingError);
            assert_eq!(e.message(), " Hello");
        }
    }
}

#[test]
fn read_array_error2() {
    let stream = TestRedisStream::new(vec![
        REDIS_TYPE_ERROR,
        'N' as u8,
        'O' as u8,
        'S' as u8,
        'C' as u8,
        'R' as u8,
        'I' as u8,
        'P' as u8,
        'T' as u8,
        ' ' as u8,
        'H' as u8,
        'e' as u8,
        'l' as u8,
        'l' as u8,
        'o' as u8,
        '\r' as u8,
        '\n' as u8,
    ]);
    let mut box_stream: Box<dyn RedisStream> = Box::new(stream);

    match read_array(&mut box_stream) {
        Ok(_) => panic!("Must be return error!"),
        Err(e) => {
            assert_eq!(e.kind(), ErrorKind::NoScriptError);
            assert_eq!(e.message(), " Hello");
        }
    }
}
