//
// ReZe.Rs - Router Daemon
//   Copyright (C) 2018,2019 Toshiaki Takada
//
// Event Handler
//

pub enum EventType {
    SimpleEvent,
    ReadEvent,
    WriteEvent,
    TimerEvent,
}

//pub enum EventParam {
//    Param(String)
//}

pub trait EventHandler {
    fn handle(&self, event_type: EventType/*, _event_param: i32*/);
}

