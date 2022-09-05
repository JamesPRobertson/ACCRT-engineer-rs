// James Robertson
//
//

const BUFFER_SIZE: usize = 8192;


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}


mod data_source {
    pub mod local_test {
        pub fn get_telemetry() -> u8 {

        }

        pub fn get_bad_telemetry() -> u8 {

        }
    }

    pub mod network {
        pub struct NetworkInfo {
            socket:    std::net::UdpSocket,
            server_ip:  String,
            _listen_ip: String,
            heartbeat: std::time::SystemTime
        }

        impl NetworkInfo {
            fn new(listen_ip: String, server_ip: String) -> NetworkInfo {
                NetworkInfo {
                    socket: std::net::UdpSocket::bind(&listen_ip).unwrap(),
                    server_ip,
                    _listen_ip: listen_ip, // TODO: this is to change which internet device we listen on
                    heartbeat: std::time::SystemTime::now()
                }
            }

            fn get_telemetry(&self) -> u8 {
                let mut buffer = [0; BUFFER_SIZE];

                let buf_len: usize = match self.network.socket.recv(&mut buffer) {
                    Ok(_) => { }
                    Err(e) => panic!("{}", e)
                };

                return buffer;
            }
        }
    }
}
