@0xaa760cf01a714145;

using Java = import "/capnp/java.capnp";
$Java.package("org.kylekewley.picore");
$Java.outerClassname("Error");

struct Error {
    code @0 :UInt32 = 0;
    message @1 :Text = "";
    blameId @2 :UInt64; # The ID of the message responsible for the error
}
