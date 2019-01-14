use flatbuffers::{FlatBufferBuilder, WIPOffset};
use fbs_payload_example::{Message, MessageBuilder, PayloadBuilder};

/// A server that write flatbuffers messages to a `Sink`
/// (absent here) and reuses an internal `FlatBufferBuilder`.
struct Server<'bldr> {
    pub builder: FlatBufferBuilder<'bldr>,
}

impl<'bldr> Server<'bldr> {
    fn send_payload(&mut self, payload_struct: &PayloadStruct) {
        let bytes = payload_struct.to_bytes(&mut self.builder);
        // Here we would write the bytes to a `Sink`.
        self.builder.reset();
    }
}

/// A struct that can be converted to a flatbuffer `Payload`.
struct PayloadStruct {
    message_structs: Vec<MessageStruct>,
}

impl PayloadStruct {
    /// This compiles but I have to do the MessageStruct to Message conversion
    /// myself.
    pub fn to_bytes<'bldr>(
        &self,
        builder: &'bldr mut FlatBufferBuilder,
    ) -> &'bldr [u8] {
        let mut vec = Vec::new();
        for message in &self.message_structs {
            // Here I have to know how to convert MessageStruct to Message.
            let mut message_builder = MessageBuilder::new(builder);
            message_builder.add_value(message.value);
            vec.push(message_builder.finish());
        }
        let vec_offset = builder.create_vector(&vec);

        let mut payload_builder = PayloadBuilder::new(builder);
        payload_builder.add_messages(vec_offset);
        let payload_offset = payload_builder.finish();

        builder.finish(payload_offset, None);
        builder.finished_data()
    }

    /// I can't get this to compile.
    pub fn to_bytes_delegated<'bldr>(
        &self,
        builder: &'bldr mut FlatBufferBuilder,
    ) -> &'bldr [u8] {
        let mut vec = Vec::new();
        for message in &self.message_structs {
            // Here I can let MessageStruct do the conversion.
            vec.push(message.to_fbs(builder));
        }
        let vec_offset = builder.create_vector(&vec);

        let mut payload_builder = PayloadBuilder::new(builder);
        payload_builder.add_messages(vec_offset);
        let payload_offset = payload_builder.finish();

        builder.finish(payload_offset, None);
        builder.finished_data()
    }
}

/// A struct that can be converted to a flatbuffer `Message`.
struct MessageStruct {
    value: u32,
}

impl MessageStruct {
    pub fn to_fbs<'bldr>(
        &self,
        builder: &'bldr mut FlatBufferBuilder,
    ) -> WIPOffset<Message> {
        let mut message_builder = MessageBuilder::new(builder);
        message_builder.add_value(self.value);
        message_builder.finish()
    }
}

fn main() {
}
