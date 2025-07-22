use glib::Object;

mod objimpl;

glib::wrapper! {
    pub struct Subscription(ObjectSubclass<objimpl::Subscription>);
}

impl Default for Subscription {
    fn default() -> Self {
        Self::new()
    }
}

impl Subscription {
    pub fn new() -> Self {
        Object::builder()
            .property("topic", "".to_string())
            .property("id", 0)
            .property("active", false)
            .build()
    }
}
