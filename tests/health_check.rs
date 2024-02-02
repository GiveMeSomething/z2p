use actix_web::{
    body::{BodySize, MessageBody},
    test,
};
use z2p::spawn_app;

#[actix_web::test]
async fn health_check_works() {
    let app = spawn_app().await;

    let req = test::TestRequest::get().uri("/health_check").to_request();
    let res = test::call_service(&app, req).await;

    assert!(res.status().is_success());
    assert_eq!(res.into_body().size(), BodySize::Sized(0));
}
