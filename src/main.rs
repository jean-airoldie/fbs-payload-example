use flatbuffers::{FlatBufferBuilder, WIPOffset};

/// A server that write flatbuffers messages to a `Sink`
/// (absent here) and reuses an internal `FlatBufferBuilder`.
struct Server<'bldr> {
    pub builder: FlatBufferBuilder<'bldr>,
}

impl<'bldr> Server<'bldr> {
    // This compiles but it requires to clone the content of the
    // builder which is less efficient.
    fn send_payload(&mut self, payload: &Payload) {
        let bytes_vec = payload.to_bytes(&mut self.builder);
        // Here we would write the bytes to a `Sink`.
    }

    // This would be the most efficient way to do it because we don't
    // need to clone any bytes. But this doesn't compile because it
    // requires the lifetime of the `&Payload` to be the same as the one
    // of the `FlatBufferBuilder`.
    fn send_payload_delegated(&mut self, payload: &Payload) {
        // This doesn't compile.
        let bytes_ref = payload.to_bytes_delegated(&mut self.builder);
        // Here we would write the bytes to a `Sink`.
        self.builder.reset();
    }

}

/// A struct that can be converted to a flatbuffers representation named
/// `fbs_payload::Payload`.
struct Payload {
    pub messages: Vec<Message>,
}

impl Payload {
    // This works because no references escape the anonymous lifetime of the function.
    // But this sucks since we have to clone the content of the `FlatBufferBuilder`
    // into a vector and it prevents code reuse.
    pub fn to_bytes<'a>(&self, builder: &'a mut FlatBufferBuilder) -> Vec<u8> {
        let mut vec = Vec::new();
        for message in &self.messages {
            // Here we can't call any functions that do the conversion for us
            // because it would let a `WIPOffset<_>` whose lifetime must be bound
            // to the builder escape.
            let mut message_builder = fbs_payload::MessageBuilder::new(builder);
            message_builder.add_value(message.value);
            vec.push(message_builder.finish());
        }
        let vec_offset = builder.create_vector(&vec);

        let mut payload_builder = fbs_payload::PayloadBuilder::new(builder);
        payload_builder.add_messages(vec_offset);
        let payload_offset = payload_builder.finish();

        builder.finish(payload_offset, None);
        let vec = builder.finished_data().to_vec();
        builder.reset();
        vec
    }

    // This is the most efficient way of doing because it doesnt to clone the content
    // of the `FlatBufferBuilder` into a temporary `Vec`. This also allows code reuse
    // since we can let `Message` write itself to the `FlatBufferBuilder`. But this
    // cannot compile because:
    // * The byte slice we return has the same lifetime as the `FlatBufferBuilder`,
    //   yet we will eventually call `FlatBufferBuilder::reset()` and it will expire.
    // * The `Message::to_fbs(...)` returns a `WIPOffset<_>` bound to the builder's
    //   lifetime.
    pub fn to_bytes_delegated<'a>(
        &'a self,
        builder: &'a mut FlatBufferBuilder<'a>,
    ) -> &'a [u8] {
        let mut vec = Vec::new();
        for message in &self.messages {
            // This compile because a `WIPOffset<_>` is copyable, yet
            // it will expire after being `FlatBufferBuilder::reset()`.
            vec.push(message.to_fbs(builder));
        }
        let vec_offset = builder.create_vector(&vec);

        let mut payload_builder = fbs_payload::PayloadBuilder::new(builder);
        payload_builder.add_messages(vec_offset);
        let payload_offset = payload_builder.finish();

        builder.finish(payload_offset, None);
        builder.finished_data()
    }
}

/// A struct that can be converted to a flatbuffers representation named
/// `fbs_payload::Message`.
struct Message {
    pub value: u32,
}

impl Message {
    pub fn to_fbs<'a>(
        &'a self,
        builder: &'a mut FlatBufferBuilder<'a>,
    ) -> WIPOffset<fbs_payload::Message> {
        let mut message_builder = fbs_payload::MessageBuilder::new(builder);
        message_builder.add_value(self.value);
        message_builder.finish()
    }
}

fn main() {
    let message = Message { value: 0 };
    let payload = Payload { messages: vec![message] };

    let mut server = Server { builder: FlatBufferBuilder::new() };
    // This works.
    server.send_payload(&payload);
    // This does not.
    server.send_payload_delegated(&payload);
}
