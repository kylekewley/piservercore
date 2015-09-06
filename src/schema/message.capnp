@0xf8b166d9fe74285a;

using Java = import "/capnp/java.capnp";
$Java.package("org.kylekewley.picore");
$Java.outerClassname("Message");

struct Message {
    messageId @0 :UInt64;
    parserId @1 :UInt32;
    recvAck @2 :Bool = true;
    message @3 :Data;
}
