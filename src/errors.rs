
error_chain! {
    types {
        Error, ErrorKind, ResultExt, Result;
    }

    links {
    }

    foreign_links {
        Io(::std::io::Error);
        Crypto(::ring::error::Unspecified);
        Bincode(::bincode::Error);
    }

    errors {
        General(s: String) {
            description("General Error"),
            display("General Error: '{}'", s),
        }
        InvalidPacket {
            description("Invalid Packet"),
        }
        NotSynchronized {
            description("Not Synchronized"),
        }
        PartialSend {
            description("Packet not fully sent"),
        }
        SendingFailed {
            description("Packet sending failed"),
        }
        Shutdown {
            description("Shutdown"),
        }
        UpgradeRequired(version: u32) {
            description("Upgrade required"),
            display("Upgrade to version {} required", version),
        }
        RemoteFailedChallenge {
            description("Remote failed challenge"),
        }
    }
}
