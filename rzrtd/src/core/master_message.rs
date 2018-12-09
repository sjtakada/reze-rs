//
// ReZe.Rs - Router Daemon
//   Copyright (C) 2018 Toshiaki Takada
//
// Master Message
// - Master to Protocol
//   - Timer Expiration
//   - Config Command (async)
//   - Show Command (sync)
// - Protocol to Master
//   - Timer Registration
//   - Show Command output
//   - Protocol Termination
//

pub enum MessageToMaster {
    TimerRegistration((i32, i32)),
    ProtoTermination(i32)
}

pub enum MessageToProto {
    TimerExpiration(i32),
    PostConfig((String, Vec<String>))
}


