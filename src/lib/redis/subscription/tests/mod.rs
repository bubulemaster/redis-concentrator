use crate::lib::redis::stream::tests::TestRedisStream;
use crate::lib::redis::stream::RedisStream;
use crate::lib::redis::subscription::RedisSubscription;
use crate::lib::redis::types::{
    RedisError, RedisValue, REDIS_TYPE_ARRAY, REDIS_TYPE_BULK_STRING, REDIS_TYPE_INTEGER,
};

#[test]
fn test_subscription() -> Result<(), RedisError> {
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

    let box_stream: Box<dyn RedisStream> = Box::new(stream);

    let mut sub = RedisSubscription::new(box_stream, String::from("truc"));

    sub.subscribe()?;

    let result = RedisValue::Array(vec![
        RedisValue::BulkString(vec!['H' as u8, 'e' as u8, 'l' as u8, 'l' as u8, 'o' as u8]),
        RedisValue::Integer(12),
    ]);

    assert_eq!(sub.pool()?, result);

    /*
    TODO How test this ?
    let mut write_data = Vec::new();

    write_data.extend_from_slice(box_stream.write_buf.as_ref());

    assert_eq!(
        write_data,
        vec![
            'S' as u8, 'U' as u8, 'B' as u8, 'S' as u8, 'C' as u8, 'R' as u8, 'I' as u8, 'B' as u8,
            'E' as u8, ' ' as u8, 't' as u8, 'r' as u8, 'u' as u8, 'c' as u8, '\r' as u8,
            '\n' as u8
        ]
    );
    */
    Ok(())
}
