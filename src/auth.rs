use base64::Engine;
use scrypt::{
  password_hash::{
    rand_core::{OsRng, RngCore},
    PasswordHash, PasswordHasher, PasswordVerifier, Salt, SaltString,
  },
  Scrypt,
};
use tonic::Status;

use crate::db::get_hashed_password;

pub fn hash_password(password: &str) -> Result<String, scrypt::password_hash::Error> {
  let salt = SaltString::generate(&mut OsRng);
  let params = scrypt::Params::new(15, 2, 2, scrypt::Params::RECOMMENDED_LEN).unwrap();
  let hash = Scrypt
    .hash_password_customized(
      password.as_bytes(),
      None,
      None,
      params,
      Salt::try_from(salt.as_ref())?,
    )?
    .to_string();
  Ok(hash)
}

fn verify_password_with_hash(
  password: &str,
  hash: &str,
) -> Result<(), scrypt::password_hash::Error> {
  let hash = PasswordHash::new(&hash)?;
  Scrypt.verify_password(password.as_bytes(), &hash)
}

/// Returns the user id if the username and password are correct.
pub async fn verify_password(username: &str, password: &str) -> Result<i32, Status> {
  let (user_id, hash) = match get_hashed_password(username).await? {
    Some((user_id, hash)) => (user_id, hash),
    None => {
      // Perform a dummy hash to make the function take a consistent amount of time to prevent user
      // enumeration attacks.
      let _ = hash_password(password);

      return Err(Status::unauthenticated("Invalid username or password"));
    },
  };

  match verify_password_with_hash(password, &hash) {
    Ok(_) => Ok(user_id),
    Err(err) => {
      error!("Error verifying password for user {username}: {err}");
      Err(Status::unauthenticated("Invalid username or password"))
    },
  }
}

pub fn generate_session_token() -> String {
  let mut rng = OsRng;
  let mut bytes = [0u8; 64];
  rng.fill_bytes(&mut bytes);
  base64::engine::general_purpose::STANDARD.encode(&bytes)
}

#[test]
fn test_hash_password() {
  let password = "password";
  let hash = hash_password(password).unwrap();
  assert!(verify_password_with_hash(password, &hash).is_ok());
}
