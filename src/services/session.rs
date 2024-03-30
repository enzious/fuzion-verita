use actix_web_thiserror::ResponseError;
use fuzion_commons::db::{PgClient, PgClientError};
use thiserror::Error;

use crate::dao::realm::RealmId;
use crate::repos::user::UserRepo;
use crate::repos::RepoError;

use super::user::{UserService, UserServiceError};

pub struct SessionService;

impl SessionService {
  pub async fn login(
    db_client: &PgClient<'_>,
    realm_id: RealmId,
    user: &str,
    password: &str,
  ) -> Result<(), SessionServiceError> {
    let user_id = UserRepo::get_user_by_username_email(&db_client, realm_id, &user)
      .await?
      .and_then(|user| user.id)
      .ok_or(SessionServiceError::InvalidCredentialsUser)?;

    let _user_credential = UserService::verify_credential(db_client, user_id, password)
      .await
      .map_err(|err| match err {
        UserServiceError::CredentialServiceError(_) => SessionServiceError::InvalidCredentials(err),
        _ => err.into(),
      })?;

    Ok(())
  }
}

#[derive(Debug, Error, ResponseError)]
pub enum SessionServiceError {
  #[error("internal_error")]
  InternalError,
  #[response(status = 401, reason = "invalid_credentials")]
  #[error("invalid_user")]
  InvalidCredentialsUser,
  #[response(status = 401, reason = "invalid_credentials")]
  #[error(transparent)]
  InvalidCredentials(UserServiceError),
  #[error(transparent)]
  PostgresError(#[from] PgClientError),
  #[error(transparent)]
  RepoError(#[from] RepoError),
  #[error(transparent)]
  UserServiceError(#[from] UserServiceError),
}
