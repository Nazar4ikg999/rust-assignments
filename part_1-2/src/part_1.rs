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

    
    pub fn submit_for_moderation(self) -> Post<Unmoderated> {
        Post {
            title: self.title,
            body: self.body,
            _state: PhantomData,
        }
    }
}

impl Post<Unmoderated> {
    
    pub fn allow(self) -> Post<Published> {
        Post {
            title: self.title,
            body: self.body,
            _state: PhantomData,
        }
    }

    
    pub fn deny(self) -> Post<Deleted> {
        Post {
            title: self.title,
            body: self.body,
            _state: PhantomData,
        }
    }
}

impl Post<Published> {
    pub fn delete(self) -> Post<Deleted> {
        Post {
            title: self.title,
            body: self.body,
            _state: PhantomData,
        }
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn body(&self) -> &str {
        &self.body
    }
}

impl Post<Deleted> {
    pub fn title(&self) -> &str {
        &self.title
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn happy_flow_from_new_to_published() {
        let post = Post::new("Hello", "Rust");
        let post = post.submit_for_moderation();
        let post = post.allow();
        assert_eq!(post.title(), "Hello");
    }

    #[test]
    fn delete_published_post() {
        let post = Post::new("Title", "Body")
            .submit_for_moderation()
            .allow()
            .delete();
        assert_eq!(post.title(), "Title");
    }
    
}