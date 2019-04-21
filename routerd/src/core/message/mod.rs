//
// ReZe.Rs - Router Daemon
//   Copyright (C) 2018,2019 Toshiaki Takada
//
// Message
//
//   Nexus Message Channels
//
//               +-------+ 
//               |       | 
//               | Nexus | 
//               |       | receiver_p2n
//               +---m---+ sender_n2p
//                   |
//     +-------------+--------------+
//     |             |              |
// +---s---+     +---s---+      +---s---+  sender_p2n
// |       |     |       |      |       |  receiver_n2p
// | Zebra |     | OSPF  |      |  BGP  |
// |       |     |       |      |       |
// +-------+     +-------+      +-------+
//
//
//   Zebra Message Channels
//
//               +-------+ 
//               |       | 
//               | Zebra | 
//               |       | receiver_p2z
//               +---m---+ sender_z2p
//                   |
//     +-------------+--------------+
//     |                            |
// +---s---+                    +---s---+  sender_p2z
// |       |                    |       |  receiver_z2p
// | OSPF  |                    |  BGP  |
// |       |                    |       |
// +-------+                    +-------+
//
pub mod nexus;
pub mod zebra;
