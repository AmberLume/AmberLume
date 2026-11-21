use std::fmt::Debug;
use std::fmt::Write;
use tracing::field::Field;
use tracing::field::Visit;

pub struct TracingMessage {
    message: String,
    fields: String,
}

impl TracingMessage {
    pub fn new() -> Self {
        Self {
            message: String::new(),
            fields: String::new(),
        }
    }

    pub fn into_text(mut self) -> String {
        self.message.push_str(&self.fields);

        self.message
    }
}

impl Visit for TracingMessage {
    fn record_debug(&mut self, field: &Field, value: &dyn Debug) {
        if field.name() == "message" {
            write!(self.message, "{value:?}").expect("writing to a String cannot fail");
        } else {
            write!(self.fields, " {}={value:?}", field.name()).expect("writing to a String cannot fail");
        }
    }
}
