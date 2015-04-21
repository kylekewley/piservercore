@0xf8b166d9fe74285a;

struct Message(Type) {
    messageId @0 :UInt64;
    parserId @1 :UInt32;
    recvAck @2 :Bool = true;
    message @3 :Type;
}
