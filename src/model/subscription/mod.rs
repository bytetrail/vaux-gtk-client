use glib::Object;

mod objimpl;

glib::wrapper! {
    pub struct Subscription(ObjectSubclass<objimpl::Subscription>);
}

impl Subscription {
    pub fn new() -> Self {
        let obj = Object::builder()
            .property("topic", "".to_string())
            .property("id", 0)
            .property("active", false)
            .build();
        obj
    }
}
