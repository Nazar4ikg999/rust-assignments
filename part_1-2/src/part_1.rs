use std::marker::PhantomData;

pub struct Post<State> {
    title: String,
    body: String,
    _state: PhantomData<State>,
}

pub struct New;
pub struct Unmoderated;
pub struct Published;
pub struct Deleted;

impl Post<New> {
    pub fn new(title: impl Into<String>, body: impl Into<String>) -> Self {
        Post {
            title: title.into(),
            body: body.into(),
            _state: PhantomData,
}
    }
}
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Headers {
    pub accept: String,
    pub authorization: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Payload {
    pub active: bool,
    pub role: String,
    pub limit: u32,
}
