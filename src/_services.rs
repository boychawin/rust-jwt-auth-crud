// use crate::{AppState, TokenClaims};
// use actix_web::{
//     get, post,
//     web::{Data, Json, ReqData},
//     HttpResponse, Responder,
// };
// use actix_web_httpauth::extractors::basic::BasicAuth;
// use argonautica::{Hasher, Verifier};
// use chrono::NaiveDateTime;
// use hmac::{Hmac, Mac};
// use jwt::SignWithKey;
// use serde::{Deserialize, Serialize};
// use sha2::Sha256;
// use sqlx::{self, query, FromRow, Row};

// #[derive(Deserialize)]
// pub struct CreateUserBody {
//     username: String,
//     password: String,
// }

// #[derive(Serialize, FromRow)]
// pub struct UserNoPassword {
//     id: u64,
//     username: String,
// }

// #[derive(Serialize, FromRow)]
// struct AuthUser {
//     id: i32,
//     username: String,
//     password: String,
// }

// #[derive(Deserialize)]
// pub struct CreateArticleBody {
//     title: String,
//     content: String,
// }

// #[derive(Serialize, FromRow)]
// pub struct Article {
//     id: i32,
//     title: String,
//     content: String,
//     published_by: i32,
//     published_on: Option<NaiveDateTime>,
// }

// #[post("/user")]
// async fn create_user(state: Data<AppState>, body: Json<CreateUserBody>) -> impl Responder {
//     let user: CreateUserBody = body.into_inner();

//     // Retrieve the HASH_SECRET environment variable
//     let hash_secret = match std::env::var("HASH_SECRET") {
//         Ok(secret) => secret,
//         Err(_) => {
//             return HttpResponse::InternalServerError().json("HASH_SECRET must be set!");
//         }
//     };

//     // Check if the username already exists
//     let username_check: Result<sqlx::mysql::MySqlRow, sqlx::Error> = query("SELECT COUNT(*) as count FROM users WHERE username = ?")
//         .bind(&user.username)
//         .fetch_one(&state.db)
//         .await;

//     match username_check {
//         Ok(row) => {
//             let count: i64 = row.get("count");
//             if count > 0 {
//                 return HttpResponse::BadRequest().json("Username already exists");
//             }
//         }
//         Err(_) => {
//             return HttpResponse::InternalServerError().json("Database error checking username");
//         }
//     }

//     // Hash the password
//     let mut hasher = Hasher::default();
//     let hash = match hasher
//         .with_password(&user.password)
//         .with_secret_key(&hash_secret)
//         .hash()
//     {
//         Ok(hash) => hash,
//         Err(_) => {
//             return HttpResponse::InternalServerError().json("Hashing error");
//         }
//     };

//     // Insert the user into the database
//     let insert_result = sqlx::query("INSERT INTO users (username, password) VALUES (?, ?)")
//         .bind(&user.username)
//         .bind(&hash)
//         .execute(&state.db)
//         .await;

//     if let Err(_) = insert_result {
//         return HttpResponse::InternalServerError().json("Database error");
//     }

//     // Fetch the inserted user
//     let fetch_result =
//         sqlx::query_as::<_, UserNoPassword>("SELECT id, username FROM users WHERE username = ?")
//             .bind(&user.username)
//             .fetch_one(&state.db)
//             .await;

//     match fetch_result {
//         Ok(user) => HttpResponse::Ok().json(user),
//         Err(_) => HttpResponse::InternalServerError().json("Error fetching user"),
//     }
// }

// #[get("/auth")]
// async fn basic_auth(state: Data<AppState>, credentials: BasicAuth) -> impl Responder {

//     println!("{:?}",credentials);
//     let jwt_secret: Hmac<Sha256> = Hmac::new_from_slice(
//         std::env::var("JWT_SECRET")
//             .expect("JWT_SECRET must be set!")
//             .as_bytes(),
//     )
//     .unwrap();
//     let username = credentials.user_id();
//     let password = credentials.password();

//     match password {
//         None => HttpResponse::Unauthorized().json("Must provide username and password"),
//         Some(pass) => {
//             match sqlx::query_as::<_, AuthUser>(
//                 "SELECT id, username, password FROM users WHERE username = $1",
//             )
//             .bind(username.to_string())
//             .fetch_one(&state.db)
//             .await
//             {
//                 Ok(user) => {
//                     let hash_secret =
//                         std::env::var("HASH_SECRET").expect("HASH_SECRET must be set!");
//                     let mut verifier = Verifier::default();
//                     let is_valid = verifier
//                         .with_hash(user.password)
//                         .with_password(pass)
//                         .with_secret_key(hash_secret)
//                         .verify()
//                         .unwrap();

//                     if is_valid {
//                         let claims = TokenClaims { id: user.id };
//                         let token_str = claims.sign_with_key(&jwt_secret).unwrap();
//                         HttpResponse::Ok().json(token_str)
//                     } else {
//                         HttpResponse::Unauthorized().json("Incorrect username or password")
//                     }
//                 }
//                 Err(error) => HttpResponse::InternalServerError().json(format!("{:?}", error)),
//             }
//         }
//     }
// }

// #[post("/article")]
// async fn create_article(
//     state: Data<AppState>,
//     req_user: Option<ReqData<TokenClaims>>,
//     body: Json<CreateArticleBody>,
// ) -> impl Responder {
//     match req_user {
//         Some(user) => {
//             let article: CreateArticleBody = body.into_inner();

//             match sqlx::query_as::<_, Article>(
//                 "INSERT INTO articles (title, content, published_by)
//                 VALUES ($1, $2, $3)
//                 RETURNING id, title, content, published_by, published_on",
//             )
//             .bind(article.title)
//             .bind(article.content)
//             .bind(user.id)
//             .fetch_one(&state.db)
//             .await
//             {
//                 Ok(articles) => HttpResponse::Ok().json(articles),
//                 Err(error) => HttpResponse::InternalServerError().json(format!("{:?}", error)),
//             }
//         }
//         _ => HttpResponse::Unauthorized().json("Unable to verify identity"),
//     }
// }
