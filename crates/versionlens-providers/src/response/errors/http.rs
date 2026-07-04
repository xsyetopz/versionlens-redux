const HTTP_STATUS_MESSAGES: &[(u16, &str)] = &[
    (400, "400 bad request"),
    (401, "401 not authorized"),
    (403, "403 forbidden"),
    (404, "not found"),
    (500, "500 internal server error"),
];

pub fn http_status_message_from_code(status: u16) -> Option<&'static str> {
    HTTP_STATUS_MESSAGES
        .iter()
        .find_map(|(code, message)| (*code == status).then_some(*message))
}
