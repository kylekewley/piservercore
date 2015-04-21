@0x8737d1e5f070a0af;

using Error = import "Error.capnp".Error;

struct Ack {
    messageId @0 :UInt64;
    status @1 :Status;
    error @2 :Error;

    enum Status {
        success @0;
        resend @1;
        failure @2;
    }
}
