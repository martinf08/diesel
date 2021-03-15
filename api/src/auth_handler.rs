use diesel::prelude::*;
use actix_identity;
use actix_web::{web, HttpResponse};
use db::models::{User, SlimUser};
use diesel::{QueryDsl, ExpressionMethods};
use db::DbPool;
use futures::{Future, FutureExt};
use crate::errors::ServiceError;
use actix_identity::Identity;
use actix_web::error::BlockingError;

pub fn login(
    auth_data: web::Json<User>,
    id: Identity,
    pool: web::Data<DbPool>,
) -> impl Future<Item = HttpResponse, Error = ServiceError> {
    web::block(move || query_user(auth_data.into_inner(), pool)).then(
        move |res: Result<(User, User), BlockingError<ServiceError>>| match res {
            Ok(users) => {
                let (user_data, user_found) = users;
                if user_data.password == user_found.password {
                    let user = SlimUser {
                        name: user_data.name
                    };
                    id.remember(serde_json::to_string(&user).unwrap());
                    Ok(HttpResponse::Ok().finish());
                };
                Err(ServiceError::InternalServerError)
            },
            Err(_) => Err(ServiceError::InternalServerError)
        }
    )
}

fn query_user(auth_data: User, pool: web::Data<DbPool>) -> Result<(User, User), ServiceError> {
    use db::schema::users::dsl::*;
    let conn = &pool.get().unwrap();
    let mut users_found = users.
        filter(name.eq(&auth_data.name))
        .load::<User>(conn)?;

    Ok((auth_data, users_found.pop().unwrap()))
}