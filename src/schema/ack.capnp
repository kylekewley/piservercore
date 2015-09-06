@0x8737d1e5f070a0af;


using Java = import "/capnp/java.capnp";
$Java.package("org.kylekewley.picore");
$Java.outerClassname("Ack");

using Error = import "error.capnp".Error;

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
