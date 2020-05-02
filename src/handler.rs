use actix_redis::RedisSession;
use actix_files as fs;
use actix_session::Session;
use actix_web::{web, HttpResponse, Result};
use time;
use serde::{Deserialize, Serialize};

pub fn redis_session(private_key: &[u8], session_key: &'static str, cookie_name: &'static str)-> Result<RedisSession> {
    // Generate a random 32 byte key. Note that it is important to use a unique
    // private key for every project. Anyone with access to the key can generate
    // authentication cookies for any user!
    let session = RedisSession::new("127.0.0.1:6379", private_key)
        .cookie_name(cookie_name)
        .cookie_max_age(time::Duration::minutes(20))
        .cache_keygen(Box::new(move |key: &str| format!("{}:{}", session_key, &key)));

    Ok(session)
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct IndexResponse {
    user_id: Option<String>,
    counter: i32,
}

pub async fn index(session: Session) -> Result<HttpResponse> {
    let user_id: Option<String> = session.get::<String>("user_id").unwrap();
    let counter: i32 = session
        .get::<i32>("counter")
        .unwrap_or(Some(0))
        .unwrap_or(0);

    Ok(HttpResponse::Ok().json(IndexResponse { user_id, counter }))
}

pub async fn count_up(session: Session) -> Result<HttpResponse> {
    if let Some(user_id) = session.get::<String>("user_id").unwrap() {
        let counter: i32 = session
            .get::<i32>("counter")
            .unwrap_or(Some(0))
            .map_or(1, |inner| inner + 1);
        session.set("counter", counter)?;

        Ok(HttpResponse::Ok().json(IndexResponse { user_id: Some(user_id), counter }))
    } else {
        Ok(HttpResponse::Ok().json(IndexResponse { user_id: None, counter: 0 }))
    }
}

#[derive(Deserialize)]
pub struct Identity {
    user_id: String,
    password: String,
}

pub async fn login(web_json: web::Json<Identity>, session: Session) -> Result<HttpResponse> {
    let json = web_json.into_inner();
    let id = json.user_id;
    let pass = json.password;
    if id == "user".to_string() && pass == "password".to_string() {
        session.set("user_id", &id)?;
        session.renew();

        let counter: i32 = session
            .get::<i32>("counter")
            .unwrap_or(Some(0))
            .unwrap_or(0);

        Ok(HttpResponse::Ok().json(IndexResponse {
            user_id: Some(id),
            counter,
        }))
    } else {
        Ok(HttpResponse::Ok().json(IndexResponse {
            user_id: None,
            counter: 0,
        }))
    }
}

pub async fn logout(session: Session) -> Result<HttpResponse> {
    let id: Option<String> = session.get("user_id")?;
    if let Some(x) = id {
        session.purge();
        Ok(format!("Logged out: {}", x).into())
    } else {
        Ok("Could not log out anonymous user".into())
    }
}

pub async fn favicon() -> Result<fs::NamedFile>{
    Ok(fs::NamedFile::open("./dist/rust-rust.png")?)
}

pub fn asset() -> Result<fs::Files> {
    Ok(fs::Files::new("/", "./dist/").index_file("index.html"))
}


#[cfg(test)]
mod test {
    use super::*;
    use actix_http::httpmessage::HttpMessage;
    use actix_web::{
        middleware, test, App,
        web::{get, post, resource},
    };
    use serde_json::json;
    use time;
    use rand::Rng;

    #[actix_rt::test]
    async fn test_workflow() {
        // Step 1:  GET index
        //   - set-cookie actix-session will be in response (session cookie #1)
        //   - response should be: {"counter": 0, "user_id": None}
        // Step 2:  GET index, including session cookie #1 in request
        //   - set-cookie will *not* be in response
        //   - response should be: {"counter": 0, "user_id": None}
        // Step 3: POST to count_up, including session cookie #1 in request
        //   - adds new session state in redis:  {"counter": 1}
        //   - response should be: {"counter": 1, "user_id": None}
        // Step 4: POST again to count_up, including session cookie #1 in request
        //   - updates session state in redis:  {"counter": 2}
        //   - response should be: {"counter": 2, "user_id": None}
        // Step 5: POST to login, including session cookie #1 in request
        //   - set-cookie actix-session will be in response  (session cookie #2)
        //   - updates session state in redis: {"counter": 2, "user_id": "ferris"}
        // Step 6: GET index, including session cookie #2 in request
        //   - response should be: {"counter": 2, "user_id": "ferris"}
        // Step 7: POST again to count_up, including session cookie #2 in request
        //   - updates session state in redis: {"counter": 3, "user_id": "ferris"}
        //   - response should be: {"counter": 2, "user_id": None}
        // Step 8: GET index, including session cookie #1 in request
        //   - set-cookie actix-session will be in response (session cookie #3)
        //   - response should be: {"counter": 0, "user_id": None}
        // Step 9: POST to logout, including session cookie #2
        //   - set-cookie actix-session will be in response with session cookie #2
        //     invalidation logic
        // Step 10: GET index, including session cookie #2 in request
        //   - set-cookie actix-session will be in response (session cookie #3)
        //   - response should be: {"counter": 0, "user_id": None}

        let private_key = rand::thread_rng().gen::<[u8; 32]>();
        let srv = test::start(move || {
            App::new()
                .wrap(redis_session(&private_key, "test-session", "test-session").unwrap())
                .wrap(middleware::Logger::default())
                .service(resource("/user").route(get().to(index)))
                .service(resource("/count_up").route(post().to(count_up)))
                .service(resource("/login").route(post().to(login)))
                .service(resource("/logout").route(post().to(logout)))
        });

        // Step 1:  GET index
        //   - set-cookie actix-session will be in response (session cookie #1)
        //   - response should be: {"counter": 0, "user_id": None}
        let req_1a = srv.get("/user").send();
        let mut resp_1 = req_1a.await.unwrap();
        let cookie_1 = resp_1
            .cookies()
            .unwrap()
            .clone()
            .into_iter()
            .find(|c| c.name() == "test-session")
            .unwrap();
        let result_1 = resp_1.json::<IndexResponse>().await.unwrap();
        assert_eq!(
            result_1,
            IndexResponse {
                user_id: None,
                counter: 0
            }
        );

        // Step 2:  GET index, including session cookie #1 in request
        //   - set-cookie will *not* be in response
        //   - response should be: {"counter": 0, "user_id": None}
        let req_2 = srv.get("/user").cookie(cookie_1.clone()).send();
        let resp_2 = req_2.await.unwrap();
        let cookie_2 = resp_2
            .cookies()
            .unwrap()
            .clone()
            .into_iter()
            .find(|c| c.name() == "test-session");
        assert_eq!(cookie_2, None);

        // Step 3: POST to count_up, including session cookie #1 in request
        //   - adds new session state in redis:  {"counter": 1}
        //   - response should be: {"counter": 1, "user_id": None}
        let req_3 = srv.post("/count_up").cookie(cookie_1.clone()).send();
        let mut resp_3 = req_3.await.unwrap();
        let result_3 = resp_3.json::<IndexResponse>().await.unwrap();
        assert_eq!(
            result_3,
            IndexResponse {
                user_id: None,
                counter: 1
            }
        );

        // Step 4: POST again to count_up, including session cookie #1 in request
        //   - updates session state in redis:  {"counter": 2}
        //   - response should be: {"counter": 2, "user_id": None}
        let req_4 = srv.post("/count_up").cookie(cookie_1.clone()).send();
        let mut resp_4 = req_4.await.unwrap();
        let result_4 = resp_4.json::<IndexResponse>().await.unwrap();
        assert_eq!(
            result_4,
            IndexResponse {
                user_id: None,
                counter: 2
            }
        );

        // Step 5: POST to login, including session cookie #1 in request
        //   - set-cookie actix-session will be in response  (session cookie #2)
        //   - updates session state in redis: {"counter": 2, "user_id": "ferris"}
        let req_5 = srv
            .post("/login")
            .cookie(cookie_1.clone())
            .send_json(&json!({"user_id": "ferris"}));
        let mut resp_5 = req_5.await.unwrap();
        let cookie_2 = resp_5
            .cookies()
            .unwrap()
            .clone()
            .into_iter()
            .find(|c| c.name() == "test-session")
            .unwrap();
        assert_eq!(
            true,
            cookie_1.value().to_string() != cookie_2.value().to_string()
        );

        let result_5 = resp_5.json::<IndexResponse>().await.unwrap();
        assert_eq!(
            result_5,
            IndexResponse {
                user_id: Some("ferris".into()),
                counter: 2
            }
        );

        // Step 6: GET index, including session cookie #2 in request
        //   - response should be: {"counter": 2, "user_id": "ferris"}
        let req_6 = srv.get("/user").cookie(cookie_2.clone()).send();
        let mut resp_6 = req_6.await.unwrap();
        let result_6 = resp_6.json::<IndexResponse>().await.unwrap();
        assert_eq!(
            result_6,
            IndexResponse {
                user_id: Some("ferris".into()),
                counter: 2
            }
        );

        // Step 7: POST again to count_up, including session cookie #2 in request
        //   - updates session state in redis: {"counter": 3, "user_id": "ferris"}
        //   - response should be: {"counter": 2, "user_id": None}
        let req_7 = srv.post("/count_up").cookie(cookie_2.clone()).send();
        let mut resp_7 = req_7.await.unwrap();
        let result_7 = resp_7.json::<IndexResponse>().await.unwrap();
        assert_eq!(
            result_7,
            IndexResponse {
                user_id: Some("ferris".into()),
                counter: 3
            }
        );

        // Step 8: GET index, including session cookie #1 in request
        //   - set-cookie actix-session will be in response (session cookie #3)
        //   - response should be: {"counter": 0, "user_id": None}
        let req_8 = srv.get("/user").cookie(cookie_1.clone()).send();
        let mut resp_8 = req_8.await.unwrap();
        let cookie_3 = resp_8
            .cookies()
            .unwrap()
            .clone()
            .into_iter()
            .find(|c| c.name() == "test-session")
            .unwrap();
        let result_8 = resp_8.json::<IndexResponse>().await.unwrap();
        assert_eq!(
            result_8,
            IndexResponse {
                user_id: None,
                counter: 0
            }
        );
        assert!(cookie_3.value().to_string() != cookie_2.value().to_string());

        // Step 9: POST to logout, including session cookie #2
        //   - set-cookie actix-session will be in response with session cookie #2
        //     invalidation logic
        let req_9 = srv.post("/logout").cookie(cookie_2.clone()).send();
        let resp_9 = req_9.await.unwrap();
        let cookie_4 = resp_9
            .cookies()
            .unwrap()
            .clone()
            .into_iter()
            .find(|c| c.name() == "test-session")
            .unwrap();
        assert!(&time::now().tm_year != &cookie_4.expires().map(|t| t.tm_year).unwrap());

        // Step 10: GET index, including session cookie #2 in request
        //   - set-cookie actix-session will be in response (session cookie #3)
        //   - response should be: {"counter": 0, "user_id": None}
        let req_10 = srv.get("/user").cookie(cookie_2.clone()).send();
        let mut resp_10 = req_10.await.unwrap();
        let result_10 = resp_10.json::<IndexResponse>().await.unwrap();
        assert_eq!(
            result_10,
            IndexResponse {
                user_id: None,
                counter: 0
            }
        );

        let cookie_5 = resp_10
            .cookies()
            .unwrap()
            .clone()
            .into_iter()
            .find(|c| c.name() == "test-session")
            .unwrap();
        assert!(cookie_5.value().to_string() != cookie_2.value().to_string());
    }
}
