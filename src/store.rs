use std::error::Error;
use std::io::Read;
use redis::{Commands, RedisError, RedisResult};
use sha2::{Sha256, Digest};
use sha2::digest::consts::U32;
use sha2::digest::generic_array::GenericArray;
use teloxide::prelude::*;

pub fn get_star_count(
    user_id: UserId,
    group_id: ChatId,
    client: &redis::Client
) -> RedisResult<usize>  {
    let mut conn = client.get_connection()?;
    conn.scard(&format!("{}:{}:stars", group_id, user_id.0))
}

pub fn give_star(
    giver: UserId,
    receiver: UserId,
    salt: &[u8],
    redis_key: &str,
    redis_client: &redis::Client,
) -> Result<(), RedisError> {
    let mut conn = redis_client.get_connection()?;
    conn.sadd(redis_key, &hash(giver, receiver, salt)[..])?;
    Ok(())
}

fn hash(
    giver: UserId,
    receiver: UserId,
    salt: &[u8],
) -> GenericArray<u8, U32> {
    Sha256::new()
        .chain_update(giver.0.to_be_bytes())
        .chain_update(receiver.0.to_be_bytes())
        .chain_update(salt)
        .finalize()
}

#[cfg(test)]
mod tests {
    use teloxide::prelude::*;
    use crate::store::hash;

    #[test]
    fn hash_is_same() {
        let hash1 = hash(
            UserId(279838373),
            UserId(195125422),
            b"makaroshki"
        );
        let hash2 = hash(
            UserId(279838373),
            UserId(195125422),
            b"makaroshki"
        );
        println!("{:?}", hash1);
        assert_eq!(hash1, hash2)
    }
}