use actix_web::HttpResponse;
use actix_web_flash_messages::FlashMessage;

use crate::{
    routes::helpers::{e500, see_other},
    session_state::TypedSession,
};

pub async fn logout(session: TypedSession) -> Result<HttpResponse, actix_web::error::Error> {
    if session.get_user_id().map_err(e500)?.is_none() {
        Ok(see_other("/login"))
    } else {
        session.logout();
        FlashMessage::info("You have successfully logged out.").send();
        Ok(see_other("/login"))
    }
}
