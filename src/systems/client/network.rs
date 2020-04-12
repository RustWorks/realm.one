use amethyst::{
    core::{bundle::SystemBundle},
    core::{SystemDesc},
    ecs::{Read, System, SystemData, World, Write, DispatcherBuilder},
    shrev::{EventChannel, ReaderId}, 
    network::simulation::{NetworkSimulationEvent, NetworkSimulationTime, TransportResource}, 
    Result, 
};
use log::{info, error};

use crate::network::{Pack, Cmd, Dest};
use crate::resources::{AppConfig};

pub struct TcpSystemBundle;

impl<'a, 'b> SystemBundle<'a, 'b> for TcpSystemBundle {
    fn build(self, world: &mut World, builder: &mut DispatcherBuilder<'a, 'b>) -> Result<()> {
        builder.add(
            TcpSystemDesc::default().build(world),
            "client_tcp_system",
            &[],
        );
        Ok(())
    }
}

#[derive(Default, Debug)]
pub struct TcpSystemDesc;

impl<'a, 'b> SystemDesc<'a, 'b, TcpSystem> for TcpSystemDesc {
    fn build(self, world: &mut World) -> TcpSystem {
        <TcpSystem as System<'_>>::SystemData::setup(world);
        let net_reader = world
            .fetch_mut::<EventChannel<NetworkSimulationEvent>>()
            .register_reader();
        let packs_reader = world
            .fetch_mut::<EventChannel<Pack>>()
            .register_reader();
        
        TcpSystem::new(net_reader, packs_reader)
    }
}

pub struct TcpSystem {
    net_reader: ReaderId<NetworkSimulationEvent>,
    packs_reader: ReaderId<Pack>,
    connected: bool,
}

impl TcpSystem {
    pub fn new(net_reader: ReaderId<NetworkSimulationEvent>, packs_reader: ReaderId<Pack>) -> Self {
        Self { 
            net_reader,
            packs_reader,
            connected: false,
        }
    }
}

impl<'a> System<'a> for TcpSystem {
    type SystemData = (
        Read<'a, EventChannel<Pack>>,
        Read<'a, NetworkSimulationTime>,
        Write<'a, TransportResource>,
        Read<'a, EventChannel<NetworkSimulationEvent>>,
        Read<'a, AppConfig>,
    );
    fn run(&mut self, (in_packs, sim_time, mut net, channel, conf): Self::SystemData) {
        if sim_time.should_send_message_now() {
            if !self.connected {
                info!("We are not connected, ready player 1");
                let proof = format!("{} 1580235330 SignatureHere", conf.player_name);
                let p = Pack::new(Cmd::Connect(proof.to_string()), Dest::All);  
                net.send(conf.server_ip.parse().unwrap(), &p.to_bin());
                self.connected = true;
            }
            else {
                for pack in in_packs.read(&mut self.packs_reader) {
                    net.send(conf.server_ip.parse().unwrap(), &pack.to_bin());
                }
            }
        }

        // Incoming packets
        let mut packs = Vec::<Pack>::new();
        for event in channel.read(&mut self.net_reader) {
            match event {
                NetworkSimulationEvent::Message(_addr, payload) => {
                    info!("Payload: {:?}", payload);
                    if *payload != b"ok".to_vec() {
                        let pl =  Pack::from_bin(payload.to_vec());
                        info!("Payload: {:?}", pl);
                        packs.push(pl); // Add the pack to the IO vector
                    }
                }
                NetworkSimulationEvent::Connect(addr) => info!("New client connection: {}", addr),
                NetworkSimulationEvent::Disconnect(addr) => {
                    info!("Server Disconnected: {}", addr);
                }
                NetworkSimulationEvent::RecvError(e) => {
                    error!("Recv Error: {:?}", e);
                }
                NetworkSimulationEvent::SendError(e, msg) => {
                    error!("Send Error: {:?}, {:?}", e, msg);
                }
                _ => {}
            }
        }

        // Now take packs and send them off to the correct systems.
    }
}
